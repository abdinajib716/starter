//! `navigation-progress.tsx`: the `react-top-loading-bar` line at the top of the window on
//! every page change. Same markup and timing as the reference (measured on the demo):
//! a 2px `--muted-foreground` bar with a glowing tip; it jumps to a random 10–20% (500ms),
//! completes to 100% (500ms), waits 1s, then fades out (300ms ease-out).
//! Dioxus pages render at once (data loads behind Suspense), so "complete" follows the
//! start after 250ms — about what the demo's router takes.
use dioxus::prelude::*;

use crate::app::Route;

const ANIMATE: &str = r#"(() => {
  const bar = document.getElementById('nav-progress-bar');
  const tip = document.getElementById('nav-progress-tip');
  if (!bar || !tip) return;
  (window.__navProgress || []).forEach(clearTimeout);
  const set = (w, t) => { bar.style.transition = t; bar.style.width = w + '%'; tip.style.left = (w - 5.5) + '%'; };
  bar.style.opacity = '1';
  set(0, 'none');
  void bar.offsetWidth;
  set(10 + Math.floor(Math.random() * 11), '500ms');
  window.__navProgress = [
    setTimeout(() => set(100, '500ms'), 250),
    setTimeout(() => { bar.style.transition = '300ms ease-out'; bar.style.opacity = '0'; }, 1250),
    setTimeout(() => set(0, 'none'), 1550),
  ];
})()"#;

#[component]
pub fn NavigationProgress() -> Element {
    let route = use_route::<Route>();
    // Path only: query changes (typing in a table filter) are not page changes.
    let path = route
        .to_string()
        .split('?')
        .next()
        .unwrap_or_default()
        .to_string();
    let mut last = use_signal(|| path.clone());
    use_effect(use_reactive!(|path| {
        if *last.peek() != path {
            last.set(path);
            let _ = document::eval(ANIMATE);
        }
    }));
    rsx! {
        div {
            "aria-hidden": "true",
            style: "position: fixed; top: 0; left: 0; height: 2px; background: transparent; z-index: 2147483647; width: 100%; pointer-events: none;",
            div {
                id: "nav-progress-bar",
                style: "height: 100%; background: var(--muted-foreground); width: 0%; opacity: 1; color: var(--muted-foreground);",
                div {
                    id: "nav-progress-tip",
                    style: "box-shadow: 0 0 10px var(--muted-foreground), 0 0 10px var(--muted-foreground); width: 5%; opacity: 1; position: absolute; height: 100%; transform: rotate(2deg) translate(0px, -2px); left: -5.5%;",
                }
            }
        }
    }
}
