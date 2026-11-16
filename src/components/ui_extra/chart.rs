//! SVG bar and area charts reproducing the Recharts output used by shadcn-admin's
//! dashboard (`overview.tsx`, `analytics-chart.tsx`): 12px #888 axis labels without axis or
//! tick lines, "nice" Y ticks from 0, rounded bar tops, monotone area curves.
//! Width follows the container (`ResizeObserver`); SSR renders a 600px default.
use dioxus::prelude::*;

const Y_AXIS_WIDTH: f64 = 60.0;
const X_AXIS_HEIGHT: f64 = 30.0;
const MARGIN: f64 = 5.0;
const TICK_GAP: f64 = 8.0;
const AXIS_COLOR: &str = "#888888";

/// One area series: values plus the Tailwind text-colour class used for stroke and fill.
#[derive(Clone, PartialEq)]
pub struct Series {
    pub values: Vec<f64>,
    pub class: &'static str,
    pub fill_opacity: f64,
}

/// Axis ticks from 0, as Recharts' `getNiceTickValues` (tickCount 5) computes them: the
/// rough step is rounded up to a multiple of 0.05 × 10ⁿ (0.1 × 10ⁿ for one-digit steps),
/// so a maximum of 5,800 gives 0 / 1,500 / 3,000 / 4,500 / 6,000.
pub fn nice_ticks(max: f64, count: usize) -> Vec<f64> {
    if max <= 0.0 || count < 2 {
        return vec![0.0];
    }
    let rough = max / (count - 1) as f64;
    let digits = rough.log10().floor() + 1.0;
    let unit = 10f64.powf(digits);
    // Scale in hundredths of `unit`, so steps like 1,500 stay exact in floating point.
    let scale = if digits == 1.0 { 10.0 } else { 5.0 };
    let mut correction = 0.0;
    loop {
        let step = ((rough / unit * 100.0 / scale).ceil() + correction) * scale * unit / 100.0;
        if step * (count - 1) as f64 >= max {
            return (0..count).map(|i| i as f64 * step).collect();
        }
        correction += 1.0;
    }
}

/// d3 `curveMonotoneX` as an SVG path (no overshoot between points).
pub fn monotone_path(pts: &[(f64, f64)]) -> String {
    let n = pts.len();
    if n == 0 {
        return String::new();
    }
    if n < 3 {
        return pts
            .iter()
            .enumerate()
            .map(|(i, (x, y))| format!("{}{x:.2},{y:.2}", if i == 0 { "M" } else { "L" }))
            .collect();
    }
    let sign = |v: f64| {
        if v > 0.0 {
            1.0
        } else if v < 0.0 {
            -1.0
        } else {
            0.0
        }
    };
    let slope = |i: usize| (pts[i + 1].1 - pts[i].1) / (pts[i + 1].0 - pts[i].0);
    let mut t = vec![0.0; n];
    for i in 1..n - 1 {
        let (h0, h1) = (pts[i].0 - pts[i - 1].0, pts[i + 1].0 - pts[i].0);
        let (s0, s1) = (slope(i - 1), slope(i));
        let p = (s0 * h1 + s1 * h0) / (h0 + h1);
        t[i] = (sign(s0) + sign(s1)) * s0.abs().min(s1.abs()).min(0.5 * p.abs());
    }
    t[0] = (3.0 * slope(0) - t[1]) / 2.0;
    t[n - 1] = (3.0 * slope(n - 2) - t[n - 2]) / 2.0;
    let mut d = format!("M{:.2},{:.2}", pts[0].0, pts[0].1);
    for i in 0..n - 1 {
        let ((x0, y0), (x1, y1)) = (pts[i], pts[i + 1]);
        let dx = (x1 - x0) / 3.0;
        d.push_str(&format!(
            "C{:.2},{:.2},{:.2},{:.2},{x1:.2},{y1:.2}",
            x0 + dx,
            y0 + dx * t[i],
            x1 - dx,
            y1 - dx * t[i + 1]
        ));
    }
    d
}

/// Width of the chart's wrapper, kept current by a `ResizeObserver` (container resize,
/// window resize, or a hidden tab becoming visible). SSR renders the 600px default.
/// Returns the width signal and the element id to put on the wrapper.
fn use_width() -> (Signal<f64>, String) {
    // The scope id is the same during SSR and hydration (same component tree), unlike a counter.
    let id = use_hook(|| format!("chart-{}", dioxus::core::current_scope_id().0));
    let mut width = use_signal(|| 600.0);
    let observed = id.clone();
    use_effect(move || {
        let id = observed.clone();
        spawn(async move {
            let mut ev = document::eval(&format!(
                "const el = document.getElementById('{id}'); if (el) new ResizeObserver(es => {{
                   const w = es[0].contentRect.width; if (w > 0) dioxus.send(w); }}).observe(el);"
            ));
            while let Ok(w) = ev.recv::<f64>().await {
                width.set(w.max(200.0));
            }
        });
    });
    (width, id)
}

#[component]
fn YAxis(ticks: Vec<f64>, top: f64, plot_h: f64, max: f64, prefix: &'static str) -> Element {
    rsx! {
        for v in ticks {
            text {
                x: "{Y_AXIS_WIDTH - TICK_GAP + MARGIN}",
                y: "{top + plot_h - v / max * plot_h}",
                "text-anchor": "end",
                "dominant-baseline": "central",
                "font-size": "12",
                fill: AXIS_COLOR,
                "{prefix}{v}"
            }
        }
    }
}

/// Recharts `BarChart` + `Bar radius={[4,4,0,0]}` (`overview.tsx`).
#[component]
pub fn BarChart(
    labels: Vec<&'static str>,
    values: Vec<f64>,
    #[props(default = 350.0)] height: f64,
    #[props(default = "")] y_prefix: &'static str,
    #[props(default = "fill-primary")] bar_class: &'static str,
) -> Element {
    let (width, id) = use_width();
    let w = width();
    let ticks = nice_ticks(values.iter().cloned().fold(0.0, f64::max), 5);
    let max = *ticks.last().unwrap_or(&1.0);
    let (left, top) = (MARGIN + Y_AXIS_WIDTH, MARGIN);
    let plot_w = (w - left - MARGIN).max(1.0);
    let plot_h = height - top - MARGIN - X_AXIS_HEIGHT;
    let band = plot_w / values.len().max(1) as f64;
    let bar_w = band * 0.8;
    let r = 4.0_f64.min(bar_w / 2.0);

    let bars: Vec<(String, f64, &'static str)> = values
        .iter()
        .zip(&labels)
        .enumerate()
        .map(|(i, (v, l))| {
            let x = left + band * i as f64 + (band - bar_w) / 2.0;
            let h = (v / max * plot_h).max(0.0);
            let y = top + plot_h - h;
            let rr = r.min(h);
            // Rounded top corners only.
            let d = format!(
                "M{x:.2},{b:.2}L{x:.2},{y1:.2}Q{x:.2},{y:.2} {x1:.2},{y:.2}L{x2:.2},{y:.2}Q{xr:.2},{y:.2} {xr:.2},{y1:.2}L{xr:.2},{b:.2}Z",
                b = top + plot_h,
                y1 = y + rr,
                x1 = x + rr,
                x2 = x + bar_w - rr,
                xr = x + bar_w,
            );
            (d, left + band * (i as f64 + 0.5), *l)
        })
        .collect();

    rsx! {
        div { id: "{id}", class: "w-full",
            svg { width: "{w}", height: "{height}", view_box: "0 0 {w} {height}", class: "overflow-visible", role: "img",
                YAxis { ticks, top, plot_h, max, prefix: y_prefix }
                for (d, cx, label) in bars {
                    path { d: "{d}", class: bar_class, fill: "currentColor" }
                    text {
                        x: "{cx}",
                        y: "{top + plot_h + TICK_GAP}",
                        dy: "0.71em",
                        "text-anchor": "middle",
                        "font-size": "12",
                        fill: AXIS_COLOR,
                        "{label}"
                    }
                }
            }
        }
    }
}

/// Recharts `AreaChart` with `type="monotone"` areas (`analytics-chart.tsx`).
#[component]
pub fn AreaChart(
    labels: Vec<&'static str>,
    series: Vec<Series>,
    #[props(default = 300.0)] height: f64,
) -> Element {
    let (width, id) = use_width();
    let w = width();
    let all_max = series
        .iter()
        .flat_map(|s| s.values.iter().cloned())
        .fold(0.0, f64::max);
    let ticks = nice_ticks(all_max, 5);
    let max = *ticks.last().unwrap_or(&1.0);
    let (left, top) = (MARGIN + Y_AXIS_WIDTH, MARGIN);
    let plot_w = (w - left - MARGIN).max(1.0);
    let plot_h = height - top - MARGIN - X_AXIS_HEIGHT;
    let n = labels.len().max(2);
    let x_at = |i: usize| left + plot_w * i as f64 / (n - 1) as f64;
    let base = top + plot_h;

    let paths: Vec<(String, String, &'static str, f64)> = series
        .iter()
        .map(|s| {
            let pts: Vec<(f64, f64)> = s
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| (x_at(i), base - v / max * plot_h))
                .collect();
            let line = monotone_path(&pts);
            let area = format!(
                "{line}L{:.2},{base:.2}L{:.2},{base:.2}Z",
                pts.last().map_or(left, |p| p.0),
                pts.first().map_or(left, |p| p.0)
            );
            (line, area, s.class, s.fill_opacity)
        })
        .collect();

    rsx! {
        div { id: "{id}", class: "w-full",
            svg { width: "{w}", height: "{height}", view_box: "0 0 {w} {height}", class: "overflow-visible", role: "img",
                YAxis { ticks, top, plot_h, max, prefix: "" }
                for (line, area, class, opacity) in paths {
                    g { class,
                        path { d: "{area}", fill: "currentColor", "fill-opacity": "{opacity}", stroke: "none" }
                        path { d: "{line}", fill: "none", stroke: "currentColor", "stroke-width": "1" }
                    }
                }
                for (i, label) in labels.iter().enumerate() {
                    text {
                        x: "{x_at(i)}",
                        y: "{base + TICK_GAP}",
                        dy: "0.71em",
                        "text-anchor": if i == 0 { "start" } else if i + 1 == labels.len() { "end" } else { "middle" },
                        "font-size": "12",
                        fill: AXIS_COLOR,
                        "{label}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_ticks_like_recharts() {
        assert_eq!(
            nice_ticks(5800.0, 5),
            vec![0.0, 1500.0, 3000.0, 4500.0, 6000.0]
        );
        assert_eq!(nice_ticks(940.0, 5), vec![0.0, 250.0, 500.0, 750.0, 1000.0]);
        assert_eq!(nice_ticks(0.0, 5), vec![0.0]);
    }

    #[test]
    fn monotone_path_passes_through_points() {
        let d = monotone_path(&[(0.0, 10.0), (10.0, 0.0), (20.0, 5.0)]);
        assert!(d.starts_with("M0.00,10.00C"));
        assert!(d.ends_with("20.00,5.00"));
        assert_eq!(d.matches('C').count(), 2);
    }
}
