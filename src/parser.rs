use std::{ffi, fs, path::PathBuf};
use swc_common::{
    self,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, SourceMap,
};
use swc_ecma_ast::Module;
use swc_ecma_parser::{lexer::Lexer, Capturing, EsConfig, Parser, StringInput, Syntax, TsConfig};

fn infer_syntax(mod_path: PathBuf) -> Syntax {
    let ts_ext = vec!["ts", "tsx"];
    // let ES_EXT = vec!["js", "jsx", "jsm"];
    let ext = mod_path.extension().unwrap_or(&ffi::OsStr::new("js"));
    return match ts_ext.contains(&ext.to_str().unwrap()) {
        true => Syntax::Typescript(TsConfig {
            tsx: true,
            decorators: false,
            dynamic_import: true,
            dts: true,
            no_early_errors: true,
            import_assertions: true,
        }),
        false => Syntax::Es(EsConfig {
            ..Default::default()
        }),
    };
}

// Parses an EcmaScript module located in `src_path`.
// It is a (temporary) wrapper of `swc`'s parsing capabilities.
//
// ```
// let path = PathBuf::from("/app/src/index.js");
// let module = parse_em(path);
// ```
pub fn parse_em(mod_path: PathBuf) -> Module {
    let cm: Lrc<SourceMap> = Default::default();
    // Read source file into a String
    let src = fs::read_to_string(&mod_path).unwrap();
    let fm = cm.new_source_file(FileName::Real(mod_path.clone()), src);
    // Define an error Handler that will check for any lex/semantic/syntax errors in SourceFile
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let syntax = infer_syntax(mod_path);
    // Create Parser to turn tokens into an AST
    let lexer = Lexer::new(syntax, Default::default(), StringInput::from(&*fm), None);
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
