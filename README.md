# Satisfactory Accounting

This is a tool for players of the game Satisfactory. In contrast to many other tools for
Satisfactory which focus on calculating what production lines are needed to set up a new
factory, this tool is intended for keeping track of the factories you already have. It
provides a heirarchical accounting of what inputs are consumed and produced by groups of
factory machines. This allows you to see quickly which resources are available and where,
though it does not keep track of flows, so it does not know where resources come from,
only whether each level has more production than consumption.

See the tool at https://satisfactory-accounting.github.io

## Running

Satisfactory Accounting is based on [Yew](https://yew.rs/) and uses
[Trunk](https://trunkrs.dev/) for building and local dev serving.

For initial setup, you need to install `trunk` and the Rust wasm target:

```
cargo install trunk
rustup target add wasm32-unknown-unknown
```

To compile and run, changed to the `satisfactory-accounting-app` directory and use `trunk
serve`. This will compile and start a dev server on port 8080.

## Building for Release

We use trunk again for this. Delete any existing `dist` directory, then run `trunk build
--release`.

To release it, clone the `satisfactory-accounting.github.io` repository, and

```shell
$ rsync -ah --inplace --no-whole-file --info=progress2 dist/ /path/to/satisfactory-accounting.github.io
```

Note that the trailing slash on `dist/` is important otherwise it will create a nested
`dist` directory in the publish repository.

## Updating the Database

Download the database from [Satisfactory
Tools](https://github.com/greeny/SatisfactoryTools) to `satisfactory-db/data.json`, and
use `cargo run` to run the `satisfactory-db` binary. This will output the Satisfactory
Accounting database to stdout.
