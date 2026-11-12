//! Build-time shader registration support. Runtime crates do not depend on this crate.

mod gfx;
mod registry;

pub use gfx::generate_gfx;
pub use registry::{Effect, Mapping, Registry, Shader};
