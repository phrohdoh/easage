# `contrib`

Contributed examples of how users may use the `easage` library and tools.

---

## `easage-pack.cmd` and `easage-unpack.cmd`

Scripts for drag-and-drop operations in Windows' File Explorer.

![usage](https://user-images.githubusercontent.com/5067989/31890321-9c3b6a04-b7fa-11e7-87e0-037db87b73f2.gif)

Input can be a selection of alpha-numeric filenames or folders.

To use simply select some files or folders and drop them on top of the desired `.cmd` script.

`easage-pack.cmd` will pack your files/folders into the `BIG` format while `easage-unpack.cmd` unpacks selected `BIG`'s.

A [prefix](https://github.com/Phrohdoh/easage/blob/master/contrib/easage-pack.cmd#L27) for `easage-pack.cmd` `BIG` output(s) can be configured for your particular mod.

`easage-list.cmd` will list the packed filenames that the `BIG` contains.

It only takes a single `BIG` file as input, a selection will not work.

The [verbose](https://github.com/Phrohdoh/easage/blob/master/contrib/easage-list.cmd#L13) variable can be set to list more information about the `BIG`.

## License

[MIT](LICENSE.md)

## Contributing

Any contribution you intentionally submit for inclusion in the work, as defined
in the `LICENSE.md` file, shall be licensed as above, and are subject to the
project's [CLA](https://gist.github.com/Phrohdoh/d402395a3d8c453e4399f7ae345c0d72).
