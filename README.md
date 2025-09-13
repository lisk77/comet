# ☄️ Comet

a free and open source game engine

> [!WARNING]
> This project is in early development and is not yet ready for use.
>
> It could be potentially used to make something very basic in a very hacky way, but it is not a good experience.
>
> Installation is manual as of right now but if it reaches an acceptable state I will publish the crate on crates.
>
> There is a plan for a project creation tool that will automate the project setup process.

## Recommended setup

The project structure as of right now should look like this:

```
project
│   Cargo.toml
│   build.rs
│   crates
│   └── comet
│   res
│   └── shaders
│   └── textures
│   └── sounds
│   src
│   └── main.rs
```

```toml
# Cargo.toml
# ...
[dependencies]
comet = { path = "path/of/the/comet/crate" }
# ...
```

```rust
// main.rs example
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
        .with_preset(App2D) // Pre-registers the `Transform2D` component in the scene
        .with_title("Comet App") // Sets the window title
        .with_icon(r"res/textures/comet_icon.png") // Sets the window icon
        .with_size(1920, 1080) // Sets the window size
        .with_game_state(GameState::new()) // Adds a custom game state struct
        .run::<Renderer2D>(setup, update) // Starts app with the given
}
```

```rust
// build.rs example

use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
  // Watch resource directories for changes
  println!("cargo:rerun-if-changed=res/materials/*");
  println!("cargo:rerun-if-changed=res/objects/*");
  println!("cargo:rerun-if-changed=res/textures/*");
  println!("cargo:rerun-if-changed=res/shaders/*");
  println!("cargo:rerun-if-changed=res/data/*");
  println!("cargo:rerun-if-changed=res/sounds/*");
  println!("cargo:rerun-if-changed=res/fonts/*");

  let profile = env::var("PROFILE")?;
  let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
  let target_dir = manifest_dir.join("target").join(&profile);

  let dest_resources_dir = target_dir.join("res");

  std::fs::create_dir_all(&dest_resources_dir)?;

  let mut copy_options = CopyOptions::new();
  copy_options.overwrite = true;
  copy_options.copy_inside = true;

  let resource_folders = vec![
    "res/materials/",
    "res/objects/",
    "res/textures/",
    "res/shaders/",
    "res/data/",
    "res/sounds/",
    "res/fonts/",
  ];

  let resource_paths: Vec<PathBuf> = resource_folders
          .iter()
          .map(|p| manifest_dir.join(p))
          .collect();

  copy_items(&resource_paths, &dest_resources_dir, &copy_options)?;

  Ok(())
}

```
