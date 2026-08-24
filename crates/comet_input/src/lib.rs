pub mod keyboard;
pub mod mouse;
pub mod input_module;

pub use comet_macros::Action;
pub use input_module::{Action, InputModule, InputModuleExt};
