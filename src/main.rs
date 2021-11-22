use std::env;
use std::io;
use std::fs;
use swc_common::{
    self,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, SourceMap,
};
use std::path::{
    Component, Path, PathBuf
};
use swc_ecma_ast::Module;
use swc_ecma_parser::{lexer::Lexer, Capturing, Parser, StringInput, Syntax};

fn extract_path_from_arg() -> io::Result<PathBuf> {
    let args: Vec<String> = env::args().collect();
    let mut path = PathBuf::new();
    path.push(env::current_dir()?);
    path.push(&args[1]);
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Path {} not found.", &args[1])));
    }
    Ok(path)
}

fn parse_ecma_module(src_path: PathBuf) -> Module {
    // Read source file into a String
    let src = fs::read_to_string(&src_path).unwrap();
    // Create a SourceMap container
    let cm: Lrc<SourceMap> = Default::default();
    // Define an error Handler that will check for any lex/semantic/syntax errors in SourceFile
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    // Create a new SourceFile
    let fm = cm.new_source_file(
        FileName::Real(src_path),
        src,
    );

    // Parse src into ES tokens
    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
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

fn absolute_path(entrypoint: &PathBuf, relative: &String) -> io::Result<PathBuf> {
    let mut module_path = PathBuf::new();
    module_path.push(entrypoint.parent().unwrap());
    for component in Path::new(&relative).components() {
        if component == Component::Normal(component.as_os_str()) {
            // Ignore '.', '..' and win's root '\\'
            module_path.push(component.as_os_str())
        } 
    }
    if !module_path.is_file() {
        module_path = module_path.with_extension("ts");
    }
    match module_path.exists() {
        true => Ok(module_path),
        false => Err(io::Error::new(io::ErrorKind::NotFound, "Dependency could not be resolved.")),
    }
}

fn track_dependencies(module_path: PathBuf) -> (Vec<PathBuf>, Vec<String>) {
    let module = parse_ecma_module(module_path.clone());
    let mut first_parties: Vec<PathBuf> = Vec::new();
    let mut third_parties: Vec<String> = Vec::new();
    
    for statement in module.body.iter() {
    // println!("{:#?}", statement.clone());
        if statement.clone().is_module_decl() && statement.clone().module_decl().unwrap().is_import() {
            let stmt = statement.clone().module_decl().unwrap();
            let module_name = stmt.clone().import().unwrap().src.value.to_string();
            match absolute_path(&module_path, &module_name) {
                Ok(path) => first_parties.push(path),
                Err(err) => {
                    if err.kind() == io::ErrorKind::NotFound {
                        third_parties.push(module_name);
                    }
                }
            }
        }
    }
    println!("Module: {:?}", module_path);
    println!("First party dependencies: {:?}", first_parties);
    println!("Third party dependencies: {:?}", third_parties);
    return (first_parties, third_parties);
}

fn main() {
    let entrypoint = match extract_path_from_arg() {
        Err(e) => { println!("error {}", e); std::process::exit(1) },
        Ok(v) => v,
    };
    let mut dependency_queue: Vec<PathBuf> = Vec::new();
    // let mut third_parties: Vec<String> = Vec::new();
    dependency_queue.push(entrypoint);
    while dependency_queue.len() > 0 {
        let (first_parties_it, _third_parties_it) = track_dependencies(dependency_queue.pop().unwrap());
        for deps in first_parties_it.iter() {
            dependency_queue.push(deps.to_path_buf());
        }
        // dependency_queue.extend(first_parties_it.iter());
        // third_parties.extend(third_parties_it.iter());
    }
}
