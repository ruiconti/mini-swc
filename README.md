# mini-swc

A toy ES bundler & builder built w/ Rust. Its made entirely for fun, in an attempt to better understand how building and bundling works. It is also a playground for learning Rust.

This project has nothing to do with swc besides greatly relying on the amazing work people are doing there.

## Running

```
cargo run <path-to-entrypoint> <path-to-node_modules-dir>
```

## What does it do?

- [x] Parse a ES and TS file
- [x] Build AST (thanks to `swc`)
- [x] Build a (loosely structured) dependency graph
- [ ] Bundle in one script file

