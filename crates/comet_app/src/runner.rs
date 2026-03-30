use crate::App;
use comet_log::*;

pub trait Runner: Send + 'static {
    fn run(self: Box<Self>, app: App, setup: fn(&mut App), update: fn(&mut App, f32));
}

pub struct HeadlessRunner;

impl Runner for HeadlessRunner {
    fn run(self: Box<Self>, mut app: App, setup: fn(&mut App), update: fn(&mut App, f32)) {
        info!("Starting up (headless)!");
        setup(&mut app);

        let mut time_stack = 0.0f32;
        let mut last_tick = std::time::Instant::now();

        loop {
            app.run_tick_cycle(&mut last_tick, &mut time_stack, update);
            if app.should_quit() { break; }
            sleep_until_next_tick(app.dt(), last_tick);
        }

        info!("Shutting down!");
    }
}

pub fn sleep_until_next_tick(update_timer: f32, last_tick: std::time::Instant) {
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
