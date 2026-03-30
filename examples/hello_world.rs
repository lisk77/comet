use comet::prelude::*;

// This function will only be called once before the event loop starts.
fn setup(_app: &mut App) {}

// This function will be called on every tick after the event loop starts.
fn update(_app: &mut App, _dt: f32) {}

fn main() {
    // This creates a window with the title "Hello world".
    // Note: You can call your functions differently if you want to though it is advised to use
    // `setup` and `update` as the names.
    // Instead of using new, you can also use a preset which will add some default modules.
    // For custom configurations, just use `with_module` and add the modules you want.
    App::new()
        .with_title("Hello world")
        .run(setup, update);
}
