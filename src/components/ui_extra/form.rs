//! Form building blocks with shadcn's classes (`form.tsx`, `input.tsx`, `textarea.tsx`,
//! `switch.tsx`, `checkbox.tsx`, `radio-group.tsx`, `select.tsx`, `date-picker.tsx`).
//! All controlled by signals. Rust/UI's `Switch`/`Textarea` cannot be bound to state, so
//! these are written here (src/ui stays vendored).
use dioxus::prelude::*;
use time::{Date, Month};
use tw_merge::tw_merge;

use super::icon::{Icon, IconType};
use super::menu::{Align, Menu};

// ---------- layout ----------

#[component]
pub fn FormItem(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    rsx! { div { "data-slot": "form-item", class: tw_merge!("grid gap-2", class.as_deref().unwrap_or("")), {children} } }
}

#[component]
pub fn FormLabel(
    #[props(into, optional)] html_for: Option<String>,
    #[props(default)] invalid: bool,
    #[props(into, optional)] class: Option<String>,
    children: Element,
) -> Element {
    let class = tw_merge!(
        "flex items-center gap-2 text-sm leading-none font-medium select-none peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
        if invalid { "text-destructive" } else { "" },
        class.as_deref().unwrap_or("")
    );
    rsx! { label { r#for: html_for, class: "{class}", {children} } }
}

#[component]
pub fn FormDescription(
    #[props(into, optional)] class: Option<String>,
    children: Element,
) -> Element {
    rsx! { p { class: tw_merge!("text-sm text-muted-foreground", class.as_deref().unwrap_or("")), {children} } }
}

/// Validation message; renders nothing when `None`.
#[component]
pub fn FormMessage(message: Option<String>) -> Element {
    let Some(m) = message else { return rsx! {} };
    rsx! { p { class: "text-sm text-destructive", "{m}" } }
}

// ---------- inputs ----------

const INPUT: &str = "flex h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none selection:bg-primary selection:text-primary-foreground placeholder:text-muted-foreground disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm dark:bg-input/30 focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40";

#[component]
pub fn TextInput(
    value: Signal<String>,
    #[props(into, optional)] id: Option<String>,
    #[props(into, optional)] placeholder: Option<String>,
    #[props(into, default = "text".to_string())] r#type: String,
    #[props(default)] invalid: bool,
    #[props(default)] disabled: bool,
    #[props(into, optional)] class: Option<String>,
    #[props(optional)] oninput: Option<EventHandler<String>>,
) -> Element {
    let mut value = value;
    rsx! {
        input {
            id,
            r#type,
            placeholder,
            disabled,
            "aria-invalid": if invalid { "true" } else { "false" },
            class: tw_merge!(INPUT, class.as_deref().unwrap_or("")),
            value: value(),
            oninput: move |e| {
                value.set(e.value());
                if let Some(h) = &oninput { h.call(e.value()) }
            },
        }
    }
}

#[component]
pub fn TextArea(
    value: Signal<String>,
    #[props(into, optional)] id: Option<String>,
    #[props(into, optional)] placeholder: Option<String>,
    #[props(default)] invalid: bool,
    #[props(into, optional)] class: Option<String>,
) -> Element {
    let mut value = value;
    rsx! {
        textarea {
            id,
            placeholder,
            "aria-invalid": if invalid { "true" } else { "false" },
            class: tw_merge!(
                "flex field-sizing-content min-h-16 w-full rounded-md border border-input bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow] outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 md:text-sm dark:bg-input/30 dark:aria-invalid:ring-destructive/40",
                class.as_deref().unwrap_or("")
            ),
            value: value(),
            oninput: move |e| value.set(e.value()),
            // SSR writes `value` as an attribute, which <textarea> ignores: render the text as
            // content too, so the first paint shows it (the `value` property drives updates).
            "{value}"
        }
    }
}

#[component]
pub fn Switch(
    checked: Signal<bool>,
    #[props(default)] disabled: bool,
    #[props(into, optional)] id: Option<String>,
) -> Element {
    let mut checked = checked;
    let state = if checked() { "checked" } else { "unchecked" };
    rsx! {
        button {
            r#type: "button",
            role: "switch",
            id,
            disabled,
            "aria-checked": "{checked()}",
            "data-state": state,
            onclick: move |_| { let v = checked(); checked.set(!v) },
            class: "peer inline-flex h-[1.15rem] w-8 shrink-0 items-center rounded-full border border-transparent shadow-xs transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input dark:data-[state=unchecked]:bg-input/80",
            span {
                "data-state": state,
                class: "pointer-events-none block size-4 rounded-full bg-background ring-0 transition-transform data-[state=checked]:translate-x-[calc(100%-2px)] data-[state=unchecked]:translate-x-0 rtl:data-[state=checked]:-translate-x-[calc(100%-2px)] dark:data-[state=checked]:bg-primary-foreground dark:data-[state=unchecked]:bg-foreground",
            }
        }
    }
}

/// Checkbox; `onchange` receives the new value (the caller owns the state).
#[component]
pub fn CheckBox(
    checked: bool,
    onchange: EventHandler<bool>,
    /// "Some selected" (header checkbox): shows a dash, counts as checked styling.
    #[props(default)]
    indeterminate: bool,
    #[props(into, optional)] id: Option<String>,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] aria_label: Option<String>,
) -> Element {
    let state = if checked || indeterminate {
        "checked"
    } else {
        "unchecked"
    };
    rsx! {
        button {
            r#type: "button",
            role: "checkbox",
            id,
            "aria-label": aria_label,
            "aria-checked": if indeterminate && !checked { "mixed".to_string() } else { checked.to_string() },
            "data-state": state,
            onclick: move |e| { e.stop_propagation(); onchange.call(!checked) },
            class: tw_merge!(
                "peer size-4 shrink-0 rounded-[4px] border border-input shadow-xs transition-shadow outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:border-primary data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground dark:bg-input/30 dark:data-[state=checked]:bg-primary",
                class.as_deref().unwrap_or("")
            ),
            if checked {
                span { class: "flex items-center justify-center text-current", Icon { icon: IconType::Check, class: "size-3.5" } }
            } else if indeterminate {
                span { class: "flex items-center justify-center text-current", Icon { icon: IconType::Minus, class: "size-3.5" } }
            }
        }
    }
}

/// Radio item for a group whose value lives in `value`.
#[component]
pub fn Radio(
    value: Signal<String>,
    item: String,
    #[props(into, optional)] id: Option<String>,
    #[props(into, optional)] class: Option<String>,
) -> Element {
    let mut value = value;
    let on = value() == item;
    rsx! {
        button {
            r#type: "button",
            role: "radio",
            id,
            "aria-checked": "{on}",
            "data-state": if on { "checked" } else { "unchecked" },
            onclick: move |_| value.set(item.clone()),
            class: tw_merge!(
                "aspect-square size-4 shrink-0 rounded-full border border-input text-primary shadow-xs transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-input/30",
                class.as_deref().unwrap_or("")
            ),
            if on {
                span { class: "relative flex items-center justify-center",
                    svg { view_box: "0 0 24 24", class: "absolute top-1/2 left-1/2 size-2 -translate-x-1/2 -translate-y-1/2 fill-primary", circle { cx: "12", cy: "12", r: "12" } }
                }
            }
        }
    }
}

const SELECT_TRIGGER: &str = "flex h-9 w-fit items-center justify-between gap-2 rounded-md border border-input bg-transparent px-3 py-2 text-sm whitespace-nowrap shadow-xs transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:bg-input/30 dark:hover:bg-input/50 [&_svg]:pointer-events-none [&_svg]:shrink-0";
const SELECT_ITEM: &str = "relative flex w-full cursor-default items-center gap-2 rounded-sm py-1.5 ps-2 pe-8 text-sm outline-hidden select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground";

/// `Select` with options `(value, label)`.
#[component]
pub fn SelectBox(
    value: Signal<String>,
    options: Vec<(String, String)>,
    placeholder: String,
    #[props(default)] invalid: bool,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] trigger_class: Option<String>,
    /// Called with the chosen value (after `value` is updated).
    #[props(optional)]
    onchange: Option<EventHandler<String>>,
) -> Element {
    let mut value = value;
    let current = options
        .iter()
        .find(|(v, _)| *v == value())
        .map(|(_, l)| l.clone());
    let empty = current.is_none();
    rsx! {
        Menu {
            match_trigger_width: true,
            trigger_class: tw_merge!("w-fit", class.as_deref().unwrap_or("")),
            trigger: rsx! {
                button {
                    r#type: "button",
                    role: "combobox",
                    "aria-invalid": if invalid { "true" } else { "false" },
                    class: tw_merge!(SELECT_TRIGGER, if empty { "text-muted-foreground" } else { "" }, trigger_class.as_deref().unwrap_or("")),
                    span { class: "line-clamp-1 flex items-center gap-2", {current.unwrap_or(placeholder)} }
                    Icon { icon: IconType::ChevronDown, class: "size-4 opacity-50" }
                }
            },
            for (v, label) in options {
                button {
                    r#type: "button",
                    role: "option",
                    class: SELECT_ITEM,
                    onclick: {
                        let v = v.clone();
                        move |_| {
                            value.set(v.clone());
                            if let Some(h) = &onchange { h.call(v.clone()) }
                        }
                    },
                    "{label}"
                    if value() == v {
                        span { class: "absolute end-2 flex size-3.5 items-center justify-center", Icon { icon: IconType::Check, class: "size-4" } }
                    }
                }
            }
        }
    }
}

/// Searchable list in a popover (`Popover` + `Command` combobox in the reference).
#[component]
pub fn Combobox(
    value: Signal<String>,
    options: Vec<(String, String)>,
    placeholder: String,
    search_placeholder: String,
    empty_text: String,
    #[props(into, optional)] class: Option<String>,
) -> Element {
    let mut value = value;
    let open = use_signal(|| false);
    let mut query = use_signal(String::new);
    let current = options
        .iter()
        .find(|(v, _)| *v == value())
        .map(|(_, l)| l.clone());
    let unset = current.is_none();
    let q = query().to_lowercase();
    let shown: Vec<(String, String)> = options
        .iter()
        .filter(|(_, l)| l.to_lowercase().contains(&q))
        .cloned()
        .collect();
    rsx! {
        Menu {
            open_signal: open,
            close_on_click: false,
            class: tw_merge!("p-0", class.as_deref().unwrap_or("")),
            trigger_class: "w-fit",
            trigger: rsx! {
                button {
                    r#type: "button",
                    role: "combobox",
                    class: tw_merge!(
                        "inline-flex h-9 items-center justify-between gap-2 rounded-md border bg-background px-4 py-2 text-sm font-medium whitespace-nowrap shadow-xs transition-all hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-input/30 dark:hover:bg-input/50",
                        class.as_deref().unwrap_or(""),
                        if unset { "text-muted-foreground" } else { "" }
                    ),
                    {current.unwrap_or(placeholder)}
                    Icon { icon: IconType::ChevronsUpDown, class: "ms-2 h-4 w-4 shrink-0 opacity-50" }
                }
            },
            div { class: "flex h-9 items-center gap-2 border-b px-3",
                Icon { icon: IconType::Search, class: "size-4 shrink-0 opacity-50" }
                input {
                    class: "flex h-10 w-full rounded-md bg-transparent py-3 text-sm outline-hidden placeholder:text-muted-foreground",
                    placeholder: search_placeholder,
                    value: query(),
                    oninput: move |e| query.set(e.value()),
                }
            }
            div { class: "max-h-[300px] overflow-y-auto p-1",
                if shown.is_empty() { div { class: "py-6 text-center text-sm", "{empty_text}" } }
                for (v, label) in shown {
                    button {
                        r#type: "button",
                        class: "relative flex w-full cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none hover:bg-accent hover:text-accent-foreground",
                        onclick: {
                            let v = v.clone();
                            let mut open = open;
                            move |_| { value.set(v.clone()); open.set(false); query.set(String::new()); }
                        },
                        Icon { icon: IconType::Check, class: if value() == v { "size-4 opacity-100" } else { "size-4 opacity-0" } }
                        "{label}"
                    }
                }
            }
        }
    }
}

// ---------- date picker ----------

/// Today in the browser (wasm) or on the server (UTC).
pub fn today() -> Date {
    #[cfg(target_arch = "wasm32")]
    {
        let d = js_sys::Date::new_0();
        let month = Month::try_from(d.get_month() as u8 + 1).unwrap_or(Month::January);
        Date::from_calendar_date(d.get_full_year() as i32, month, d.get_date() as u8)
            .unwrap_or(Date::MIN)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        time::OffsetDateTime::now_utc().date()
    }
}

pub fn format_date(d: Date) -> String {
    // "MMM d, yyyy" (date-fns), e.g. "Jan 5, 1990"
    let m = &format!("{:?}", d.month())[..3];
    format!("{m} {}, {}", d.day(), d.year())
}

/// `date-picker.tsx`: outline button + popover calendar; future dates and dates before
/// 1900 are disabled. Month and year can be picked from dropdowns (`captionLayout="dropdown"`).
#[component]
pub fn DatePicker(selected: Signal<Option<Date>>, placeholder: String) -> Element {
    let mut selected = selected;
    let open = use_signal(|| false);
    let start = selected().unwrap_or_else(today);
    let mut view = use_signal(|| (start.year(), start.month()));
    let label = selected().map(format_date);
    let empty = label.is_none();
    let (year, month) = view();
    let first = Date::from_calendar_date(year, month, 1).unwrap_or(Date::MIN);
    let lead = first.weekday().number_days_from_sunday() as i64;
    let days = month.length(year);
    let today = today();
    let min = Date::from_calendar_date(1900, Month::January, 1).unwrap_or(Date::MIN);
    let cells: Vec<Option<Date>> = (0..42)
        .map(|i| {
            let n = i - lead + 1;
            (1..=days as i64)
                .contains(&n)
                .then(|| first.replace_day(n as u8).unwrap_or(first))
        })
        .collect();
    let mut shift = move |delta: i32| {
        let (y, m) = view();
        let idx = y * 12 + (m as i32 - 1) + delta;
        let m = Month::try_from((idx.rem_euclid(12) + 1) as u8).unwrap_or(Month::January);
        view.set((idx.div_euclid(12), m));
    };
    let months: Vec<Month> = (1..=12).filter_map(|m| Month::try_from(m).ok()).collect();

    rsx! {
        Menu {
            open_signal: open,
            close_on_click: false,
            align: Align::Start,
            class: "w-auto p-0",
            trigger_class: "w-fit",
            trigger: rsx! {
                button {
                    r#type: "button",
                    "data-empty": "{empty}",
                    class: "inline-flex h-9 w-60 items-center justify-start gap-2 rounded-md border bg-background px-4 py-2 text-start text-sm font-normal whitespace-nowrap shadow-xs transition-all hover:bg-accent hover:text-accent-foreground data-[empty=true]:text-muted-foreground dark:border-input dark:bg-input/30",
                    {label.unwrap_or(placeholder)}
                    Icon { icon: IconType::Calendar, class: "ms-auto h-4 w-4 opacity-50" }
                }
            },
            div { class: "bg-background p-3 [--cell-size:--spacing(8)]",
                div { class: "flex h-(--cell-size) items-center justify-between gap-1",
                    button { r#type: "button", class: "inline-flex size-(--cell-size) items-center justify-center rounded-md hover:bg-accent", "aria-label": "Previous month", onclick: move |_| shift(-1), Icon { icon: IconType::ChevronLeft, class: "size-4" } }
                    div { class: "flex items-center gap-1.5 text-sm font-medium",
                        select {
                            class: "rounded-md border border-input bg-background px-1 py-0.5 text-sm shadow-xs",
                            "aria-label": "Month",
                            value: "{month as u8}",
                            onchange: move |e| { if let Some(m) = e.value().parse::<u8>().ok().and_then(|m| Month::try_from(m).ok()) { view.set((view().0, m)) } },
                            for m in months { option { value: "{m as u8}", selected: m == month, {format!("{m:?}")[..3].to_string()} } }
                        }
                        select {
                            class: "rounded-md border border-input bg-background px-1 py-0.5 text-sm shadow-xs",
                            "aria-label": "Year",
                            value: "{year}",
                            onchange: move |e| { if let Ok(y) = e.value().parse::<i32>() { view.set((y, view().1)) } },
                            for y in (1900..=today.year()).rev() { option { value: "{y}", selected: y == year, "{y}" } }
                        }
                    }
                    button { r#type: "button", class: "inline-flex size-(--cell-size) items-center justify-center rounded-md hover:bg-accent", "aria-label": "Next month", onclick: move |_| shift(1), Icon { icon: IconType::ChevronRight, class: "size-4" } }
                }
                div { class: "mt-4 grid grid-cols-7",
                    for wd in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
                        div { class: "flex h-(--cell-size) w-(--cell-size) items-center justify-center text-[0.8rem] font-normal text-muted-foreground select-none", "{wd}" }
                    }
                    for cell in cells {
                        match cell {
                            None => rsx! { div { class: "size-(--cell-size)" } },
                            Some(d) => {
                                let disabled = d > today || d < min;
                                let is_sel = selected() == Some(d);
                                rsx! {
                                    button {
                                        r#type: "button",
                                        disabled,
                                        class: tw_merge!(
                                            "inline-flex size-(--cell-size) items-center justify-center rounded-md text-sm font-normal hover:bg-accent hover:text-accent-foreground disabled:pointer-events-none disabled:opacity-50",
                                            if d == today { "bg-accent text-accent-foreground" } else { "" },
                                            if is_sel { "bg-primary text-primary-foreground hover:bg-primary hover:text-primary-foreground" } else { "" }
                                        ),
                                        onclick: {
                                            let mut open = open;
                                            move |_| { selected.set(Some(d)); open.set(false); }
                                        },
                                        "{d.day()}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
