# A planner's plan

## Code-wise

- Improve borrowing: Get rid of `clone()` and `to_<type>()`
- Make better usage of `trait` and mechanic composition -- e.g. `Asset` module

## Feature-wise

- Add support for naïve bundling
- Implement dependency graph using a way more efficient graph map structure, available on petgraph crate: https://docs.rs/petgraph/latest/petgraph/
    - Add support for graphviz
- Calculate bundle-size, per package
- Calculate a traversal between an entrypoint and a specific module
- Simulate `webpack`'s chunking system, in an attempt to generate a bundlestat.json file w/o building
- Add a chunking system aware of intentional chunk splitting per dynamic imports