//! Build-time shader registration support. Runtime crates do not depend on this crate.

mod registry;

pub use registry::{Effect, Mapping, Registry, Shader};
