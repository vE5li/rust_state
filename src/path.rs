//! Module providing the base mechanism for indexing state, namely [`Path`] and
//! [`Selector`].

/// A `Selector` can be used to get an item from the state or itself.
///
/// `Selector<State, T>` is implemented for `T`, so any value is also a
/// `Selector` to itself.
///
/// An example of how this can be used:
///
/// ```rust
/// use rust_state::{Context, RustState, Selector};
///
/// #[derive(Default, RustState)]
/// #[state_root]
/// struct GlobalState {
///     number: u32,
/// }
///
/// struct Uses<S>(S);
///
/// impl<S: Selector<GlobalState, u32>> Uses<S> {
///     pub fn new(selector: S) -> Self {
///         Self(selector)
///     }
///
///     pub fn do_work(&self, state: &Context<GlobalState>) {
///         let _: &u32 = state.get(&self.0);
///     }
/// }
///
/// let state = Context::new(GlobalState::default());
///
/// let uses_0 = Uses::new(GlobalState::path().number());
/// let uses_1 = Uses::new(1u32);
///
/// uses_0.do_work(&state);
/// uses_1.do_work(&state);
/// ```
///
/// As can be seen in the example above, the main purpose of [`Selector`] is to
/// abstract over `T` and [`Path`]s that follow to a `T`.
pub trait Selector<State, To: ?Sized, const SAFE: bool = true>: 'static {
    fn select<'a>(&'a self, state: &'a State) -> Option<&'a To>;
}

// Blanket implementation so that any `T` is a `Selector` for `T`.
impl<State, T: 'static> Selector<State, T> for T {
    fn select<'a>(&'a self, _: &'a State) -> Option<&'a T> {
        Some(self)
    }
}

/// A `Path` can be followed to get a mutable or immutable reference to
/// arbitrary data from the state.
///
/// Paths are forced to be [`Copy`] so they are easier to pass around and
/// duplicate.
///
/// Additionally, every path is forced to implement [`Selector`] to improve the
/// ergonomics of the [`Context`].
///
/// The [`Path`] trait is automatically implemented when deriving
/// [`RustState`](crate::RustState).
///
/// Example:
///```
/// use rust_state::{Context, RustState, Path};
///
/// #[derive(Default, RustState)]
/// #[state_root]
/// struct GlobalState {
///     number: u32,
/// }
///
/// fn takes_path<T>(_: impl Path<GlobalState, T>) {}
///
/// takes_path::<GlobalState>(GlobalState::path());
/// takes_path::<u32>(GlobalState::path().number());
/// ```
pub trait Path<State, To: ?Sized, const SAFE: bool = true>: Selector<State, To, SAFE> + Copy {
    /// Follow the path and try to return a reference to its target.
    fn follow<'a>(&self, state: &'a State) -> Option<&'a To>;

    /// Follow the path and try to return a mutable reference to its target.
    fn follow_mut<'a>(&self, state: &'a mut State) -> Option<&'a mut To>;
}
