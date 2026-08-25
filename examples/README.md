# Examples

These examples demonstrate Comet's current public APIs.

Run an example with:

```bash
cargo run --example <example_name>
```

| Example | Description |
|---|---|
| [hello_world](hello_world.rs) | Creates a minimal Comet application with setup and update functions. |
| [textured_entity](textured_entity.rs) | Spawns a 2D camera and a textured sprite entity. |
| [simple_move_2d](simple_move_2d.rs) | Moves a tagged entity using input and a typed ECS query. |
| [simple_text](simple_text.rs) | Renders anchored screen text with text layout settings. |
| [simple_audio](simple_audio.rs) | Loads an audio asset and controls ECS-backed audio playback. |
| [input](input.rs) | Reads keyboard, mouse, and gamepad input directly. |
| [input_mapping](input_mapping.rs) | Maps typed actions to reusable input bindings. |
| [bundles](bundles.rs) | Defines and spawns a reusable named component bundle. |
| [prefabs](prefabs.rs) | Registers and spawns prefab entities. |
| [required_components](required_components.rs) | Uses `#[require(...)]` to insert required components automatically. |
| [query_change_filters](query_change_filters.rs) | Uses the typed `Added<T>` and `Changed<T>` query filters. |
| [gizmos](gizmos.rs) | Defines a component gizmo and controls its visibility. |
