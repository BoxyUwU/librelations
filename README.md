# Relations for `bevy_ecs`

This crate implements the relations feature for bevy by building on top of a fork of bevy that introduces despawn hooks (see [on_remove_from_despawn](https://hackmd.io/7npDczZqTdK2gqHV1q9Rfw#Hooks)).

Relations allow you to build custom hierarchies and more generally "point" to other entities via a component. Each source entity can have multiple kinds of relations, each with their own data, and multiple relations of the same kind pointing to a different target entity.

Bevy's `Parent`/`Children` component's are a form of this pattern which historically (and currently) are implemented such that users can leave the hierarchy in an inconsistent or broken state. With this library anyone can make their own hierarchy like `Parent`/`Children` without having to worry about implementing it correctly.

See this (dead) RFC for more information on relations (note that not all of it is implemented in this crate): [min-relations](https://github.com/BoxyUwU/rfcs/blob/min-relations/rfcs/min-relations.md)

## How to add as a dependency

As this crate uses a forked bevy it cannot be easily uploaded to crates.io and depended on in the "normal" way.

The following should be added to `Cargo.toml` making sure to substitute "Relations Branch" and "Fork Branch" with the
values listed in table below:

```toml
[dependencies]
librelations = { git = "https://github.com/BoxyUwU/librelations", branch = "Relations Branch" }

[patch.crates-io]
bevy = { git = "https://github.com/BoxyUwU/bevy", branch = "Fork Branch" }
```

|Bevy Version|Fork Branch          |Relations Branch|
|------------|---------------------|----------------|
|`0.9.1`     |`despawn_hooks_0_9_1`|`release_0_9_1` |

## Known flaws
- No change detection, this would require either reimplementing `Mut` or bevy to expose a public `Mut::new`
- No `RemovedComponents` for relations
- Non-`Entity` targets are not supported
- Lack of `WorldQuery` support for filtering targets, have to use `Iterator::filter` manually
- No cycle detection for unrestricted relation graphs.
- Despawns are always recursive
- `insert_relation` has no fallible alternative
- Cant clear all relations of a kind on an entity