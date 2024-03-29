# bitworks

## How to run and build things

Create assets by running `assets-generator`.

```cargo run --bin assets-generator bitworks/assets```

Run the main executable `bitworks` to run a current demo.
Probably is is starting as paused and you need to press a key to unpause.
Try it `Enter` or `Space`. Exit with `ESC`.

```cargo run --bin bitworks```

There are other executables in this repository too.
They are usually experiments.

## System overview

Sprites are created with a tool using `tiny-skia` to draw simple 2D shapes
and save them to PNG.

Bitworks is a executable (`main.rs`) and a library (`lib.rs`).
The library exports all the modules public symbols but also bundles related things together in plugins.
The executable than adds the plugins to the bevy App and a bunch of less stable systems.
The executable also implements systems, usually those which are for debugging, drawing, user interaction and all this experimental things I am currently working on.

Usually functions, traits and structs are weakly organised.

Extension traits for bevy and other dependency types are for convinience.
They may be worth a PR to the respective dependency, or not.

Systems are usually complex and not only consist of a single system function but multiple
and some structs and components.

`belt_advance_items` was kind of complicated to implement.
It deals with advancing items on belts and through item inputs into buildings and item outputs onto belts.
`map_cache` is quite useful. It allows to organise entities on a grid, look up by integer coordinates.
Probably gonna use that in the future too but have separate map caches for different entity types.
`belt_input_output_hookup` uses map cache to connect belt and building inputs with outputs.
In conjunction with `simple_spawner` this helps and spawning some connected buildings and belts.

`assets` for now can keep some loading and organising assets systems and resources.

`camera` contains some spawn and configuration helpers for static and interactive cameras.
The default orbit camera can be used with mouse right drag, wheel and move while holding CTRL.

`config` has the type definitions for the single config file `config.ron`.