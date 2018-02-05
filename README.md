# `easage`

[![Build Status](https://travis-ci.org/Phrohdoh/easage.svg?branch=master)](https://travis-ci.org/Phrohdoh/easage) [![Build Status](https://ci.appveyor.com/api/projects/status/github/Phrohdoh/easage?branch=master&svg=true)](https://ci.appveyor.com/project/Phrohdoh/easage) [![GitHub (pre-)release](https://img.shields.io/github/release/Phrohdoh/easage/all.svg)](https://github.com/Phrohdoh/easage/releases) [![Chat on IRC](https://img.shields.io/badge/irc-%23orcaware%20on%20freenode-brightgreen.svg)](https://kiwiirc.com/client/irc.freenode.net/#orcaware)

A library that provides programmatic manipulation of BIG archives.

---

## What is a `BIG` archive?

BIG files are an archive format used in many games published by Electronic Arts.
The supported features vary between games, with some using compression or
encryption, but for SAGE, the files are trivially concatenated together and
wrapped with a header containing a series of index entries that located a given
file within the archive.

> Note: The above was lifted directly from https://github.com/TheAssemblyArmada/Thyme/wiki/BIG-File-Format

Notable games built on the SAGE engine are:
* Battle For Middle-Earth (1, 2, RotWK)
* Command & Conquer Generals (and the expansion Zero Hour)

## Building

You must have the [Rust](https://rust-lang.org) toolchain installed (which includes `cargo`):

```sh
cargo build --release
```

## Testing

To run the included unit tests execute the following command in the root of the project:

```sh
cargo test
```

We do require that all tests pass before a PR is merged.

If you need help getting a test to pass (existing or one you have written) do not hesitate to reach out to us (either via GitHub issues or on IRC).

## Running

Included in this source tree is a command-line application named `easage` that uses the `easage` library.

See the [src/bin/](./src/bin/) directory for more details.

---

See [contrib](https://github.com/Phrohdoh/easage/tree/master/contrib) for more usage suggestions.

## Getting help

I am often present in the `#orcaware` channel on `irc.freenode.net`.

Note that this channel has many topics of discussion so you may see conversation about other projects there too.

If IRC is not your thing or you don't get a good response I am happy to respond to [GitHub issues](https://github.com/Phrohdoh/easage/issues/new) as well.

## License

[MIT](LICENSE.md)

## Contributing

Any contribution you intentionally submit for inclusion in the work, as defined
in the `LICENSE.md` file, shall be licensed as above, and are subject to the
project's [CLA](https://gist.github.com/Phrohdoh/d402395a3d8c453e4399f7ae345c0d72).