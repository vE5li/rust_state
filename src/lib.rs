#![feature(auto_traits)]
#![feature(negative_impls)]

pub use context::{Context, StateMarker};
pub use macros::RustState;
pub use manual::ManuallyAssertExt;
pub use map::{MapItem, MapLookupExt};
pub use path::{AutoImplSelector, Path, Selector};
pub use vec::{VecItem, VecLookupExt};

// Reexport self as `rust_state` so that the derive macro works in this crate.
extern crate self as rust_state;

mod context;
mod manual;
mod map;
mod path;
mod vec;
