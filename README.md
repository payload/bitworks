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

Things which are in the library are kind of less fragile.
But beware there is dead code in sub-directories and
Functions, traits and structs are weakly organised.
When looking at `lib.rs` we can see which directories and files are still in use and what is garbage:
`merger`, `game_types`, `systems`, `extension_traits` and `assets`.
The rest is from an old iteration, probably can be deleted.

Extesion traits for bevy and other dependency types are for convinience.
They may be worth a PR to the respective dependency, or not.

Systems are usually complex and not only consist of a single system function but multiple
and some structs and components.

`belt_advance_items` was kind of complicated to implement.
It deals with advancing items on belts and through item inputs into buildings and item outputs onto belts.
`map_cache` is quite useful. It allows to organise entities on a grid, look up by integer coordinates.
Probably gonna use that in the future too but have separate map caches for different entity types.

`game_types` is an exageration. Just a few structs and impls.

`assets` for now can keep some loading and organising assets systems and resources.

`merger` contains just a single system. It was kind of complicated, so it got its own file.
Maybe should move to systems directory?
