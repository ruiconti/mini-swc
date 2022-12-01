use std::collections::HashMap;
use std::path::PathBuf;

mod asset;
mod parser;
mod path;

pub fn build_dependency_graph(
    entrypoint: PathBuf,
    node_modules_path: PathBuf,
) -> HashMap<PathBuf, asset::Asset> {
    let mut id_counter: usize = 0;
    let mut queue: Vec<PathBuf> = Vec::new();
    let mut graph: HashMap<PathBuf, asset::Asset> = HashMap::new();
    // TODO: Use graph structure from petgraph: https://docs.rs/petgraph/latest/petgraph/

    queue.push(entrypoint);
    while queue.len() > 0 {
        let asset = asset::track_dependencies(&mut id_counter, queue.pop().unwrap());
        graph.insert(asset.clone().path(), asset.clone());

        for dependency_path in asset.dependencies().first_party().iter() {
            if path::is_ecmascript_file(dependency_path.to_path_buf()) {
                queue.push(dependency_path.to_path_buf());
            }
        }

        for src in asset.dependencies().third_party().iter() {
            match path::resolve_relative(&node_modules_path, &src) {
                Ok(mut mod_path) => {
                    if !mod_path.clone().extension().unwrap_or_default().eq("js") {
                        // If third-party is NOT a direct import e.g., import { fn } from 'esm/lib/fn.js'
                        // Try to load from ***inferred*** package index.
                        // TODO: Read from package.json to get righteous main file.
                        mod_path =
                            match path::resolve_relative(&mod_path, &String::from("/lib/index.js"))
                            {
                                Ok(path) => path,
                                Err(_err) => continue,
                            };
                    }
                    if !graph.contains_key(&mod_path) {
                        queue.push(mod_path);
                    }
                }
                Err(_err) => {
                    continue;
                }
            }
        }

        for src in asset.dependencies().exports().iter() {
            // Exports are assumed to be *always* relative, even though the spec is much broader, and it is allowed
            // to group and export any arbitrary module
            if !graph.contains_key(&*src) {
                queue.push(src.to_path_buf());
            }
        }
    }
    graph
}

fn main() {
    let (entrypoint, node_modules_path) = match path::extract_path_from_args() {
        Err(e) => {
            println!("error {}", e);
            std::process::exit(1)
        }
        Ok(v) => v,
    };
    let graph = build_dependency_graph(entrypoint, node_modules_path);
    println!("Dep graph\n:{:#?}", graph)
    // let packages = find_npm_packages(node_modules_path);
    // println!("Total monorepo dependencies: {:?}", packages.len());
}
