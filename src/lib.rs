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
//!fn setup(app: &mut App) {}
//!fn update(app: &mut App, dt: f32) {}
//!
//!fn main() {
//!    App::with_preset(App2D)
//!        .with_title("Comet App")
//!        .with_size(1920, 1080)
//!        .run(setup, update)
//!}
//!```
//! # Subcrates
//!
//! The Comet crate is structured into several subcrates:
//!
//! | Subcrate | Description |
//! |----------|-------------|
//! | `comet_app` | Provides the core application and module system. |
//! | `comet_window` | Winit window creation, event loop, and `Renderer`/`RendererHandle` traits. |
//! | `comet_colors` | Offers a variety of color representations and utilities. |
//! | `comet_ecs` | Implements an Entity-Component-System (ECS) architecture for game development. |
//! | `comet_input` | Handles input events via `WinitInputModule`. |
//! | `comet_log` | Provides logging functionality for debugging and error reporting. |
//! | `comet_math` | Includes mathematical utilities and data structures. |
//! | `comet_renderer` | Implements a simple 2D renderer for drawing graphics and text. |
//! | `comet_assets` | Manages resources such as textures, shaders and fonts. |
pub use comet_app as app;
pub use comet_window as window;
pub use comet_colors as colors;
pub use comet_ecs as ecs;
pub use comet_input as input;
pub use comet_log as log;
pub use comet_math as math;
pub use comet_renderer as renderer;
pub use comet_assets as assets;

use comet_app::App;
use comet_assets::AssetModule;
use comet_audio::AudioModule;
use comet_ecs::EcsModule;
use comet_input::WinitInputModule;
use comet_renderer::Renderer2DModule;

pub enum Preset {
    Headless,
    App2D,
    App3D,
}

pub trait AppPresets {
    fn with_preset(preset: Preset) -> Self;
}

impl AppPresets for App {
    fn with_preset(preset: Preset) -> Self {
        let app = App::new();
        match preset {
            Preset::Headless => app
                .with_modules((AssetModule::new(), EcsModule::new())),
            Preset::App2D => app
                .with_modules((AssetModule::new(), WinitInputModule::new(), EcsModule::preset_2d(), Renderer2DModule::new(), AudioModule::new())),
            Preset::App3D => app
                .with_modules((AssetModule::new(), WinitInputModule::new(), EcsModule::preset_3d(), AudioModule::new())),
        }
    }
}

/// Everything you normally need to get started with Comet.
pub mod prelude {
    pub use comet_app::{App, Module};
    pub use comet_window::{WinitAppExt, WinitModule, WinitModuleExt, Renderer, RendererHandle};
    pub use comet_assets::*;
    pub use comet_colors::{
        sRgba, Color as CometColor, Hsla, Hsva, Hwba, Laba, Lcha, LinearRgba, Oklaba, Oklcha, Xyza,
    };
    pub use comet_ecs::{EcsModule, EcsModuleExt, *};
    pub use comet_input::{keyboard::Key, WinitInputModule, WinitInputModuleExt};
    pub use comet_log::*;
    pub use comet_math::*;
    pub use comet_renderer::{
        renderer2d::{RenderHandle2D, RenderHandle2DExt, Renderer2D},
        Renderer2DModule,
    };
    pub use comet_audio::{AudioModule, AudioModuleExt, KiraAudio};
    pub use crate::{AppPresets, Preset::*};
}
