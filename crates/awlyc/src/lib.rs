use std::{collections::HashMap, fs, path::PathBuf};

use awlyc_error::{Diagnostic, DiagnosticKind, DiagnosticReporter, FileId, Span};
use awlyc_parser::{
    ast::{Expr, Spanned},
    parse, Module,
};
use awlyc_values::{deserialize::from_awlyc_val, lower};
use la_arena::Arena;
use serde::de::DeserializeOwned;
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
            panic!("could not open file");
        }
        Ok(absolute_path) => absolute_path,
    }
}

/// Apologies to anyone reading this
fn parse_file(
    path: &str,
    modules: &mut HashMap<FileId, Module>,
    expr_arena: &mut Arena<Spanned<Expr>>,
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
    let (module, errors) = parse(&src, expr_arena, file_id.clone());
    for err in &errors {
        diagnostic_reporter.report(err);
    }

    let imports = module.imports.to_vec();
    modules.insert(file_id, module); // must insert before looping over imports to prevent infinite recursion

    for import in &imports {
        path.pop();
        path.push(import.path.as_str());
        parse_file(
            path.to_str().unwrap(),
            modules,
            expr_arena,
            diagnostic_reporter,
        )
    }
}

pub fn from_file<T>(path: &str) -> T
where
    T: DeserializeOwned,
{
    let mut modules = HashMap::new();
    let mut expr_arena = Arena::default();
    let mut diagnostic_reporter = DiagnosticReporter { files: vec![] };
    parse_file(
        path,
        &mut modules,
        &mut expr_arena,
        &mut diagnostic_reporter,
    );

    let res = lower(path, &modules, &expr_arena);
    let value = match res {
        Err(err) => {
            diagnostic_reporter.report(&err);
            panic!("")
        }
        Ok(value) => value,
    };

    match from_awlyc_val(&value) {
        Err(err) => {
            diagnostic_reporter.report(&err);
            panic!("")
        }
        Ok(value) => value,
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::from_file;

    #[test]
    fn basic() {
        #[derive(Debug, Deserialize)]
        struct Project {
            title: String,
            author: String,
        }

        let result: Project = from_file("../../examples/basic.awlyc");

        println!("{:#?}", result);
    }
}
