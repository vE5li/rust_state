use std::hash::Hash;
use std::marker::PhantomData;

use crate::{Path, Selector};

/// An item inside a vector accessible through a [`Path`].
pub trait VecItem {
    /// The unique Id of the item. To make sure that [`Path`]s stay valid
    /// between updates of the state, this Id should be unique for each
    /// entry and not be re-used.
    type Id: Copy + PartialEq + Eq + Hash;

    /// Get the unique id of this entry.
    fn get_id(&self) -> Self::Id;
}

/// A path for doing a dynamic lookup into a [`Vec`] of [`VecItem`]s.
///
/// This type is not accessible outside this module, instead
/// [`VecLookupExt`] can be used to construct it and receive a `impl
/// Path<State, Item>`.
struct VecLookup<State, VectorPath, Item, const SAFE: bool>
where
    Item: VecItem,
{
    vector_path: VectorPath,
    id: Item::Id,
    _marker: PhantomData<State>,
}

impl<State, VectorPath, Item, const SAFE: bool> Clone for VecLookup<State, VectorPath, Item, SAFE>
where
    State: 'static,
    VectorPath: Path<State, Vec<Item>, SAFE>,
    Item: VecItem + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<State, VectorPath, Item, const SAFE: bool> Copy for VecLookup<State, VectorPath, Item, SAFE>
where
    State: 'static,
    VectorPath: Path<State, Vec<Item>, SAFE>,
    Item: VecItem + 'static,
{
}

impl<State, VectorPath, Item, const SAFE: bool> Selector<State, Item, false> for VecLookup<State, VectorPath, Item, SAFE>
where
    State: 'static,
    VectorPath: Path<State, Vec<Item>, SAFE>,
    Item: VecItem + 'static,
{
    fn select<'a>(&'a self, state: &'a State) -> Option<&'a Item> {
        self.vector_path.follow(state)?.iter().find(|e| e.get_id() == self.id)
    }
}

impl<State, VectorPath, Item, const SAFE: bool> Path<State, Item, false> for VecLookup<State, VectorPath, Item, SAFE>
where
    State: 'static,
    VectorPath: Path<State, Vec<Item>, SAFE>,
    Item: VecItem + 'static,
{
    fn follow<'a>(&self, state: &'a State) -> Option<&'a Item> {
        self.vector_path.follow(state)?.iter().find(|e| e.get_id() == self.id)
    }

    fn follow_mut<'a>(&self, state: &'a mut State) -> Option<&'a mut Item> {
        self.vector_path.follow_mut(state)?.iter_mut().find(|e| e.get_id() == self.id)
    }
}

pub trait VecLookupExt<State, T, Item, const SAFE: bool>
where
    State: 'static,
    Self: Path<State, Vec<Item>, SAFE>,
    Item: VecItem + 'static,
{
    fn lookup(self, id: Item::Id) -> impl Path<State, Item, false> {
        VecLookup {
            vector_path: self,
            id,
            _marker: PhantomData,
        }
    }
}

impl<State, T, Item, const SAFE: bool> VecLookupExt<State, T, Item, SAFE> for T
where
    State: 'static,
    T: Path<State, Vec<Item>, SAFE>,
    Item: VecItem + 'static,
{
}
