pub use winit::keyboard as keyboard;
pub use comet_math as math;
pub use comet_renderer as renderer;
pub use comet_resources as resources;
pub use comet_ecs as ecs;
pub use comet_app as app;
pub use comet_colors as colors;
pub use comet_input as input;
pub use comet_log as log;

pub mod prelude {
	pub use comet_app::App;
	pub use comet_app::ApplicationType::App2D;
	pub use comet_renderer::renderer2d::Renderer2D;
	pub use comet_input::keyboard::Key;
	pub use winit_input_helper::WinitInputHelper as InputManager;
	pub use comet_log::*;
	pub use comet_colors::{
		LinearRgba,
		sRgba,
		Hsla,
		Hsva,
		Hwba,
		Laba,
		Lcha,
		Oklaba,
		Oklcha,
		Xyza,
		Color as CometColor
	};
	pub use comet_ecs::*;
	pub use comet_math::*;
}
