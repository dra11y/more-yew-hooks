#![cfg(feature = "storage")]

use gloo::storage::{LocalStorage, Storage};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, rc::Rc};
use web_sys::StorageEvent;
use yew::prelude::*;

use yew_hooks::use_event_with_window;

/// State handle for the [`use_local_storage_default`] hook.
#[derive(Clone, Debug)]
pub struct UseLocalStorageDefaultHandle<T> {
    inner: UseStateHandle<T>,
    key: Rc<String>,
}

impl<T> UseLocalStorageDefaultHandle<T>
where
    T: Default,
{
    /// Set a `value` for the specified key.
    pub fn set(&self, value: T)
    where
        T: Serialize + Clone,
    {
        if LocalStorage::set(&*self.key, value.clone()).is_ok() {
            let ser = serde_json::to_string(&value).unwrap_or_default();
            info!("Set storage: {} = {ser}", &*self.key);
            self.inner.set(value);
        }
    }

    /// Delete a key and it's stored value.
    /// Resets stored value to [`Default`].
    #[allow(unused)]
    pub fn delete(&self) {
        LocalStorage::delete(&*self.key);
        info!("deleting storage: {} = DEFAULT", &*self.key);
        self.inner.set(T::default());
    }
}

impl<T> Deref for UseLocalStorageDefaultHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> PartialEq for UseLocalStorageDefaultHandle<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        *self.inner == *other.inner
    }
}

/// A side-effect hook that manages a single localStorage key.
/// Returns `T::default()` if the key is not found or if deserialization fails.
///
/// Based on [`yew_hooks::use_local_storage`].
///
/// # Example
///
/// ```rust
/// # use yew::prelude::*;
/// #
/// use yew_hooks::prelude::*;
///
/// #[function_component(LocalStorage)]
/// fn local_storage() -> Html {
///     let storage = use_local_storage_default::<String>("foo".to_string());
///
///     let onclick = {
///         let storage = storage.clone();
///         Callback::from(move |_| storage.set("bar".to_string()))
///     };
///     let ondelete = {
///         let storage = storage.clone();
///         Callback::from(move |_| storage.delete())
///     };
///
///     html! {
///         <div>
///             <button onclick={onclick}>{ "Set to bar" }</button>
///             <button onclick={ondelete}>{ "Delete" }</button>
///             <p>
///                 <b>{ "Current value: " }</b>
///                 {
///                         html! { &*storage }
///                 }
///             </p>
///         </div>
///     }
/// }
/// ```
#[hook]
pub fn use_local_storage_default<T>(key: String) -> UseLocalStorageDefaultHandle<T>
where
    T: for<'de> Deserialize<'de> + Default + 'static,
{
    let inner: UseStateHandle<T> =
        use_state(|| LocalStorage::get(&key).ok().flatten().unwrap_or_default());
    let key = use_memo((), |_| key);

    {
        let key = key.clone();
        let inner = inner.clone();
        use_event_with_window("storage", move |e: StorageEvent| {
            let Some(k) = e.key() else {
                return;
            };
            if Some(LocalStorage::raw()) != e.storage_area() {
                warn!("Expected LocalStorage event for key {k}, got SessionStorage event instead");
                return;
            }
            if k == *key {
                info!("Storage event for key: {k}");
                inner.set(LocalStorage::get(&*key).unwrap_or_default());
            }
        });
    }

    UseLocalStorageDefaultHandle { inner, key }
}
