use std::collections::HashMap;
use std::path::PathBuf;
use std::{
    ffi::OsStr,
    fs, io,
    sync::atomic::{AtomicUsize, Ordering},
};
use swc_common::private::serde::de::IdentifierDeserializer;
use swc_common::{
    self,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, SourceMap,
};
use swc_ecma_ast::{Module, ModuleDecl};
use swc_ecma_parser::{lexer::Lexer, Capturing, EsConfig, Parser, StringInput, Syntax};

static MODULE_ID: AtomicUsize = AtomicUsize::new(0);

mod path;

#[derive(Debug)]
struct Dependencies {
    first_party: Vec<PathBuf>,
    exports: Vec<PathBuf>,
    third_party: Vec<String>,
}

#[derive(Debug)]
struct ModuleDependency {
    id: usize,
    path: PathBuf,
    dependencies: Dependencies,
    mappings: HashMap<PathBuf, usize>,
}

impl Clone for Dependencies {
    fn clone(&self) -> Self {
        Dependencies {
            first_party: self.first_party.clone(),
            third_party: self.third_party.clone(),
            exports: self.exports.clone(),
        }
    }
}

impl Clone for ModuleDependency {
    fn clone(&self) -> Self {
        ModuleDependency {
            id: self.id.clone(),
            path: self.path.clone(),
            dependencies: self.dependencies.clone(),
            mappings: self.mappings.clone(),
        }
    }
}

enum ModuleType {
    Javascript(Syntax),
    Typescript(Syntax),
}

// Typescript lexer
// let lexer = Lexer::new(
//     Syntax::Typescript(TsConfig {
//         tsx: true,
//         decorators: false,
//         dynamic_import: true,
//         dts: true,
//         no_early_errors: true,
//         import_assertions: true,
//     }),
//     Default::default(),
//     StringInput::from(&*fm),
//     None,
// );

fn parse_ecma_module(src_path: PathBuf) -> Module {
    // Read source file into a String
    let src = fs::read_to_string(&src_path).unwrap();
    // Create a SourceMap container
    let cm: Lrc<SourceMap> = Default::default();
    // Define an error Handler that will check for any lex/semantic/syntax errors in SourceFile
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    // Create a new SourceFile
    let fm = cm.new_source_file(FileName::Real(src_path), src);

    // Parse src into ES tokens
    let lexer = Lexer::new(
        Syntax::Es(EsConfig {
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );
    // println!("Lexer tokens: ");
    // for token in lexer.clone().into_iter() {
    //     println!("{:?}", token)
    // }

    // Create Parser to turn tokens into an AST
    let capturing = Capturing::new(lexer);
    let mut parser = Parser::new_from(capturing);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let module = parser
        .parse_module()
        .map_err(|e| e.into_diagnostic(&handler).emit())
        .expect("Failed to parse module.");

    module
}

fn track_dependencies(module_path: PathBuf) -> ModuleDependency {
    let module = parse_ecma_module(module_path.clone());
    let mut first_parties: Vec<PathBuf> = Vec::new();
    let mut third_parties: Vec<String> = Vec::new();
    let mut exports: Vec<PathBuf> = Vec::new();

    for item in module.body.iter() {
        // println!("{:#?}", item.clone());
        if item.clone().is_module_decl() // stmt.rs -> Stmt.Decl
            && item.clone().module_decl().unwrap().is_import()
        {
            let declaration = item.clone().module_decl().unwrap();
            match declaration.clone() {
                ModuleDecl::Import(import) => {
                    let import_mod_name = import.src.value.to_string();
                    let import_abs_path = path::absolute_path(&module_path, &import_mod_name);

                    match import_abs_path {
                        Ok(path) => first_parties.push(path),
                        Err(err) => {
                            if err.kind() == io::ErrorKind::NotFound {
                                third_parties.push(import_mod_name);
                            }
                        }
                    }
                }
                ModuleDecl::ExportAll(export) => {
                    let export_mod_name = export.src.value.to_string();
                    let export_abs_path = path::absolute_path(&module_path, &export_mod_name);

                    match &export_abs_path {
                        Ok(path) => exports.push(path.to_path_buf()),
                        Err(err) => {
                            if err.kind() == io::ErrorKind::NotFound {
                                println!("Couldn't find {:#?}", export_abs_path)
                            }
                        }
                    }
                }
                ModuleDecl::ExportNamed(_export) => {
                    // export.src.value is a JsWord, which I don't know how to get the actual value
                    ()
                }
                _ => (),
            }
        }
    }

    let module: ModuleDependency = ModuleDependency {
        id: MODULE_ID.fetch_add(1, Ordering::SeqCst),
        path: module_path,
        dependencies: Dependencies {
            first_party: first_parties,
            third_party: third_parties,
            exports: exports,
        },
        mappings: HashMap::new(),
    };

    // println!("Module: {:#?}", &module);
    return module;
}

fn is_ecmascript_file(fpath: PathBuf) -> bool {
    let extension = fpath
        .extension()
        .unwrap_or(&OsStr::new(""))
        .to_str()
        .unwrap_or(&"");
    return vec!["js", "jsx"].contains(&extension);
}

fn build_dependency_graph(
    entrypoint: PathBuf,
    node_modules_path: PathBuf,
) -> HashMap<usize, ModuleDependency> {
    let mut queue: Vec<PathBuf> = Vec::new();
    let mut graph: HashMap<usize, ModuleDependency> = HashMap::new();
    // let mut dependencies: Vec<ModuleDependency> = Vec::new();

    queue.push(entrypoint);
    while queue.len() > 0 {
        let asset = track_dependencies(queue.pop().unwrap());
        graph.insert(asset.id, asset.clone());
        for dependency_path in asset.dependencies.first_party.iter() {
            if is_ecmascript_file(dependency_path.to_path_buf()) {
                queue.push(dependency_path.to_path_buf());
                println!("'got here {:#?}", queue);
            }
        }

        for package_root in asset.dependencies.third_party.iter() {
            let mut abs_path = path::join_path(&node_modules_path, &package_root).unwrap();

            if !abs_path.clone().extension().unwrap_or_default().eq("js") {
                abs_path = path::join_path(&abs_path, &String::from("/lib/index.js")).unwrap();
            }
            // println!(
            //     "3rd party index root: {:#?} {:?}",
            //     abs_path.clone(),
            //     PathBuf::from(&abs_path).exists(),
            // );
            queue.push(abs_path);
        }
    }
    graph
}

fn is_npm_package(directory: &PathBuf) -> bool {
    if !directory.is_dir() {
        return false;
    }

    for item in directory.read_dir().unwrap() {
        let found_package = item
            .map_err(|x| false)
            .and_then(|entry| Ok(entry.file_name().to_str().unwrap() == "package.json"))
            .map_err(|x| false);

        if found_package.unwrap_or(false) {
            return true;
        }
    }
    return false;
}

fn find_npm_packages(node_modules_path: PathBuf) -> Vec<PathBuf> {
    let mut packages: Vec<PathBuf> = Vec::new();
    for item in node_modules_path.clone().read_dir().unwrap() {
        let entry = item.unwrap().path();
        if is_npm_package(&entry) {
            packages.push(entry)
        }
    }
    return packages;
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
