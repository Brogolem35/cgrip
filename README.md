# cgrip

A tool to extract resources from `.cg` files used by French Bread games such as Melty Blood and Under Night.

Early WIP. Expect problems. Help is appreciated.

This is an almost line by line reimplemtation of the cgpack script, whose most up-to-date version is almost as elusive as the Turkish dub of LOGH.

## Known Issues

- It is only tested on MBAACC on Linux. Further testing would be appreciated.

- Packaging resources is not implemented yet.

- `--tile_width` argument does nothing.

## Building and Usage

Building requires [Rust](https://rust-lang.org/) toolchain.

Type `cargo build --release` to build. The executable will be avaible on the path `/target/release/cgrip`.

Usage is as follows:
```sh
cgrip --help # prints out the help menu
cgrip ries.cg # extract a given .cg file
```

You can also directly run it using the `cargo run --release -- {arguments}` command.

The use of release build is highly encouraged. Debug build is slow as hell.
