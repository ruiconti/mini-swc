use std::env;
use std::io::{Error, ErrorKind, Result};
use std::path::{Component, Path, PathBuf};

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

pub fn absolute_path(current_dir: &PathBuf, relative: &String) -> Result<PathBuf> {
    let mut module_path = PathBuf::new();
    module_path.push(current_dir.parent().unwrap());
    for component in Path::new(&relative).components() {
        if component == Component::Normal(component.as_os_str()) {
            // Ignore '.', '..' and win's root '\\'
            module_path.push(component.as_os_str())
        }
    }
    if !module_path.is_dir() && !module_path.is_file() {
        module_path = module_path.with_extension("js");
    }

    match module_path.exists() {
        true => Ok(module_path),
        false => Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "Path could not be resolved: {:#?} {} {:#?}",
                current_dir, relative, module_path
            ),
        )),
    }
}

pub fn join_path(a: &PathBuf, b: &String) -> Result<PathBuf> {
    let mut path = PathBuf::new();
    path.push(a);
    for component in Path::new(&b).components() {
        if component == Component::Normal(component.as_os_str()) {
            // Ignore '.', '..' and win's root '\\'
            path.push(component.as_os_str())
        }
    }

    if !path.is_dir() && !path.is_file() && !path.exists() {
        path = path.with_extension("js");
    }

    match path.exists() {
        true => Ok(path),
        false => Err(Error::new(
            ErrorKind::NotFound,
            format!("Path could not be resolved: {:#?} {} {:#?}", a, b, path),
        )),
    }
}
