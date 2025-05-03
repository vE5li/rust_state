#![feature(auto_traits)]
#![feature(negative_impls)]

pub use array::ArrayLookupExt;
pub use context::{Context, StateMarker};
pub use macros::RustState;
pub use manual::ManuallyAssertExt;
pub use map::{MapItem, MapLookupExt};
// pub use option::OptionExt;
pub use path::{AutoImplSelector, Path, Selector};
pub use vec::{VecIndexExt, VecItem, VecLookupExt};

// Reexport self as `rust_state` so that the derive macro works in this crate.
extern crate self as rust_state;

mod array;
mod context;
mod manual;
mod map;
// mod option;
mod path;
mod vec;
