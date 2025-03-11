use comet::prelude::*;

// This function will only be called once before the event loop starts.
fn setup(app: &mut App, renderer: &mut Renderer2D) {}

// This function will be called on every tick after the event loop starts.
fn update(app: &mut App, renderer: &mut Renderer2D, dt: f32) {}

fn main() {
	// This creates a window with the title "Hello world".
	// Note: You can call your functions differently if you want to though it is advised to use
	// `setup` and `update` as the names.
	// You can also replace `Renderer2D` with any other struct that implements the `Renderer` trait.
	App::new()
		.with_title("Hello world")
		.run::<Renderer2D>(setup, update);
}