#![feature(auto_traits)]
#![feature(negative_impls)]

// Reexport self as `rust_state` so that the derive macro works in this crate.
extern crate self as rust_state;

mod array;
mod boxed;
mod context;
mod downcast;
mod manual;
mod map;
mod option;
mod path;
mod vec;

pub use array::ArrayLookupExt;
pub use boxed::BoxedExt;
pub use context::{Context, StateMarker};
pub use downcast::DowncastExt;
pub use macros::RustState;
pub use manual::ManuallyAssertExt;
pub use map::{MapItem, MapLookupExt};
pub use option::OptionExt;
pub use path::{AutoImplSelector, Path, Selector};
pub use vec::{VecIndexExt, VecItem, VecLookupExt};
