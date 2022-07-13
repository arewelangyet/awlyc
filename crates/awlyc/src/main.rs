use std::{collections::HashMap, env, fs, path::PathBuf, process::exit};

use awlyc_error::{Diagnostic, DiagnosticKind, DiagnosticReporter, FileId, Span};
use awlyc_parser::{parse, Module};
use smol_str::SmolStr;
use text_size::TextRange;

fn canonicalize_path(path: &str, diagnostic_reporter: &mut DiagnosticReporter) -> PathBuf {
    let absolute_path = fs::canonicalize(path);
    match absolute_path {
        Err(err) => {
            diagnostic_reporter.report(&Diagnostic {
                kind: DiagnosticKind::Error,
                msg: format!("could not open file `{}`: {}", path, err),
                span: Span {
                    range: TextRange::new(0.into(), 0.into()),
                    file_id: FileId(SmolStr::from("")),
                },
            });
            exit(1);
        }
        Ok(absolute_path) => absolute_path,
    }
}

/// Apologies to anyone reading this
fn parse_file(
    path: &str,
    modules: &mut HashMap<FileId, Module>,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
    let mut path = canonicalize_path(path, diagnostic_reporter);
    let file_id = FileId(SmolStr::from(path.to_str().unwrap()));

    // we've already parsed this file
    if modules.get(&file_id).is_some() {
        return;
    }

    let src = fs::read_to_string(&path).unwrap();
    diagnostic_reporter.add_file(file_id.0.clone(), src.clone());
    let (module, _, errors) = parse(&src, file_id.clone());
    for err in &errors {
        diagnostic_reporter.report(err);
    }

    let imports = module.imports.to_vec();
    modules.insert(file_id, module); // must insert before looping over imports to prevent infinite recursion

    for import in &imports {
        path.pop();
        path.push(import.path.as_str());
        parse_file(path.to_str().unwrap(), modules, diagnostic_reporter)
    }
}

fn main() {
    let mut modules = HashMap::new();
    let mut diagnostic_reporter = DiagnosticReporter { files: vec![] };
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        diagnostic_reporter.report(&Diagnostic {
            kind: DiagnosticKind::Error,
            msg: format!("please provide an input file"),
            span: Span {
                range: TextRange::new(0.into(), 0.into()),
                file_id: FileId(SmolStr::from("")),
            },
        });
        exit(1);
    }
    let input_file_path = &args[1];
    parse_file(input_file_path, &mut modules, &mut diagnostic_reporter);
    println!("{:#?}", modules);
}
