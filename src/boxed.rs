//! Module providing a trait and implementation to dereference [`Box<T>`] paths in the state.
//!
//! This allows paths that resolve to a `Box<T>` to be treated as if they point directly to `T`.
//!
//! Example:
//! ```
//! use rust_state::{BoxedExt, Context, Path, RustState};
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct TestItem {
//!     value: u32,
//! }
//!
//! #[derive(RustState)]
//! #[state_root]
//! struct State {
//!     inner: Box<TestItem>,
//! }
//!
//! let context = Context::new(State {
//!     inner: Box::new(TestItem { value: 42 }),
//! });
//!
//! let path = State::path().inner().unboxed();
//!
//! assert_eq!(context.get(&path), TestItem { value : 42 });
//! ```

use std::marker::PhantomData;

use crate::{Path, Selector};

/// A path that dereferences a [`Box<T>`] in the state into a path of `T`.
///
/// This type is not accessible outside this module. Instead,
/// [`BoxedExt`] is used to construct it and receive an `impl Path<State, T>`.
struct Unboxed<State, BoxPath, Inner, const SAFE: bool> {
    box_path: BoxPath,
    _marker: PhantomData<(State, Inner)>,
}

impl<State, BoxPath, Inner, const SAFE: bool> Clone for Unboxed<State, BoxPath, Inner, SAFE>
where
    State: 'static,
    BoxPath: Path<State, Box<Inner>, SAFE>,
    Inner: 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<State, BoxPath, Inner, const SAFE: bool> Copy for Unboxed<State, BoxPath, Inner, SAFE>
where
    State: 'static,
    BoxPath: Path<State, Box<Inner>, SAFE>,
    Inner: 'static,
{
}

impl<State, BoxPath, Inner, const SAFE: bool> Selector<State, Inner, SAFE> for Unboxed<State, BoxPath, Inner, SAFE>
where
    State: 'static,
    BoxPath: Path<State, Box<Inner>, SAFE>,
    Inner: 'static,
{
    fn select<'a>(&'a self, state: &'a State) -> Option<&'a Inner> {
        self.box_path.follow(state).map(|inner| &**inner)
    }
}

impl<State, BoxPath, Inner, const SAFE: bool> Path<State, Inner, SAFE> for Unboxed<State, BoxPath, Inner, SAFE>
where
    State: 'static,
    BoxPath: Path<State, Box<Inner>, SAFE>,
    Inner: 'static,
{
    fn follow<'a>(&self, state: &'a State) -> Option<&'a Inner> {
        self.box_path.follow(state).map(|inner| &**inner)
    }

    fn follow_mut<'a>(&self, state: &'a mut State) -> Option<&'a mut Inner> {
        self.box_path.follow_mut(state).map(|inner| &mut **inner)
    }
}

pub trait BoxedExt<State, T, Inner, const SAFE: bool>
where
    State: 'static,
    Self: Path<State, Box<Inner>, SAFE>,
    Inner: 'static,
{
    fn unboxed(self) -> impl Path<State, Inner, SAFE> {
        Unboxed {
            box_path: self,
            _marker: PhantomData,
        }
    }
}

impl<State, T, Inner, const SAFE: bool> BoxedExt<State, T, Inner, SAFE> for T
where
    State: 'static,
    T: Path<State, Box<Inner>, SAFE>,
    Inner: 'static,
{
}
