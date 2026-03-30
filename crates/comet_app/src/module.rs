use std::any::Any;
use crate::App;

pub trait Module: Any + Send + 'static {
    /// Declare modules this module depends on. Called before `build`.
    /// Register any missing dependencies on `app` here.
    fn dependencies(_app: &mut App) where Self: Sized {}
    fn build(&mut self, app: &mut App);
}
