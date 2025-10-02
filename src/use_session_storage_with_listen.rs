#![cfg(feature = "storage")]

use gloo::storage::{SessionStorage, Storage};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, rc::Rc};
use web_sys::StorageEvent;
use yew::prelude::*;
use yew_hooks::use_event_with_window;

/// State handle for the [`use_session_storage_with_listen`] hook.
pub struct UseSessionStorageWithListenHandle<T> {
    inner: UseStateHandle<Option<T>>,
    key: Rc<String>,
}

impl<T> UseSessionStorageWithListenHandle<T> {
    /// Set a `value` for the specified key.
    pub fn set(&self, value: T)
    where
        T: Serialize + Clone,
    {
        if SessionStorage::set(&*self.key, value.clone()).is_ok() {
            self.inner.set(Some(value));
        }
    }

    /// Delete a key and it's stored value.
    pub fn delete(&self) {
        SessionStorage::delete(&*self.key);
        self.inner.set(None);
    }
}

impl<T> Deref for UseSessionStorageWithListenHandle<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Clone for UseSessionStorageWithListenHandle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            key: self.key.clone(),
        }
    }
}

impl<T> PartialEq for UseSessionStorageWithListenHandle<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        *self.inner == *other.inner
    }
}

/// A side-effect hook that manages a single sessionStorage key.
///
/// Fixes [`yew_hooks::use_session_storage`], which doesn't listen for changes.
///
/// # Example
///
/// ```rust
/// # use yew::prelude::*;
/// #
/// use yew_hooks::prelude::*;
///
/// #[function_component(SessionStorage)]
/// fn session_storage() -> Html {
///     let storage = use_session_storage_with_listen::<String>("foo".to_string());
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
///                     if let Some(value) = &*storage {
///                         html! { value }
///                     } else {
///                         html! {}
///                     }
///                 }
///             </p>
///         </div>
///     }
/// }
/// ```
#[hook]
pub fn use_session_storage_with_listen<T>(key: String) -> UseSessionStorageWithListenHandle<T>
where
    T: for<'de> Deserialize<'de> + 'static,
{
    let inner: UseStateHandle<Option<T>> =
        use_state(|| SessionStorage::get(&key).unwrap_or_default());
    let key = use_memo((), |_| key);

    {
        let key = key.clone();
        let inner = inner.clone();
        use_event_with_window("storage", move |e: StorageEvent| {
            let Some(k) = e.key() else {
                return;
            };
            if Some(SessionStorage::raw()) != e.storage_area() {
                warn!("Expected SessionStorage event for key {k}, got LocalStorage event instead");
                return;
            }
            if k == *key {
                info!("SessionStorage event for key: {k}");
                inner.set(SessionStorage::get(&*key).unwrap_or_default());
            }
        });
    }

    UseSessionStorageWithListenHandle { inner, key }
}
