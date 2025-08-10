#![doc(
    html_logo_url = "https://raw.githubusercontent.com/lisk77/comet/refs/heads/main/res/textures/comet-1024.png",
    html_favicon_url = "https://raw.githubusercontent.com/lisk77/comet/refs/heads/main/res/textures/comet-1024.png"
)]

//! Comet is an open source game engine which provides a modular interface to customize and extend its functionality.
//!
//! # Simple example
//!
//! ```rust
//!// main.rs example
//!use comet::prelude::*;
//!
//!struct GameState {}
//!
//!impl GameState {
//!    pub fn new() -> Self {
//!      Self {}
//!    }
//!}
//!
//!// This function will be called once before the event loop starts
//!fn setup(app: &mut App, renderer: &mut Renderer2D) {}
//!// This function will be called every tick
//!fn update(app: &mut App, renderer: &mut Renderer2D, dt: f32) {}
//!
//!fn main() {
//!    App::new() // Generate a basic 2D app
//!        .with_preset(App2D) // Pre-registers the `Transform2D` component in the scene
//!        .with_title("Comet App") // Sets the window title
//!        .with_icon(r"res/textures/comet_icon.png") // Sets the window icon
//!        .with_size(1920, 1080) // Sets the window size
//!        .with_game_state(GameState::new()) // Adds a custom game state struct
//!        .run::<Renderer2D>(setup, update) // Starts app with the given renderer and functions
//!}
//!```
//! # Subcrates
//!
//! The Comet crate is structured into several subcrates:
//!
//! | Subcrate | Description |
//! |----------|-------------|
//! | `comet_app` | Provides the core functionality for creating and managing applications. |
//! | `comet_colors` | Offers a variety of color representations and utilities. |
//! | `comet_ecs` | Implements an Entity-Component-System (ECS) architecture for game development. |
//! | `comet_input` | Handles input events and provides utilities for keyboard and mouse input (as well as gamepad input in the future). |
//! | `comet_log` | Provides logging functionality for debugging and error reporting. |
//! | `comet_math` | Includes mathematical utilities and data structures like vectors, matrices, and quaternions. |
//! | `comet_renderer` | (right now) implements a simple 2D renderer for drawing graphics and text. |
//! | `comet_resources` | Manages resources such as textures, shaders and fonts. |
//!
pub use comet_app as app;
pub use comet_colors as colors;
pub use comet_ecs as ecs;
pub use comet_input as input;
pub use comet_log as log;
pub use comet_math as math;
pub use comet_renderer as renderer;
pub use comet_resources as resources;

/// Everything you normally need to get started with Comet.
pub mod prelude {
    pub use comet_app::App;
    pub use comet_app::ApplicationType::App2D;
    pub use comet_colors::{
        sRgba, Color as CometColor, Hsla, Hsva, Hwba, Laba, Lcha, LinearRgba, Oklaba, Oklcha, Xyza,
    };
    pub use comet_ecs::*;
    pub use comet_input::keyboard::Key;
    pub use comet_log::*;
    pub use comet_math::*;
    pub use comet_renderer::renderer2d::Renderer2D;
    pub use winit_input_helper::WinitInputHelper as InputManager;
}
