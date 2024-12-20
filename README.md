# Roast2D

[![Crates.io](https://img.shields.io/crates/v/roast2d.svg)](https://crates.io/crates/roast2d)
[![Docs](https://docs.rs/roast2d/badge.svg)](https://docs.rs/roast2d/latest/roast2d/)
[![CI](https://github.com/jjyr/roast2d/workflows/CI/badge.svg)](https://github.com/jjyr/roast2d/actions)

Roast2D is a rapid development 2D game engine written in Rust.

I wrote an article explaining [why Roast2D was made](https://jjydev.org/roast-2d)

## Features

- [Poor man's ECS][poor-man-ecs], no archetype, just a little bit ECS to improve the composition ability
- Simple physics and collision
- [LDTK][LDTK] editor integration
- Multi-platform (via SDL2 and WebAssembly)


## Examples

* A copy of the classic [breakout][breakout] shows the basic usage 
* A 2D platformer prototype [balloon game][balloon] shows how to integrate with LDTK, kira (audio) and support web platform.

## Usage

Run example:

``` bash
cargo run -p example-breakout
```

Add `roast2d` to Rust project:
 
``` bash
cargo add roast2d
```

Roast2D supports multiple backends:

### SDL2

SDL2 is the default backend when you build for Linux / Mac / Windows.

Make sure the SDL2 library is installed on your machine before developing. [This document][SDL2] can help to install SDL2.

### WebAssembly

WebAssembly backend is implement with web canvas interface, you must make sure the game can build with `wasm32-unknown-unknown` target. Ensure you have [wasm-pack][wasm-pack] installed, and use `wasm-pack build` to build project. 

To render the game, ensure you provide a `<canvas>` element with the id `#roast-2d-canvas`.

## License

The source code is licensed under MIT.

[wasm-pack]: https://github.com/rustwasm/wasm-pack
[SDL2]: https://github.com/Rust-SDL2/rust-sdl2?tab=readme-ov-file#sdl20-development-libraries
[LDTK]: https://ldtk.io/
[breakout]: https://github.com/jjyr/roast2d/tree/master/examples
[balloon]: https://github.com/jjyr/balloon-game
[poor-man-ecs]: https://github.com/jjyr/roast2d/pull/14
