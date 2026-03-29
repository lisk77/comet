use std::any::type_name;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use comet_app::App;
use comet_colors::Color;
use comet_log::*;
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use crate::renderer::{Renderer, RendererHandle};
use crate::winit_module::WinitModule;

pub trait WinitAppExt {
    fn with_title(self, title: impl Into<String>) -> Self;
    fn with_icon(self, path: impl AsRef<str>) -> Self;
    fn with_size(self, width: u32, height: u32) -> Self;
    fn with_clear_color(self, color: impl Color) -> Self;
    fn run<R: Renderer>(self, setup: fn(&mut App, &mut R::Handle), update: fn(&mut App, &mut R::Handle, f32))
    where
        R::Handle: 'static;
}

fn ensure_winit_module(app: &mut App) {
    if !app.has_module::<WinitModule>() {
        app.add_module(WinitModule::new());
    }
}

fn replace_winit_module(mut app: App, f: impl FnOnce(WinitModule) -> WinitModule) -> App {
    ensure_winit_module(&mut app);
    let m = app.take_module::<WinitModule>().unwrap();
    app.add_module(f(m));
    app
}

impl WinitAppExt for App {
    fn with_title(self, title: impl Into<String>) -> Self {
        replace_winit_module(self, |m| m.with_title(title))
    }

    fn with_icon(self, path: impl AsRef<str>) -> Self {
        replace_winit_module(self, |m| m.with_icon(path))
    }

    fn with_size(self, width: u32, height: u32) -> Self {
        replace_winit_module(self, |m| m.with_size(width, height))
    }

    fn with_clear_color(self, color: impl Color) -> Self {
        replace_winit_module(self, |m| m.with_clear_color(color))
    }

    fn run<R: Renderer>(mut self, setup: fn(&mut App, &mut R::Handle), update: fn(&mut App, &mut R::Handle, f32))
    where
        R::Handle: 'static,
    {
        let winit_mod = self
            .take_module::<WinitModule>()
            .expect("WinitModule is required to use this method");

        let title = winit_mod.title.clone();
        let event_hooks = winit_mod.event_hooks;
        let update_timer = self.dt();

        info!("Starting up {}!", title);

        let (cmd_tx, cmd_rx) = flume::unbounded();
        let (evt_tx, evt_rx) = flume::unbounded();

        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(create_window(title.clone(), &winit_mod.icon, &winit_mod.size, &event_loop));
        let mut renderer = R::new(window, winit_mod.clear_color, evt_tx);
        renderer.init_assets(&self);
        info!("Using Renderer {}", type_name::<R>());

        let quit_flag = Arc::new(AtomicBool::new(false));
        let logic_thread = std::thread::Builder::new()
            .name("logic".to_string())
            .spawn({
                let quit = quit_flag.clone();
                move || run_logic_thread::<R>(self, setup, update, quit, cmd_tx, evt_rx)
            })
            .unwrap();

        info!("Starting event loop!");
        run_event_loop::<R>(event_loop, renderer, event_hooks, quit_flag, cmd_rx, update_timer);

        logic_thread.join().ok();
        info!("Shutting down {}!", title);
    }
}

fn create_window(
    title: String,
    icon: &Option<winit::window::Icon>,
    size: &Option<winit::dpi::LogicalSize<u32>>,
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

fn run_logic_thread<R: Renderer>(
    mut app: App,
    setup: fn(&mut App, &mut R::Handle),
    update: fn(&mut App, &mut R::Handle, f32),
    quit_flag: Arc<AtomicBool>,
    cmd_tx: flume::Sender<<R::Handle as RendererHandle>::Command>,
    evt_rx: flume::Receiver<<R::Handle as RendererHandle>::Event>,
) where
    R::Handle: 'static,
{
    let mut handle = R::Handle::new(cmd_tx, evt_rx);
    info!("Setting up!");
    setup(&mut app, &mut handle);

    let mut time_stack = 0.0f32;
    let mut last_tick = std::time::Instant::now();

    while !quit_flag.load(Ordering::Relaxed) {
        app.run_tick_cycle(&mut last_tick, &mut time_stack, |a, dt| update(a, &mut handle, dt));

        if app.should_quit() {
            quit_flag.store(true, Ordering::Relaxed);
            break;
        }

        sleep_until_next_tick(app.dt(), last_tick);
    }
}

fn run_event_loop<R: Renderer>(
    event_loop: EventLoop<()>,
    mut renderer: R,
    event_hooks: Vec<Box<dyn Fn(&Event<()>) + Send + Sync>>,
    quit_flag: Arc<AtomicBool>,
    cmd_rx: flume::Receiver<<R::Handle as RendererHandle>::Command>,
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
                    WindowEvent::CloseRequested => {
                        quit_flag.store(true, Ordering::Relaxed);
                        elwt.exit();
                    }
                    WindowEvent::Occluded(occluded) => {
                        window_occluded = *occluded;
                    }
                    WindowEvent::Resized(size) => renderer.resize(*size),
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        renderer.set_scale_factor(*scale_factor);
                    }
                    WindowEvent::RedrawRequested => {
                        while let Ok(cmd) = cmd_rx.try_recv() {
                            renderer.apply_command(cmd);
                        }
                        if !window_occluded {
                            if handle_render(&mut renderer) {
                                elwt.exit();
                            }
                        }
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    if window_occluded {
                        while cmd_rx.try_recv().is_ok() {}
                        elwt.set_control_flow(ControlFlow::Wait);
                    } else {
                        while let Ok(cmd) = cmd_rx.try_recv() {
                            renderer.apply_command(cmd);
                        }
                        renderer.window().request_redraw();
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

fn handle_render<R: Renderer>(renderer: &mut R) -> bool {
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
            warn!("Surface timeout - skipping frame");
            false
        }
    }
}

fn sleep_until_next_tick(update_timer: f32, last_tick: std::time::Instant) {
    if update_timer.is_finite() && update_timer > 0.0 {
        let target = std::time::Duration::from_secs_f32(update_timer);
        let elapsed = last_tick.elapsed();
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
    } else {
        std::thread::yield_now();
    }
}
