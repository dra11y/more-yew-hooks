# more-yew-hooks

[![Crates.io](https://img.shields.io/crates/v/more-yew-hooks.svg)](https://crates.io/crates/more-yew-hooks)
[![Docs](https://docs.rs/more-yew-hooks/badge.svg)](https://docs.rs/more-yew-hooks)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](#license)
[![Rust 2024 Edition](https://img.shields.io/badge/edition-2024-orange)](https://doc.rust-lang.org/edition-guide/)

Additional hooks for the [`yew`](https://yew.rs) ecosystem, including bugfixes for [`yew-hooks`][yew-hooks].

## Motivation

I created this crate because there were (still are?) bugs/gaps in some common [`yew-hooks`][yew-hooks] hooks, such as `use_debounce` (did not work at all),  `use_session_storage` (lacked a = listener like `use_local_storage`), and `use_local_storage_with_default` (to get `T::default()` if the key does not exist without having to unwrap on every use).

I've also added a couple additional hooks that may be useful such as `use_online` and `use_btree_set`.

## `yew-hooks` Bugfixes

Differences relative to existing `yew-hooks` utilities:

- `use_debounce` / `use_debounce_state`: accept `Fn()` instead of `FnOnce()` so the same handle can be triggered multiple times.
- `use_local_storage_default`: returns `T::default()` on absence or deserialization failure and listens to `storage` events.
- `use_session_storage_with_listen`: listens for `storage` events and filters by `sessionStorage` in case keys conflict with `localStorage`.
- `use_btree_set`: ordered set state with operations (`insert`, `replace`, `retain`, etc.).
- `use_online`: minimal wrapper around `navigator.onLine` with event listeners.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
more-yew-hooks = "0.1"
```

Optional feature flags:

| Feature | Default | Purpose |
|---------|---------|---------|
| `storage` | enabled | Enables hooks that serialize to Web Storage (`serde`, `serde_json`). |

If you disable default features and only want non-storage hooks:

```toml
more-yew-hooks = { version = "0.1", default-features = false }
```

## MSRV (Minimum Supported Rust Version)

Uses Rust edition 2024. Practically, you likely need Rust 1.81+ (exact MSRV still provisional until CI enforces). If you rely on an older toolchain, please file an issue.

## Quick start example

```rust
use std::collections::BTreeSet;
use yew::prelude::*;
use more_yew_hooks::prelude::*; // (Consider adding a prelude re-export if desired)

#[function_component(App)]
fn app() -> Html {
    let planets = more_yew_hooks::use_btree_set(BTreeSet::from([
        "Mercury", "Venus", "Earth"
    ]));

    let online = more_yew_hooks::use_online();

    html! {
        <div>
            <p>{ format!("Online: {}", *online) }</p>
            <p>{ format!("Planets: {:?}", planets.current().iter().collect::<Vec<_>>()) }</p>
        </div>
    }
}
```

---
## Hook Catalogue

### `use_btree_set`
Tracks a `BTreeSet<T>` with convenient mutation helpers that automatically trigger rerenders. Order is preserved (in‑order traversal). Inspired by `yew_hooks::use_set` but using an ordered structure and exposing additional operations.

**Signature**:
```rust
fn use_btree_set<T: 'static + Eq + Ord + Hash>(initial: BTreeSet<T>) -> UseBTreeSetHandle<T>
```
**Handle methods**:
- `current() -> Ref<BTreeSet<T>>` (borrow view)
- `set(BTreeSet<T>)`
- `insert(T) -> bool`
- `replace(T) -> Option<T>` (returns replaced value)
- `remove(&T) -> bool`
- `retain(F)`
- `clear()`

**Example**: See in-source docs for a full interactive example.

**Edge cases**:
- Panics if you call `current()` while holding an outstanding mutable borrow (rare in normal hook usage).
- Equality for the handle is based on inner set content, not pointer identity.

---
### `use_debounce`
Run a callback only after no further invocations occur within a wait window (`millis`). Uses `Fn()` (not `FnOnce()`).

**Signature**:
```rust
fn use_debounce<F: Fn() + 'static>(callback: F, millis: u32) -> UseDebounceHandle
```
**Handle methods**:
- `run()` — schedule (and reset) the debounce timer
- `cancel()` — cancel any pending invocation

**Behavior**:
- Multiple rapid `run()` calls coalesce into a single future callback.
- Cleaned up automatically on component unmount.

**Pitfalls**:
- Long callbacks still run on the main thread; if work is heavy, offload to a web worker or chunk with `gloo::timers::future::TimeoutFuture`.

---
### `use_debounce_state`
Debounced state updates. Only the latest value set within the debounce window is committed to component state.

**Signature**:
```rust
fn use_debounce_state<T: 'static, F: Fn() -> T>(init: F, millis: u32) -> UseDebounceStateHandle<T>
```
**Handle**:
- Derefs to `T`
- `set(T)` — stage a new value (not immediately applied)

**Workflow**:
1. Call `set(new_value)` freely while the user types or drags.
2. After `millis` of inactivity, state updates once.

**Common use cases**: Search boxes, autosave, form validation throttling.

**Notes**:
- Equality comparisons rely on the underlying `UseStateHandle<T>` semantics.
- Any staged value replaced before timeout fires is dropped.

---
### `use_local_storage_default` (feature = `storage`)
Wrapper around `localStorage` that returns `T::default()` if the key is missing or deserialization fails. Listens to `storage` events.

**Signature**:
```rust
fn use_local_storage_default<T: DeserializeOwned + Default + 'static>(key: String) -> UseLocalStorageDefaultHandle<T>
```
**Handle**:
- Derefs to `T`
- `set(T)` — serializes (JSON) & updates state
- `delete()` — removes key and resets to `T::default()`

**Notes**:
- Serialization uses `serde_json`.
- Logs (via `log`) storage updates for debugging.

**Edge cases**:
- If JSON is corrupted, returns `T::default()` instead of erroring.
- Storage quota errors currently ignored (PRs welcome for better surfacing).

---
### `use_session_storage_with_listen` (feature = `storage`)
Session storage variant that listens for `storage` events and updates only when the event references the same storage area and key.

**Signature**:
```rust
fn use_session_storage_with_listen<T: DeserializeOwned + 'static>(key: String) -> UseSessionStorageWithListenerHandle<T>
```
**Handle**:
- Derefs to `Option<T>` (None when no value present)
- `set(T)`
- `delete()`

**Notes**:
- Only updates when the event comes from the same `sessionStorage` area (validated).
- Gracefully ignores events from other storage types.

---
### `use_online`
Hook returning a `UseStateHandle<bool>` that reflects `navigator.onLine` and updates on `online` / `offline` events.

**Signature**:
```rust
fn use_online() -> UseStateHandle<bool>
```
**Usage**:
```rust
let online = use_online();
html! { <p>{ if *online { "Online" } else { "Offline" } }</p> };
```

**Caveat**: Browser `navigator.onLine` semantics vary (e.g., may report true behind captive portals). Treat as a hint, not a guarantee.

---
## Contributing

Contributions welcome! Suggested flow:

1. Open an issue describing the hook/fix.
2. Keep APIs small and composable.
3. Include docs + an example block in the hook source.
4. Follow existing style and deny warnings.
5. Add feature flags for optional dependencies.

Run `cargo fmt` + `cargo clippy --all-targets --all-features -- -D warnings` before submitting.

## Testing & Dev Tips

Because hooks depend on `wasm32-unknown-unknown` + a browser environment:

```bash
rustup target add wasm32-unknown-unknown
# Example bundler commands go here (Trunk, Leptos CLI, etc.)
```

Consider writing integration tests in a harness crate with `wasm-bindgen-test` (future work for this repo).

## License

MIT

## Acknowledgements

- Inspired by the design & breadth of [`yew-hooks`](https://crates.io/crates/yew-hooks).
- [`gloo`](https://crates.io/crates/gloo) utilities for browser interop.

[yew-hooks]: https://crates.io/crates/yew-hooks
