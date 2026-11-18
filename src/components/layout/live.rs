//! Live updates (§11, M8.4): one `EventSource('/api/stream')` per tab. Each topic is a
//! counter; a resource that reads it refetches when the server signals a change.
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct Live {
    pub audit: Signal<u32>,
    pub users: Signal<u32>,
    pub roles: Signal<u32>,
    pub backups: Signal<u32>,
    pub dashboard: Signal<u32>,
    pub notifications: Signal<u32>,
}

pub fn use_live() -> Live {
    use_context::<Live>()
}

/// Called once by `AppShell`.
pub fn use_live_provider() {
    let live = use_context_provider(|| Live {
        audit: Signal::new(0),
        users: Signal::new(0),
        roles: Signal::new(0),
        backups: Signal::new(0),
        dashboard: Signal::new(0),
        notifications: Signal::new(0),
    });
    use_effect(move || {
        spawn(async move {
            let mut ev = document::eval(
                "const es = new EventSource('/api/stream'); let down = false;
                 ['audit','users','roles','backups','dashboard','notifications','session_ended','resync']
                   .forEach(t => es.addEventListener(t, () => dioxus.send(t)));
                 es.addEventListener('ready', () => { if (down) dioxus.send('resync'); down = false; });
                 es.onerror = () => { down = true; };",
            );
            let bump = |mut s: Signal<u32>| *s.write() += 1;
            while let Ok(topic) = ev.recv::<String>().await {
                match topic.as_str() {
                    "audit" => bump(live.audit),
                    "users" => bump(live.users),
                    "roles" => bump(live.roles),
                    "backups" => bump(live.backups),
                    "dashboard" => bump(live.dashboard),
                    "notifications" => bump(live.notifications),
                    "session_ended" => crate::util::session_ended(),
                    // Reconnected or missed messages: refetch everything once.
                    _ => {
                        for s in [
                            live.audit,
                            live.users,
                            live.roles,
                            live.backups,
                            live.dashboard,
                            live.notifications,
                        ] {
                            bump(s);
                        }
                    }
                }
            }
        });
    });
}
