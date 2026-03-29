use comet_app::{App, Module};
use comet_colors::{Color, LinearRgba};
use comet_log::*;
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::window::Icon;

pub struct WinitModule {
    pub(crate) title: String,
    pub(crate) icon: Option<Icon>,
    pub(crate) size: Option<LogicalSize<u32>>,
    pub(crate) clear_color: Option<LinearRgba>,
    pub(crate) event_hooks: Vec<Box<dyn Fn(&Event<()>) + Send + Sync>>,
}

impl WinitModule {
    pub fn new() -> Self {
        Self {
            title: "Untitled".to_string(),
            icon: None,
            size: None,
            clear_color: None,
            event_hooks: Vec::new(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_icon(mut self, path: impl AsRef<str>) -> Self {
        self.icon = Self::load_icon(&comet_app::resolve_asset_path(path));
        self
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = Some(LogicalSize::new(width, height));
        self
    }

    pub fn with_clear_color(mut self, color: impl Color) -> Self {
        self.clear_color = Some(color.to_linear());
        self
    }

    pub fn add_event_hook(&mut self, hook: impl Fn(&Event<()>) + Send + Sync + 'static) {
        self.event_hooks.push(Box::new(hook));
    }

    fn load_icon(path: &std::path::Path) -> Option<Icon> {
        let image = match image::open(path) {
            Ok(img) => img,
            Err(_) => {
                error!("Failed loading icon {}", path.display());
                return None;
            }
        };
        let rgba = image.to_rgba8();
        let (w, h) = rgba.dimensions();
        Some(Icon::from_rgba(rgba.into_raw(), w, h).unwrap())
    }
}

impl Module for WinitModule {
    fn build(&mut self, _app: &mut App) {}
}
