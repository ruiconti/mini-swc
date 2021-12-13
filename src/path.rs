use std::env;
use std::path::{Component, Path, PathBuf};
use std::{
    ffi::OsStr,
    io::{Error, ErrorKind, Result},
};

pub fn extract_path_from_args() -> Result<(PathBuf, PathBuf)> {
    let args: Vec<String> = env::args().collect();
    println!("Arg: {:?}", args);
    let (entrypoint, node_module_path) = (
        create_path_from_relative(&args[1]),
        create_path_from_relative(&args[2]),
    );
    if entrypoint.is_err() {
        return Err(entrypoint.err().unwrap());
    }
    if node_module_path.is_err() {
        return Err(node_module_path.err().unwrap());
    }
    return Ok((entrypoint.ok().unwrap(), node_module_path.ok().unwrap()));
}

pub fn create_path_from_relative(relative: &String) -> Result<PathBuf> {
    let mut path = PathBuf::new();
    path.push(env::current_dir()?);
    path.push(relative);
    if !path.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Path {} not found.", &relative),
        ));
    }
    Ok(path)
}

// TODO: Review docstring
//
// Useful path resolver
//
// If `root` is a directory, it is simple path joiner. It is aware of relative paths
//
// ```
// let root = PathBuf::from("/tmp/proj/node_modules/@fluentui/src/");
// let relative = "../../react/lib/index.js".to_string();
// let path = resolve_relative(&root, relative);
// assert_eq(path, PathBuf::from("/tmp/proj/node_modules/react/lib/index/js"))
// ```
//
// If `root` is a file, it is a path joiner relative to root file.
//
// ```
// let root = PathBuf::from("/tmp/proj/src/apps/auth/index.js");
// let relative = "./jwt.js".to_string();
// let path = resolve_relative(&root, relative);
// assert_eq(path, PathBuf::from("/tmp/proj/src/apps/auth/jwt.js"))
// ```
//
// There is an attempt to parse javascript files that ommit file extension.
//
// ```
// let root = PathBuf::from("/tmp/proj/src/apps/auth/index.js");
// let relative = "./jwt".to_string();
// let path = resolve_relative(&root, relative);
// assert_eq(path, PathBuf::from("/tmp/proj/src/apps/auth/jwt.js"))
// ```
pub fn resolve_relative(root: &PathBuf, relative: &String) -> Result<PathBuf> {
    let mut path = PathBuf::new();
    path.push(if root.is_file() {
        root.parent().unwrap()
    } else {
        root
    });

    for component in Path::new(&relative).components() {
        match component {
            Component::ParentDir => {
                let mut components = path.components();
                components.next_back();
            }
            Component::Normal(component) => {
                path.push(component);
            }
            _ => {
                // Purposefully ignore "." and "\\" components
                ()
            }
        }
    }

    // Tries to fallback and parse it as a .js file
    if !path.is_dir() && !path.is_file() {
        path = path.with_extension("js");
    }

    match path.exists() {
        true => Ok(path),
        false => Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "Resolve relative failed because the path {:#?} does not exist.",
                path
            ),
        )),
    }
}

pub fn is_ecmascript_file(fpath: PathBuf) -> bool {
    let extension = fpath
        .extension()
        .unwrap_or(&OsStr::new(""))
        .to_str()
        .unwrap_or(&"");
    // TODO: Add all ES file extensions
    return vec!["js", "jsx"].contains(&extension);
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
