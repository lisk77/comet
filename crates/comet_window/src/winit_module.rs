use crate::renderer::{ErasedRenderer, RendererFactory};
use comet_app::{App, Module, Runner, runner::sleep_until_next_tick};
use comet_colors::{Color, LinearRgba};
use comet_log::*;
use comet_macros::module;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Icon, Window};

pub struct WinitModule {
    pub(crate) title: String,
    pub(crate) icon: Option<Icon>,
    pub(crate) size: Option<LogicalSize<u32>>,
    pub(crate) clear_color: Option<LinearRgba>,
    pub(crate) event_hooks: Vec<Box<dyn Fn(&Event<()>) + Send + Sync>>,
    pub(crate) renderer_factory: Option<RendererFactory>,
}

impl WinitModule {
    pub fn new() -> Self {
        Self {
            title: "Untitled".to_string(),
            icon: None,
            size: None,
            clear_color: None,
            event_hooks: Vec::new(),
            renderer_factory: None,
        }
    }
}

#[module]
impl WinitModule {
    pub fn with_title(&mut self, title: impl Into<String>) -> &mut Self {
        self.title = title.into();
        self
    }

    pub fn with_icon(&mut self, path: impl AsRef<str>) -> &mut Self {
        self.icon = Self::load_icon(&comet_app::resolve_asset_path(path));
        self
    }

    pub fn with_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.size = Some(LogicalSize::new(width, height));
        self
    }

    pub fn with_clear_color(&mut self, color: impl Color) -> &mut Self {
        self.clear_color = Some(color.to_linear());
        self
    }

    pub fn add_event_hook(&mut self, hook: impl Fn(&Event<()>) + Send + Sync + 'static) {
        self.event_hooks.push(Box::new(hook));
    }

    pub fn set_renderer_factory(&mut self, factory: RendererFactory) {
        self.renderer_factory = Some(factory);
    }
}

impl WinitModule {
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
    fn build(&mut self, app: &mut App) {
        app.set_runner(WinitRunner);
    }
}

pub struct WinitRunner;

impl Runner for WinitRunner {
    fn run(self: Box<Self>, mut app: App, setup: fn(&mut App), update: fn(&mut App, f32)) {
        let winit_mod = app.take_module::<WinitModule>().unwrap();
        let title = winit_mod.title.clone();
        let event_hooks = winit_mod.event_hooks;
        let update_timer = app.dt();

        info!("Starting up {}!", title);

        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(create_window(title.clone(), &winit_mod.icon, &winit_mod.size, &event_loop));

        let renderer = if let Some(factory) = winit_mod.renderer_factory {
            let (mut erased, add_handle) = factory(window.clone(), winit_mod.clear_color);
            erased.init_assets(&app);
            add_handle(&mut app);
            Some(erased)
        } else {
            None
        };

        let quit_flag = Arc::new(AtomicBool::new(false));
        let logic_thread = std::thread::Builder::new()
            .name("logic".to_string())
            .spawn({
                let quit = quit_flag.clone();
                move || run_app_loop(app, setup, update, quit)
            })
            .unwrap();

        info!("Starting event loop!");
        
        run_event_loop(event_loop, renderer, window, event_hooks, quit_flag, update_timer);

        logic_thread.join().ok();
        info!("Shutting down {}!", title);
    }
}

fn create_window(
    title: String,
    icon: &Option<Icon>,
    size: &Option<LogicalSize<u32>>,
    event_loop: &EventLoop<()>,
) -> Window {
    let builder = winit::window::WindowBuilder::new().with_title(title);

    let builder = if let Some(icon) = icon.clone() {
        builder.with_window_icon(Some(icon))
    } else {
        builder
    };

    let builder = if let Some(size) = *size {
        builder.with_inner_size(size)
    } else {
        builder
    };

    builder.build(event_loop).unwrap()
}

fn run_app_loop(
    mut app: App,
    setup: fn(&mut App),
    update: fn(&mut App, f32),
    quit_flag: Arc<AtomicBool>,
) {
    info!("Setting up!");
    setup(&mut app);

    let mut time_stack = 0.0f32;
    let mut last_tick = std::time::Instant::now();

    loop {
        if quit_flag.load(Ordering::Relaxed) {
            break;
        }

        app.run_tick_cycle(&mut last_tick, &mut time_stack, update);

        if app.should_quit() {
            quit_flag.store(true, Ordering::Relaxed);
            break;
        }

        sleep_until_next_tick(app.dt(), last_tick);
    }
}

fn run_event_loop(
    event_loop: EventLoop<()>,
    mut renderer: Option<Box<dyn ErasedRenderer>>,
    window: Arc<Window>,
    event_hooks: Vec<Box<dyn Fn(&Event<()>) + Send + Sync>>,
    quit_flag: Arc<AtomicBool>,
    update_timer: f32,
) {
    let mut window_occluded = false;

    event_loop
        .run(|event, elwt| {
            if quit_flag.load(Ordering::Relaxed) {
                elwt.exit();
                return;
            }

            for hook in &event_hooks {
                hook(&event);
            }

            match event {
                Event::WindowEvent { ref event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        quit_flag.store(true, Ordering::Relaxed);
                        elwt.exit();
                    }
                    winit::event::WindowEvent::Occluded(occluded) => {
                        window_occluded = *occluded;
                    }
                    winit::event::WindowEvent::Resized(size) => {
                        if let Some(r) = renderer.as_mut() { r.resize(*size); }
                    }
                    winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        if let Some(r) = renderer.as_mut() { r.set_scale_factor(*scale_factor); }
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        drain_renderer_commands(&mut renderer);
                        if let Some(r) = renderer.as_mut() {
                            if !window_occluded {
                                if handle_render(r.as_mut()) {
                                    quit_flag.store(true, Ordering::Relaxed);
                                    elwt.exit();
                                }
                            }
                        }
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    if window_occluded {
                        drain_renderer_commands(&mut renderer);
                        elwt.set_control_flow(ControlFlow::Wait);
                    } else {
                        drain_renderer_commands(&mut renderer);
                        window.request_redraw();
                        if update_timer.is_finite() {
                            let next = std::time::Instant::now()
                                + std::time::Duration::from_secs_f32(update_timer);
                            elwt.set_control_flow(ControlFlow::WaitUntil(next));
                        } else {
                            elwt.set_control_flow(ControlFlow::Wait);
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}

fn drain_renderer_commands(renderer: &mut Option<Box<dyn ErasedRenderer>>) {
    if let Some(renderer) = renderer.as_mut() {
        renderer.drain_commands();
    }
}

fn handle_render(renderer: &mut dyn ErasedRenderer) -> bool {
    match renderer.render() {
        Ok(_) => false,
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            let size = renderer.size();
            renderer.resize(size);
            false
        }
        Err(wgpu::SurfaceError::OutOfMemory) => {
            error!("Out of memory!");
            true
        }
        Err(wgpu::SurfaceError::Timeout) => {
            warn!("Surface timeout: skipping frame");
            false
        }
    }
}

