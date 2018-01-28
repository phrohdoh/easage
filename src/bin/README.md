# `easage` the binary

This application allows you to:

- List the contents of an archive
- Create a new archive
- Extract from an existing archive

## Building

```sh
cargo build --release --bin easage --features clap
```

The output binary will be written to `target/release/easage`.

## Running

There are two ways to run this tool.

Run the `cargo build` command above then invoke the binary created directly (or put it somewhere on your shell's `$PATH`), or

Run via `cargo run` like so:

```sh
cargo run --features clap -- <subcommand> [flags] [options]
```

If you want to run via `cargo` replace `easage` in the following commands with `cargo run --features clap --`.

### Examples:

```sh
easage list path/to/a/file.big
```

```sh
easage pack --source test_data --output output/path.big --kind BIG4
```

```sh
easage unpack --source path/to/a.big --output the/directory/to/unpack/into/
```
