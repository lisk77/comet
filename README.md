# ☄️ Comet
a free and open source games framework

> [!WARNING]
> This project is in early development and is not yet ready for use.
> 
> It could be potentially used to make something very basic in a very hacky way but it is not a good experience 
> 
> Installation is manual as of right now but if it reaches an exaptable state I will publish the crate

## Recommended setup

The project structure should look like this:

```
project
│   Cargo.toml
│   build.rs
│   crates
│   └── comet
│   resources
│   └── shaders
│   └── textures
│   src
│   └── main.rs
```

```toml
# Cargo.toml
# ...
[dependencies]
comet = { path = "crates/comet" }
# ...
```

```rust
// main.rs
use comet::prelude::*;

struct GameState {}

impl GameState {
  pub fn new() -> Self {
    Self {}
  }
}

// This function will be called once before the event loop starts
fn setup(app: &mut App, renderer: &mut Renderer2D) {}
// This function will be called every tick 
fn update(app: &mut App, renderer: &mut Renderer2D, dt: f32) {}

fn main() {
  App::new() // Generate a basic 2D app
          .with_preset(App2D) // Pre-registers the `Transform2D` component in the world
          .with_title("Comet App") // Sets the window title
          .with_icon(r"resources/textures/comet_icon.png") // Sets the window icon
          .with_size(1920, 1080) // Sets the window size
          .with_game_state(GameState::new()) // Adds a custom game state struct
          .run::<Renderer2D>(setup, update) // Starts app
}
```

```rust
// build.rs example

use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;

fn main() -> Result<()> {
  println!("cargo:rerun-if-changed=resources/textures/*");
  println!("cargo:rerun-if-changed=resources/shaders/*");

  let out_dir = env::var("OUT_DIR")?;
  let mut copy_options = CopyOptions::new();
  copy_options.overwrite = true;
  let mut paths_to_copy = Vec::new();
  paths_to_copy.push("resources/textures/");
  paths_to_copy.push("resources/shaders/");

  copy_items(&paths_to_copy, out_dir, &copy_options)?;

  Ok(())
}
```

## Todo
(not ordered by importance)

- [x] Fixed update steps (60 updates per second right now)
- [ ] Rendering
  - [x] 2D
    - [x] Textures
  - [ ] 3D
    - [ ] Meshes
    - [ ] Normal maps
  - [x] Texture Atlas
  - [x] Shaders
  - [ ] Materials
  - [ ] Text
  - [ ] Particles
  - [ ] Animations
  - [ ] Lighting
  - [ ] UI
    - [ ] Buttons
    - [ ] Input
  - [ ] Post-processing
    - [ ] Multiple render passes
    - [ ] Individual effects
- [ ] Sound
- [ ] Input
  - [ ] Universal input manager
  - [x] Keyboard
  - [x] Mouse
  - [ ] Gamepad
- [x] ECS
  - [x] Components
  - [x] Entities
  - [x] Archetypes
  - [x] World
- [ ] Scene
  - [ ] loading
  - [ ] saving
- [ ] Physics
  - [ ] 2D
  - [ ] 3D
- [x] Plugin System (at least right now)
  - [x] Adding custom game state struct
  - [x] Adding custom renderer
- [ ] Math
  - [x] Vectors
  - [x] Matrices
  - [ ] Quaternions
  - [x] Interpolation
  - [ ] Bezier curves
  - [x] Easing functions
  - [ ] Noise
    - [ ] White
    - [ ] Perlin
    - [ ] Simplex
    - [ ] Worley
    - [ ] Fractal
    - [ ] Cellular
    - [ ] Gradient
    - [ ] Value
    - [ ] Cubic
  - [ ] Ray-casting
  - [ ] Pathfinding