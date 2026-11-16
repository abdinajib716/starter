//! Data-table toolkit (`components/data-table/*` + `hooks/use-table-url-state.ts`): state with
//! URL sync, toolbar (search, faceted filters, reset, view options), sortable column headers,
//! pagination and the floating bulk-actions bar. Pages own their rows and columns and apply
//! `TableState` to them (filter → sort → paginate), as TanStack Table does for the reference.
mod bulk_actions;
mod column_header;
mod faceted_filter;
mod multi_delete;
mod pagination;
mod toolbar;
mod view_options;

pub use bulk_actions::BulkActions;
pub use column_header::ColumnHeader;
pub use faceted_filter::FacetedFilter;
pub use multi_delete::{DeleteWarning, ICON_BTN, ICON_BTN_DESTRUCTIVE, MultiDeleteBody};
pub use pagination::Pagination;
#[cfg(test)]
use pagination::page_numbers;
pub use toolbar::Toolbar;
pub use view_options::ViewOptions;

use std::collections::HashSet;

use dioxus::prelude::*;

use crate::components::ui_extra::icon::IconType;
use crate::util::query::Query;

/// One choice of a faceted filter (`statuses`, `priorities`, `roles`…).
#[derive(Clone, PartialEq)]
pub struct FilterOption {
    pub value: String,
    pub label: String,
    pub icon: Option<IconType>,
}

/// A faceted filter for one column, with counts of each value in the current rows.
#[derive(Clone, PartialEq)]
pub struct FilterDef {
    pub column: &'static str,
    pub title: String,
    pub options: Vec<FilterOption>,
    /// (value, count) over the rows matching every *other* filter (TanStack "faceted" values).
    pub counts: Vec<(String, usize)>,
}

pub const PAGE_SIZES: [usize; 5] = [10, 20, 30, 40, 50];
const DEFAULT_PAGE_SIZE: usize = 10;

/// Table state shared by the toolbar, headers, rows and pagination. All signals: `Copy`.
#[derive(Clone, Copy, PartialEq)]
pub struct TableState {
    pub filter: Signal<String>,
    pub column_filters: Signal<Vec<(&'static str, Vec<String>)>>,
    /// (column, descending)
    pub sort: Signal<Option<(&'static str, bool)>>,
    /// 0-based
    pub page: Signal<usize>,
    pub page_size: Signal<usize>,
    pub selected: Signal<HashSet<String>>,
    pub hidden: Signal<HashSet<&'static str>>,
    /// URL key of the search box: `filter` (global, tasks) or a column (`username`).
    pub filter_key: &'static str,
}

/// The URL-synced part of the table state (`hooks/use-table-url-state.ts`), as plain data so
/// it can be tested without a Dioxus runtime. Arrays are comma-separated (`status=todo,done`).
#[derive(Debug, Clone, PartialEq)]
pub struct UrlState {
    pub filter: String,
    pub columns: Vec<(&'static str, Vec<String>)>,
    /// 0-based
    pub page: usize,
    pub page_size: usize,
}

impl UrlState {
    pub fn parse(q: &Query, filter_key: &str, filter_columns: &[&'static str]) -> Self {
        let num = |k: &str| q.get(k).and_then(|v| v.parse::<usize>().ok());
        let columns = filter_columns
            .iter()
            .filter_map(|c| {
                let v: Vec<String> = q
                    .get(c)?
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect();
                (!v.is_empty()).then_some((*c, v))
            })
            .collect();
        Self {
            filter: q.get(filter_key).unwrap_or("").to_string(),
            columns,
            page: num("page").unwrap_or(1).max(1) - 1,
            page_size: num("pageSize")
                .filter(|s| PAGE_SIZES.contains(s))
                .unwrap_or(DEFAULT_PAGE_SIZE),
        }
    }

    /// Defaults (page 1, size 10, empty filters) are left out, like the reference.
    pub fn build(&self, filter_key: &str, filter_columns: &[&'static str]) -> Query {
        let page = self.page + 1;
        let mut q = Query::default()
            .with("page", (page > 1).then(|| page.to_string()))
            .with(
                "pageSize",
                (self.page_size != DEFAULT_PAGE_SIZE).then(|| self.page_size.to_string()),
            )
            .with(filter_key, Some(self.filter.trim().to_string()));
        for c in filter_columns {
            let v = self
                .columns
                .iter()
                .find(|(k, _)| k == c)
                .map(|(_, v)| v.join(","));
            q = q.with(c, v);
        }
        q
    }
}

impl TableState {
    /// Read the initial state from the URL (`?page=2&pageSize=20&filter=x&status=todo,done`).
    pub fn use_from_query(
        query: &Query,
        filter_key: &'static str,
        filter_columns: &'static [&'static str],
    ) -> Self {
        let q = query.clone();
        use_hook(move || {
            let u = UrlState::parse(&q, filter_key, filter_columns);
            Self {
                filter: Signal::new(u.filter),
                column_filters: Signal::new(u.columns),
                sort: Signal::new(None),
                page: Signal::new(u.page),
                page_size: Signal::new(u.page_size),
                selected: Signal::new(HashSet::new()),
                hidden: Signal::new(HashSet::new()),
                filter_key,
            }
        })
    }

    /// The URL query for the current state.
    pub fn to_query(self, filter_columns: &[&'static str]) -> Query {
        UrlState {
            filter: (self.filter)(),
            columns: (self.column_filters)(),
            page: (self.page)(),
            page_size: (self.page_size)(),
        }
        .build(self.filter_key, filter_columns)
    }

    pub fn column_filter(&self, column: &str) -> Vec<String> {
        (self.column_filters)()
            .iter()
            .find(|(c, _)| *c == column)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    }

    pub fn set_column_filter(mut self, column: &'static str, values: Vec<String>) {
        let mut all = self.column_filters.write();
        all.retain(|(c, _)| *c != column);
        if !values.is_empty() {
            all.push((column, values));
        }
        drop(all);
        self.page.set(0);
    }

    pub fn set_filter(mut self, value: String) {
        self.filter.set(value);
        self.page.set(0);
    }

    pub fn is_filtered(&self) -> bool {
        !(self.column_filters)().is_empty() || !(self.filter)().trim().is_empty()
    }

    pub fn reset_filters(mut self) {
        self.column_filters.set(Vec::new());
        self.filter.set(String::new());
        self.page.set(0);
    }

    /// Value passes the column filter (empty filter = everything).
    pub fn matches(&self, column: &str, value: &str) -> bool {
        let f = self.column_filter(column);
        f.is_empty() || f.iter().any(|v| v == value)
    }

    pub fn visible(&self, column: &str) -> bool {
        !(self.hidden)().contains(column)
    }

    /// Keep the page inside `page_count` after filtering (`ensurePageInRange`).
    pub fn clamp_page(mut self, page_count: usize) {
        let last = page_count.saturating_sub(1);
        if *self.page.peek() > last {
            self.page.set(last);
        }
    }
}

/// Slice of `rows` for the current page and the page count (at least 1).
pub fn paginate<T: Clone>(rows: &[T], state: &TableState) -> (Vec<T>, usize) {
    let size = (state.page_size)().max(1);
    let pages = rows.len().div_ceil(size).max(1);
    let page = (state.page)().min(pages - 1);
    (
        rows.iter().skip(page * size).take(size).cloned().collect(),
        pages,
    )
}

/// shadcn `table.tsx` classes.
pub const TH: &str = "h-10 px-2 text-start align-middle font-medium whitespace-nowrap text-foreground *:[[role=checkbox]]:translate-y-0.5";
pub const TD: &str = "p-2 align-middle whitespace-nowrap *:[[role=checkbox]]:translate-y-0.5";
pub const TR: &str = "border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted";

#[cfg(test)]
mod tests {
    use super::*;

    const COLS: &[&str] = &["status", "priority"];
    fn n(v: &[usize]) -> Vec<Option<usize>> {
        v.iter().map(|&x| (x != 0).then_some(x)).collect()
    }

    // lib/utils.test.ts (0 stands for '...')
    #[test]
    fn page_numbers_all_when_few() {
        assert_eq!(page_numbers(1, 3), n(&[1, 2, 3]));
        assert_eq!(page_numbers(3, 5), n(&[1, 2, 3, 4, 5]));
    }
    #[test]
    fn page_numbers_ellipsis_near_start() {
        assert_eq!(page_numbers(1, 10), n(&[1, 2, 3, 4, 0, 10]));
        assert_eq!(page_numbers(3, 10), n(&[1, 2, 3, 4, 0, 10]));
    }
    #[test]
    fn page_numbers_ellipsis_near_end() {
        assert_eq!(page_numbers(10, 10), n(&[1, 0, 7, 8, 9, 10]));
        assert_eq!(page_numbers(9, 10), n(&[1, 0, 7, 8, 9, 10]));
    }
    #[test]
    fn page_numbers_ellipsis_both_sides() {
        assert_eq!(page_numbers(5, 10), n(&[1, 0, 4, 5, 6, 0, 10]));
    }
    #[test]
    fn page_numbers_current_beyond_total() {
        assert_eq!(page_numbers(6, 5), n(&[1, 2, 3, 4, 5]));
        assert_eq!(page_numbers(11, 10), n(&[1, 0, 7, 8, 9, 10]));
    }

    // hooks/use-table-url-state.test.ts
    #[test]
    fn url_pagination_from_search_with_defaults() {
        let u = UrlState::parse(&Query::from("page=3&pageSize=20"), "filter", COLS);
        assert_eq!((u.page, u.page_size), (2, 20));
        let u = UrlState::parse(&Query::default(), "filter", COLS);
        assert_eq!((u.page, u.page_size), (0, 10));
        let u = UrlState::parse(&Query::from("page=0"), "filter", COLS);
        assert_eq!(u.page, 0, "page below 1 clamps to the first page");
    }
    #[test]
    fn url_omits_defaults_and_writes_others() {
        let mut u = UrlState::parse(&Query::default(), "filter", COLS);
        assert_eq!(u.build("filter", COLS).to_string(), "");
        u.page = 1;
        u.page_size = 20;
        assert_eq!(u.build("filter", COLS).to_string(), "page=2&pageSize=20");
    }
    #[test]
    fn url_filter_trimmed_and_cleared_when_empty() {
        let mut u = UrlState::parse(&Query::from("filter=abc"), "filter", COLS);
        assert_eq!(u.filter, "abc");
        u.filter = "   ".into();
        assert_eq!(u.build("filter", COLS).to_string(), "");
        u.filter = " x ".into();
        assert_eq!(u.build("filter", COLS).to_string(), "filter=x");
    }
    #[test]
    fn url_column_filters_round_trip() {
        let u = UrlState::parse(
            &Query::from("status=todo,done&username=jo"),
            "username",
            COLS,
        );
        assert_eq!(
            u.columns,
            vec![("status", vec!["todo".to_string(), "done".to_string()])]
        );
        assert_eq!(u.filter, "jo");
        assert_eq!(
            u.build("username", COLS).to_string(),
            "username=jo&status=todo%2Cdone"
        );
    }
}
