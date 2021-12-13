# mini-swc

A toy ES bundler & builder built w/ Rust. Its made entirely for fun, in an attempt to better understand how building and bundling works. It is also a playground for learning Rust.

This project has nothing to do with swc besides greatly relying on the amazing work people are doing there.

## Running

It is assumed that you do have Rust toolkit already set-up.

```zsh
cargo run $PATH_ENTRYPOINT $PATH_NODE_MODULES
```

## What does it do?

- [x] Parse a ES and TS module
- [x] Build AST (thanks to `swc`)
- [x] Build a (loosely structured) dependency graph
- [ ] Bundle in one script file

