# Goals of the Comet Game Engine

Comet should be an unopinionated game engine built in Rust that tries to
combine the simplicity of Raylib and modularity of Bevy without the user
needing to become a follower of a cult.

The engine itself should be expandable and swappable in its components.
Don't like the standard 2D renderer? Just make your own, implement the
`Renderer` trait and use it instead of the provided one.

If you really don't want to work with the ECS, just ignore it and add
your own custom `GameState` (or whatever you want to call it) struct
and work on it using the tools provided to you by the engine.

These things should be provided for an official 1.0 version of Comet:

- [x] 2D rendering
- [ ] 3D rendering
- [ ] UI system
- [ ] particle system
- [x] ECS
- [x] sound system
- [ ] simple physics engine
- [ ] multiple scenes (aka serialization and deserialization)
- [ ] extensive documentation

Future endeavors might include:

- [ ] project creation tool
- [ ] editor
- [ ] scripting using Rhai
