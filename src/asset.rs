// use std::collections::HashMap;
use crate::{parser, path};
use std::io;
use std::path::PathBuf;
use swc_ecma_ast::ModuleDecl;

// TODO: Improve expressiveness w/ algebraic dt
#[derive(Debug)]
pub struct Dependencies {
    first_party: Vec<PathBuf>,
    third_party: Vec<String>,
    exports: Vec<PathBuf>,
}

impl Dependencies {
    pub fn first_party(&self) -> Vec<PathBuf> {
        return self.first_party.clone();
    }
    pub fn third_party(&self) -> Vec<String> {
        return self.third_party.clone();
    }
    pub fn exports(&self) -> Vec<PathBuf> {
        return self.exports.clone();
    }
}

#[derive(Debug)]
pub struct Asset {
    id: usize,
    path: PathBuf,
    dependencies: Dependencies,
}

impl Asset {
    pub fn dependencies(&self) -> Dependencies {
        return self.dependencies.clone();
    }

    pub fn path(&self) -> PathBuf {
        return self.path.clone();
    }
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

impl Clone for Asset {
    fn clone(&self) -> Self {
        Asset {
            id: self.id.clone(),
            path: self.path.clone(),
            dependencies: self.dependencies.clone(),
        }
    }
}

pub fn track_dependencies(id: &mut usize, module_path: PathBuf) -> Asset {
    let module = parser::parse_em(module_path.clone());
    let mut first_parties: Vec<PathBuf> = Vec::new();
    let mut third_parties: Vec<String> = Vec::new();
    let mut exports: Vec<PathBuf> = Vec::new();

    for item in module.body.iter() {
        if item.clone().is_module_decl() {
            let declaration = item.clone().module_decl().unwrap();
            // TODO: Review spec for proper coverage
            match declaration.clone() {
                ModuleDecl::Import(import) => {
                    let import_mod_name = import.src.value.to_string();
                    let import_abs_path = path::resolve_relative(&module_path, &import_mod_name);

                    match import_abs_path {
                        Ok(path) => first_parties.push(path),
                        Err(err) => {
                            if err.kind() == io::ErrorKind::NotFound {
                                third_parties.push(import_mod_name);
                            }
                        }
                    }
                }
                ModuleDecl::ExportDecl(_export) => {
                    // TODO: add_impl
                    ()
                }
                ModuleDecl::ExportDefaultDecl(_export) => {
                    // TODO: add_impl
                    ()
                }
                ModuleDecl::ExportDefaultExpr(_export) => {
                    // TODO: add_impl
                    ()
                }
                ModuleDecl::ExportAll(export) => {
                    // TODO: add_impl
                    let export_mod_name = export.src.value.to_string();
                    let export_abs_path = path::resolve_relative(&module_path, &export_mod_name);

                    match export_abs_path {
                        Ok(path) => exports.push(path.to_path_buf()),
                        Err(err) => {
                            if err.kind() == io::ErrorKind::NotFound {
                                println!("Couldn't find {:#?}", export_mod_name)
                            }
                        }
                    }
                }
                ModuleDecl::ExportNamed(export) => {
                    let export_mod_name = export.src.map_or(String::new(), |x| x.value.to_string());
                    // TODO: Non-idiomatic short circuit
                    if export_mod_name.len() == 0 {
                        continue;
                    }

                    let export_abs_path = path::resolve_relative(&module_path, &export_mod_name);
                    match export_abs_path {
                        Ok(path) => exports.push(path.to_path_buf()),
                        Err(err) => {
                            if err.kind() == io::ErrorKind::NotFound {
                                println!("Couldn't find {:#?}", export_mod_name)
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }

    *id = *id + 1;
    let module: Asset = Asset {
        id: *id,
        path: module_path,
        dependencies: Dependencies {
            first_party: first_parties,
            third_party: third_parties,
            exports: exports,
        },
    };

    return module;
}
