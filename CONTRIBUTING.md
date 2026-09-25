# Contributing

Issues and pull requests are welcome. New fishes and new tanks most of all.

## Issues

**Something broke.** Say what you did, what you saw, and what you expected instead. Include your system, your terminal and its size (a view that breaks only when the window is small is still a bug), and `fishtanks --version`. A screenshot says it faster than a paragraph.

**An idea.** Open a proposal. For a new fish or tank, draw it: put the sprite in a code block, give it a name, and say what it does that nothing else in the water does.

## Running it from source

Install [Rust](https://rustup.rs), then:

```sh
cargo run
```

`cargo run -- --debug` opens the lab bench instead: a separate save where the god and debug commands (`/spawn`, `/give`, `/add`, `/reset`...) work, so you can try a thing without fishing for it. Typing `!debugmode` in a normal game toggles the same thing.

## Pull requests

These four have to pass, and they are what CI runs:

```sh
cargo build
cargo test --no-fail-fast
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

The codebase keeps a few habits, and a pull request should keep them too:

- **No code comments.** If a name needs explaining, rename it.
- **Capabilities, not identity.** Behaviour comes from a flag on a species' or tank's config row, or a small trait it implements. Never `if species == FishSpecies::Koi`.
- **One home for every fact.** A value that can be derived from an existing table is derived, not written down a second time.
- **Every view works on a small screen.** Nothing is hidden because the window is small: it wraps, scrolls or gets denser instead. The journeys check each screen at 100×30, 80×24, 60×18 and 40×14.
- **Commit messages** are a few lowercase words, then `glupglup`, then an emoji if you like: `koi sway glupglup 🎏`.

### Adding a fish

A species is one `FishSpecies` variant and its one `SpeciesConfig` row in [`src/fishes/species.rs`](src/fishes/species.rs): the body, the colours, the speed, the rarity, and the flags that say what it can do (`buyable`, `mutatable`, `habitat`...). Most new fishes need nothing else.

### Adding a tank

A tank is one `TankKind` variant and its `TankConfig` row in [`src/tank/mod.rs`](src/tank/mod.rs), plus its scenery in [`src/tanks/`](src/tanks/). A tank is a set of rules, not only a background: say in the pull request what changes for a fish that lives there.

### Art

If the art is not yours, say where it came from and who drew it, and keep the artist's signature. That is how the art already in the tanks is credited.

### Testing what is on screen

The tests drive the real game through `Tui` in [`src/testing/`](src/testing/) and assert on the cells it draws. A screenplay is the quickest way to look at something:

```text
size 60 18
run /feed
tick 40
snap food in the water
expect cash
```

```sh
cargo run --example fishplay -- my-scene.play --print
```

`--print` shows each `snap` as text, and the reel is also written to `target/fishplay/reels/` as HTML, in colour. Player journeys live in [`tests/journeys/`](tests/journeys/): a new thing a player can do gets one, written with keystrokes the way a player would do it.
