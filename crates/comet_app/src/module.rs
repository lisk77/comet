use crate::App;
use std::any::Any;

pub trait Module: Any + Send + 'static {
    fn dependencies(_app: &mut App)
    where
        Self: Sized,
    {
    }
    fn build(&mut self, app: &mut App);
}
