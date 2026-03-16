# Examples

This directory contains a few examples that demonstrate how you can use Comet to create an application/game.

Simply run 
```bash
cargo run --example <example_name>
```

| Example                                   | Description                                                                            |
|-------------------------------------------|----------------------------------------------------------------------------------------|
| [hello_world](hello_world.rs)             | A simple boilerplate example to show how to properly start creating a Comet App.       |
| [textured_entity](textured_entity.rs)     | This covers the basics on how to create a camera and your first entity with a texture. |
| [simple_move_2d](simple_move_2d.rs)       | A simple demonstration of a hypothetical movement system in 2D.                        |
| [simple_text](simple_text.rs)             | A simple demonstration of how to write some text in Comet.                             |
| [simple_audio](simple_audio.rs)           | A simple demonstration of how to use the audio system in Comet.                        |
| [prefabs](prefabs.rs)                     | Shows how to register and spawn prefabbed entities.
                                            |
| [bundles](bundles.rs)                     | Shows how to use a reusable entity spawning pattern.
| [query_change_filters](query_change_filters.rs) | Shows how to use the `added`/`changed` query filters.          |
