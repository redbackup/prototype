# Redbackup Prototype

This is a prototype of the [redbackup system](https://www.redbackup.org/) written in [Rust](http://rust-lang.org/).

## License

This is free software, published under the [AGPL-License](https://www.gnu.org/licenses/agpl-3.0.en.html).

## Building and testing

Firstly, [install latest stable rust](https://doc.rust-lang.org/book/second-edition/ch01-01-installation.html)

```bash
$ git clone ssh://git@git.redbackup.org:40001/red/prototype.git
$ cd prototype/
$ cargo build --all
```

The project ist structured in the form of a [cargo workspace](https://github.com/rust-lang/rfcs/blob/master/text/1525-cargo-workspace.md), which means that you can build and tests all module at once in the project root using the `--all` parameter.

## Development Guidelines

The code must follow the most recent [Rust RFCs](https://aturon.github.io/) (naming conventions etc.)

Code must be formatted [rustfmt](https://github.com/rust-lang-nursery/rustfmt).

## Plans for the future

Because this project is just a prototype, there is still a lot to do and we are far from future complete.

The following sections document ideas for further improvements.