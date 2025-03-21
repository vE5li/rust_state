pub use context::{Context, StateMarker};
pub use macros::RustState;
pub use manual::ManuallyAssertExt;
pub use map::{MapItem, MapLookupExt};
pub use path::{Path, Selector};
pub use vec::{VecItem, VecLookupExt};

// Reexport self as `rust_state` so that the derive macro works in this crate.
extern crate self as rust_state;

/// Module providing the [`Context`], which is the base type for state
/// management.
mod context;

/// Module providing an extension trait to manually assert that a path is safe.
///
/// This can be useful for situations where you select an item from a vector or
/// map that you know exists.
///
/// Example:
/// ```
/// use rust_state::{Context, ManuallyAssertExt, RustState, VecItem, VecLookupExt};
///
/// struct TestItem {
///     id: u32,
/// }
///
/// impl VecItem for TestItem {
///     type Id = u32;
///
///     fn get_id(&self) -> Self::Id {
///         self.id
///     }
/// }
///
/// #[derive(RustState)]
/// #[state_root]
/// struct State {
///     items: Vec<TestItem>,
/// }
///
/// let context = Context::new(State {
///     items: vec![TestItem { id: 10 }],
/// });
///
/// // We *know* that our item exists, so we can wrap the path with `manually_asserted`.
/// let item_path = State::path().items().lookup(10).manually_asserted();
///
/// // We can use `get` since the path is safe.
/// assert_eq!(context.get(&item_path).id, 10);
/// ```
mod manual;

/// Module providing a trait and an extension trait to index a map in the state.
///
/// Example:
/// ```
/// use std::collections::HashMap;
/// use rust_state::{Context, ManuallyAssertExt, RustState, MapItem, MapLookupExt};
///
/// #[derive(Debug, PartialEq, Eq)]
/// struct TestItem;
///
/// impl MapItem for TestItem {
///     type Id = u32;
/// }
///
/// #[derive(RustState)]
/// #[state_root]
/// struct State {
///     items: HashMap<u32, TestItem>,
/// }
///
/// let context = Context::new(State {
///     items: HashMap::from([(10, TestItem)]),
/// });
///
/// let item_path = State::path().items().lookup(10);
///
/// assert_eq!(context.try_get(&item_path), Some(&TestItem));
/// ```
mod map;

/// Module providing the base mechanism for indexing state, namely [`Path`] and
/// [`Selector`].
mod path;

/// Module providing a trait and an extension trait to index a vector in the
/// state.
///
/// Example:
/// ```
/// use rust_state::{Context, ManuallyAssertExt, RustState, VecItem, VecLookupExt};
///
/// #[derive(Debug, PartialEq, Eq)]
/// struct TestItem {
///     id: u32,
/// }
///
/// impl VecItem for TestItem {
///     type Id = u32;
///
///     fn get_id(&self) -> Self::Id {
///         self.id
///     }
/// }
///
/// #[derive(RustState)]
/// #[state_root]
/// struct State {
///     items: Vec<TestItem>,
/// }
///
/// let context = Context::new(State {
///     items: vec![TestItem { id: 10 }],
/// });
///
/// let item_path = State::path().items().lookup(10);
///
/// assert_eq!(context.try_get(&item_path), Some(&TestItem { id: 10 }));
/// ```
mod vec;
