# ☄️ Comet

a free and open source game engine

> [!WARNING]
> This project is in early development and is not yet ready for use.
>
> It could be potentially used to make something very basic in a very hacky way, but it is not a good experience.
>
> Installation is manual as of right now but if it reaches an acceptable state I will publish the crate on crates.
>
> Goals and ideas can be found in [goals](goals.md).

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

struct Score(u8);

// This function will be called once before the event loop starts
fn setup(app: &mut App) {
  app.add_context(Score(0)); // Registers a shared context for the app.
}
// This function will be called every tick
fn update(app: &mut App, dt: f32) {}

fn main() {
    App::with_preset(App2D) // Creates a new `App` and pre-registers specific components and modules
        .with_title("Comet App") // Sets the window title
        .with_icon("res://textures/comet_icon.png") // Sets the window icon
        .with_size(1920, 1080) // Sets the window size
        .run(setup, update) // Starts app with the given
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
