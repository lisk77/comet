use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use comet_app::App;
use comet_log::*;
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use crate::renderer::ErasedRenderer;
use crate::winit_module::WinitModule;

pub trait WinitAppExt {
    fn run(self, setup: fn(&mut App), update: fn(&mut App, f32));
}

impl WinitAppExt for App {
    fn run(mut self, setup: fn(&mut App), update: fn(&mut App, f32)) {
        if !self.has_module::<WinitModule>() {
            run_headless(self, setup, update);
            return;
        }

        let winit_mod = self.take_module::<WinitModule>().unwrap();
        let title = winit_mod.title.clone();
        let event_hooks = winit_mod.event_hooks;
        let update_timer = self.dt();

        info!("Starting up {}!", title);

        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(create_window(title.clone(), &winit_mod.icon, &winit_mod.size, &event_loop));

        let renderer = if let Some(factory) = winit_mod.renderer_factory {
            let (mut erased, add_handle) = factory(window.clone(), winit_mod.clear_color);
            erased.init_assets(&self);
            add_handle(&mut self);
            Some(erased)
        } else {
            None
        };

        let quit_flag = Arc::new(AtomicBool::new(false));
        let logic_thread = std::thread::Builder::new()
            .name("logic".to_string())
            .spawn({
                let quit = quit_flag.clone();
                move || run_app_loop(self, setup, update, Some(quit))
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

fn run_headless(app: App, setup: fn(&mut App), update: fn(&mut App, f32)) {
    info!("Starting up (headless)!");
    run_app_loop(app, setup, update, None);
    info!("Shutting down!");
}

fn run_app_loop(
    mut app: App,
    setup: fn(&mut App),
    update: fn(&mut App, f32),
    quit_flag: Option<Arc<AtomicBool>>,
) {
    info!("Setting up!");
    setup(&mut app);

    let mut time_stack = 0.0f32;
    let mut last_tick = std::time::Instant::now();

    loop {
        if quit_flag
            .as_ref()
            .is_some_and(|quit| quit.load(Ordering::Relaxed))
        {
            break;
        }

        app.run_tick_cycle(&mut last_tick, &mut time_stack, update);

        if app.should_quit() {
            if let Some(quit) = &quit_flag {
                quit.store(true, Ordering::Relaxed);
            }
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
                    WindowEvent::CloseRequested => {
                        quit_flag.store(true, Ordering::Relaxed);
                        elwt.exit();
                    }
                    WindowEvent::Occluded(occluded) => {
                        window_occluded = *occluded;
                    }
                    WindowEvent::Resized(size) => {
                        if let Some(r) = renderer.as_mut() { r.resize(*size); }
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        if let Some(r) = renderer.as_mut() { r.set_scale_factor(*scale_factor); }
                    }
                    WindowEvent::RedrawRequested => {
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
