//! Build-time shader registration support. Runtime crates do not depend on this crate.

mod app;
mod gfx;
mod registry;
mod store;
mod ui;

pub use app::generate_app;
pub use gfx::generate_gfx;
pub use registry::{Effect, Mapping, Registry, Shader};
pub use store::generate_store;
pub use ui::generate_ui;
