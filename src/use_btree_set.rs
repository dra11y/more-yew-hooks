use std::{
    cell::{Ref, RefCell},
    collections::BTreeSet,
    hash::Hash,
    rc::Rc,
};
use yew::prelude::*;
use yew_hooks::use_update;

/// State handle for the [`use_btree_set`] hook.
pub struct UseBTreeSetHandle<T>
where
    T: Eq + Hash + Ord,
{
    inner: Rc<RefCell<BTreeSet<T>>>,
    update: Rc<dyn Fn()>,
}

impl<T> UseBTreeSetHandle<T>
where
    T: Eq + Hash + Ord,
{
    /// Get immutable ref to the set.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed
    pub fn current(&'_ self) -> Ref<'_, BTreeSet<T>> {
        self.inner.borrow()
    }

    /// Set the BTree set.
    pub fn set(&self, set: BTreeSet<T>) {
        *self.inner.borrow_mut() = set;
        (self.update)();
    }

    /// Adds a value to the BTree set.
    pub fn insert(&self, value: T) -> bool {
        let present = self.inner.borrow_mut().insert(value);
        (self.update)();
        present
    }

    /// Adds a value to the set, replacing the existing value,
    /// if any, that is equal to the given one. Returns the replaced value.
    pub fn replace(&self, value: T) -> Option<T> {
        let v = self.inner.borrow_mut().replace(value);
        (self.update)();
        v
    }

    /// Removes a value from the set. Returns whether the value was present in the set.
    pub fn remove(&self, value: &T) -> bool {
        let present = self.inner.borrow_mut().remove(value);
        (self.update)();
        present
    }

    /// Retains only the elements specified by the predicate.
    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.borrow_mut().retain(f);
        (self.update)();
    }

    /// Clears the set, removing all values.
    pub fn clear(&self) {
        self.inner.borrow_mut().clear();
        (self.update)();
    }
}

impl<T> Clone for UseBTreeSetHandle<T>
where
    T: Eq + Hash + Ord,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            update: self.update.clone(),
        }
    }
}

impl<T> PartialEq for UseBTreeSetHandle<T>
where
    T: Eq + Hash + Ord,
{
    fn eq(&self, other: &Self) -> bool {
        *self.inner == *other.inner
    }
}

/// A hook that tracks a [`BTreeSet`] and provides methods to modify it.
///
/// Based on [`yew_hooks::use_set`].
///
/// # Example
///
/// ```rust
/// # use std::collections::BTreeSet;
/// # use yew::prelude::*;
/// #
/// use yew_hooks::prelude::*;
///
/// #[function_component(UseSet)]
/// fn set() -> Html {
///     let set = use_btree_set(BTreeSet::from(["Mercury", "Venus", "Earth", "Mars"]));
///
///     let onset = {
///         let set = set.clone();
///         Callback::from(move |_| set.set(BTreeSet::from(["Moon", "Earth"])))
///     };
///     let oninsert = {
///         let set = set.clone();
///         Callback::from(move |_| {
///             let _ = set.insert("Jupiter");
///         })
///     };
///     let onreplace = {
///         let set = set.clone();
///         Callback::from(move |_| {
///             let _ = set.replace("Earth");
///         })
///     };
///     let onremove = {
///         let set = set.clone();
///         Callback::from(move |_| {
///             let _ = set.remove(&"Moon");
///         })
///     };
///     let onretain = {
///         let set = set.clone();
///         Callback::from(move |_| set.retain(|v| v.contains('a')))
///     };
///     let onclear = {
///         let set = set.clone();
///         Callback::from(move |_| set.clear())
///     };
///
///     html! {
///         <div>
///             <button onclick={onset}>{ "Set" }</button>
///             <button onclick={oninsert}>{ "Insert" }</button>
///             <button onclick={onreplace}>{ "Replace" }</button>
///             <button onclick={onremove}>{ "Remove" }</button>
///             <button onclick={onretain}>{ "Retain" }</button>
///             <button onclick={onclear}>{ "Clear all" }</button>
///             <p>
///                 <b>{ "Current value: " }</b>
///             </p>
///             {
///                 for set.current().iter().map(|v| {
///                     html! {
///                         <p><b>{ v }</b></p>
///                     }
///                 })
///             }
///         </div>
///     }
/// }
/// ```
#[hook]
pub fn use_btree_set<T>(initial_value: BTreeSet<T>) -> UseBTreeSetHandle<T>
where
    T: 'static + Eq + Hash + Ord,
{
    let inner = use_mut_ref(|| initial_value);
    let update = use_update();

    UseBTreeSetHandle { inner, update }
}
