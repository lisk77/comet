use super::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Timer {
    time_stack: f32,
    interval: f32,
    done: bool,
}

impl Timer {
    pub fn set_interval(&mut self, interval: f32) {
        self.interval = interval
    }

    pub fn update_timer(&mut self, elapsed_time: f32) {
        self.time_stack += elapsed_time;
        if self.time_stack > self.interval {
            self.done = true
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn reset(&mut self) {
        self.time_stack = 0.0;
        self.done = false;
    }
}
