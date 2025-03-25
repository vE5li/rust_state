//! Module providing the [`Context`], which is the base type for state
//! management.

use std::cell::UnsafeCell;
use std::collections::HashMap;

use crate::{MapItem, VecItem};

/// Marker trait for the root of the state.
///
/// This is only used when creating a new [`Context`].
pub trait StateMarker {}

type StateChange<State> = Box<dyn FnOnce(&mut State)>;

/// A wrapper around the root state. Can be read and mutated using
/// [`Path`](crate::Path)s.
///
/// The context allows you to work on data in the state and even keep
/// immutable references to it while simultaneously queuing state
/// changes on that same data.
pub struct Context<State> {
    state: State,
    state_changes: UnsafeCell<Vec<StateChange<State>>>,
}

impl<State: StateMarker> Context<State> {
    /// Create a new context for a root state.
    ///
    /// A context can only be created for the root state.
    ///
    /// You can mark the root of your state using the derive macro:
    ///
    /// ```
    /// use rust_state::{Context, RustState};
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState;
    ///
    /// let _ = Context::new(MyState);
    /// ```
    ///
    /// Trying to create a context for anything other than the root of the state
    /// will fail to compile.
    ///
    /// ```compile_fail
    /// use rust_state::{Context, RustState};
    ///
    /// #[derive(RustState)]
    /// struct MyState;
    ///
    /// let _ = Context::new(MyState);
    /// ```
    pub fn new(state: State) -> Self {
        Self {
            state,
            state_changes: UnsafeCell::new(Vec::new()),
        }
    }
}

impl<State> Context<State> {
    fn push_change(&self, state_change: StateChange<State>) {
        let state_changes = UnsafeCell::raw_get(&self.state_changes as *const UnsafeCell<Vec<StateChange<State>>>);
        let state_changes = unsafe { &mut *state_changes };
        state_changes.push(state_change);
    }

    /// Update the value for a given path.
    ///
    /// Example:
    /// ```
    /// use rust_state::{Context, RustState};
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     value: u32,
    /// }
    ///
    /// let mut context = Context::new(MyState { value: 5 });
    /// let value_path = MyState::path().value();
    ///
    /// context.update_value(value_path, 10);
    /// context.apply();
    ///
    /// assert_eq!(context.get(&value_path), &10);
    /// ```
    pub fn update_value<Path, Value>(&self, path: Path, value: Value)
    where
        Path: crate::Path<State, Value>,
        Value: 'static,
    {
        self.push_change(Box::new(move |state: &mut State| match path.follow_mut(state) {
            Some(reference) => *reference = value,
            None => println!("Failed to update state"),
        }));
    }

    /// Update the value for a given path with a closure.
    ///
    /// Example:
    /// ```
    /// use rust_state::{Context, RustState};
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     value: u32,
    /// }
    ///
    /// let mut context = Context::new(MyState { value: 5 });
    /// let value_path = MyState::path().value();
    ///
    /// context.update_value_with(value_path, |value| *value *= 2);
    /// context.apply();
    ///
    /// assert_eq!(context.get(&value_path), &10);
    /// ```
    pub fn update_value_with<Path, Value, F>(&self, path: Path, closure: F)
    where
        Path: crate::Path<State, Value>,
        F: Fn(&mut Value) + 'static,
    {
        self.push_change(Box::new(move |state: &mut State| match path.follow_mut(state) {
            Some(reference) => closure(reference),
            None => println!("Failed to update state"),
        }));
    }

    /// Push an item to a [`Vec`].
    ///
    /// Example:
    /// ```
    /// use rust_state::{Context, RustState, VecItem};
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
    /// struct MyState {
    ///     items: Vec<TestItem>,
    /// }
    ///
    /// let mut context = Context::new(MyState { items: Vec::new() });
    /// let items_path = MyState::path().items();
    ///
    /// context.vec_push(items_path, TestItem { id: 10 });
    /// context.apply();
    ///
    /// assert_eq!(context.get(&items_path).len(), 1);
    /// ```
    pub fn vec_push<Path, Value>(&self, path: Path, value: Value)
    where
        Path: crate::Path<State, Vec<Value>>,
        Value: 'static,
    {
        self.push_change(Box::new(move |state: &mut State| match path.follow_mut(state) {
            Some(reference) => reference.push(value),
            None => println!("Failed to update state"),
        }));
    }

    /// Remove an item from a [`Vec`].
    ///
    /// Example:
    /// ```
    /// use rust_state::{Context, RustState, VecItem};
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
    /// struct MyState {
    ///     items: Vec<TestItem>,
    /// }
    ///
    /// let mut context = Context::new(MyState { items: vec![TestItem { id: 10 }] });
    /// let items_path = MyState::path().items();
    ///
    /// context.vec_remove(items_path, 10);
    /// context.apply();
    ///
    /// assert_eq!(context.get(&items_path).len(), 0);
    /// ```
    pub fn vec_remove<Path, Value>(&self, path: Path, id: Value::Id)
    where
        Path: crate::Path<State, Vec<Value>>,
        Value: VecItem + 'static,
    {
        self.push_change(Box::new(move |state: &mut State| match path.follow_mut(state) {
            Some(reference) => reference.retain(|item| item.get_id() != id),
            None => println!("Failed to update state"),
        }));
    }

    /// Insert an item into a [`HashMap`].
    ///
    /// Example:
    /// ```
    /// use std::collections::HashMap;
    /// use rust_state::{Context, RustState, MapItem};
    ///
    /// struct TestItem;
    //
    /// impl MapItem for TestItem {
    ///     type Id = u32;
    /// }
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     items: HashMap<u32, TestItem>,
    /// }
    ///
    /// let mut context = Context::new(MyState { items: HashMap::new() });
    /// let items_path = MyState::path().items();
    ///
    /// context.map_insert(items_path, 10, TestItem);
    /// context.apply();
    ///
    /// assert_eq!(context.get(&items_path).len(), 1);
    /// ```
    pub fn map_insert<Path, Value>(&self, path: Path, id: Value::Id, value: Value)
    where
        Path: crate::Path<State, HashMap<Value::Id, Value>>,
        Value: MapItem + 'static,
    {
        self.push_change(Box::new(move |state: &mut State| match path.follow_mut(state) {
            Some(reference) => {
                reference.insert(id, value);
            }
            None => println!("Failed to update state"),
        }));
    }

    /// Remove an item from a [`HashMap`].
    ///
    /// Example:
    /// ```
    /// use std::collections::HashMap;
    /// use rust_state::{Context, RustState, MapItem};
    ///
    /// struct TestItem;
    //
    /// impl MapItem for TestItem {
    ///     type Id = u32;
    /// }
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     items: HashMap<u32, TestItem>,
    /// }
    ///
    /// let mut context = Context::new(MyState { items: HashMap::from([(10,
    /// TestItem)]) }); let items_path = MyState::path().items();
    ///
    /// context.map_remove(items_path, 10);
    /// context.apply();
    ///
    /// assert_eq!(context.get(&items_path).len(), 0);
    /// ```
    pub fn map_remove<Path, Value>(&self, path: Path, id: Value::Id)
    where
        Path: crate::Path<State, HashMap<Value::Id, Value>>,
        Value: MapItem + 'static,
    {
        self.push_change(Box::new(move |state: &mut State| match path.follow_mut(state) {
            Some(reference) => {
                reference.remove(&id);
            }
            None => println!("Failed to update state"),
        }));
    }

    /// Apply any pending changes.
    ///
    /// Example:
    /// ```
    /// use rust_state::{Context, RustState};
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     value: &'static str,
    /// }
    ///
    /// let mut context = Context::new(MyState { value: "Before" });
    /// let value_path = MyState::path().value();
    ///
    /// context.update_value(value_path, "After");
    ///
    /// assert_eq!(context.get(&value_path), &"Before");
    ///
    /// context.apply();
    ///
    /// assert_eq!(context.get(&value_path), &"After");
    /// ```
    pub fn apply(&mut self) {
        UnsafeCell::get_mut(&mut self.state_changes)
            .drain(..)
            .for_each(|apply| apply(&mut self.state));
    }

    /// Get the output of a safe selector.
    ///
    /// # Panics
    ///
    /// This function will panic if the safe selector was unable to read the
    /// value. This might happen if
    /// [`ManuallyAssertExt`](crate::ManuallyAssertExt) is used incorrectly
    /// or the [`Selector`](crate::Selector) trait is implemented incorrectly.
    pub fn get<'a, Path, Output>(&'a self, path: &'a Path) -> &'a Output
    where
        Path: crate::Selector<State, Output>,
    {
        path.select(&self.state).unwrap()
    }

    /// Try to get the output of an unsafe selector.
    ///
    /// This is deliberately only implement for unsafe selectors and not for
    /// all selectors to encourage using the safe [`get`](Self::get)
    /// if possible.
    ///
    /// For a version of this function that accepts any selector see
    /// [`try_get_any`](Self::try_get_any).
    pub fn try_get<'a, Path, Output>(&'a self, path: &'a Path) -> Option<&'a Output>
    where
        Path: crate::Selector<State, Output, false>,
    {
        path.select(&self.state)
    }

    /// Try to get the output of any (safe or unsafe) selector.
    ///
    /// Use of this function is discouraged unless `SAFE` is not known in
    /// the current scope.
    ///
    /// Please use [`get`](Self::get) and [`try_get`](Self::try_get)
    /// otherwise.
    pub fn try_get_any<Path, Output, const SAFE: bool>(&self, path: Path) -> Option<&'_ Output>
    where
        Path: crate::Path<State, Output, SAFE>,
    {
        path.follow(&self.state)
    }
}
