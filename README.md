# `easage`

A library that provides programmatic manipulation of BIG archives.

---

## What is a `BIG` archive?

BIG files are an archive format used in many games published by Electronic Arts.
The supported features vary between games, with some using compression or
encryption, but for SAGE, the files are trivially concatenated together and
wrapped with a header containing a series of index entries that located a given
file within the archive.

> Note: The above was lifted directly from https://github.com/TheAssemblyArmada/Thyme/wiki/BIG-File-Format

Noteable games built on the SAGE engine are:
* Battle For Middle-Earth (1, 2, RotWK)
* Command & Conquer Generals (and the expansion Zero Hour)

## Building

You must have the [Rust](https://rust-lang.org) toolchain installed (which includes `cargo`):

```sh
cargo build --release
```

## Running

Included in this source tree are two example command-line applications that use `easage`:

* bigread (prints out metadata given a single .big filepath)
* bigpack (recursively packages a directory into a .big file [run with the `help` command])

```sh
cargo run --bin bigread --release -- path/to/a/file.big

# or if you have the `bigread` binary itself
bigread path/to/a/file.gif
```

```sh
cargo run --bin bigpack --features="clap" --release -- --source test_data --output output/path.big

# or if you have the `bigpack` binary itself
bigpack --source test_data --output output/path.big
```

## License

[MIT](LICENSE.md)

## Contributing

Any contribution you intentionally submit for inclusion in the work, as defined
in the `LICENSE.md` file, shall be licensed as above, and are subject to the
project's [CLA](https://gist.github.com/Phrohdoh/d402395a3d8c453e4399f7ae345c0d72).