# Roast2D

Roast2D is a rapid development 2D game engine written in Rust. Inspired by [high_impact][high_impact].

I wrote an article explaining [why Roast2D was made](https://jjydev.org/roast-2d)

## Features

- Simple entity object with trait extention instead of ECS or hierarchy model
- Built-in integration with [LDTK][LDTK] level editor
- Multi-platform support (via SDL2 and WebAssembly)


## Examples

* An mini [ping-pong game][demo] shows the basic usage 
* A 2D platformer prototype [balloon game][balloon] shows how to integrate with LDTK, kira (audio) and support web platform.

## Usage

Run example:

``` bash
cargo run --example demo
```

Add `roast-2d` to Rust project:
 
``` bash
cargo add roast-2d
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
[high_impact]: https://github.com/phoboslab/high_impact
[demo]: https://github.com/jjyr/roast2d/tree/master/examples
[balloon]: https://github.com/jjyr/balloon-game
