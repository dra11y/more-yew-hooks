use gloo::utils::window;
use yew::prelude::*;
use yew_hooks::use_event_with_window;

/// Watch the browser navigator's online status
#[hook]
pub fn use_online() -> UseStateHandle<bool> {
    let online = use_state(|| window().navigator().on_line());
    {
        let online = online.clone();
        use_event_with_window("online", move |_: Event| {
            online.set(true);
        });
    }

    {
        let online = online.clone();
        use_event_with_window("offline", move |_: Event| {
            online.set(false);
        });
    }

    online
}
