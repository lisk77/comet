pub trait Audio {
    fn new() -> Self
    where
        Self: Sized;
    fn load(&mut self, name: &str, path: &str);
    fn play(&mut self, name: &str, looped: bool);
    fn pause(&mut self, name: &str);
    fn stop(&mut self, name: &str);
    fn stop_all(&mut self);
    fn update(&mut self, dt: f32);
    fn is_playing(&self, name: &str) -> bool;
    fn set_volume(&mut self, name: &str, volume: f32);
}
