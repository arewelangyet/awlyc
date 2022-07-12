use std::{env, fs, process::exit};

use awlyc_error::{Diagnostic, DiagnosticKind, DiagnosticReporter, FileId, Span};
use awlyc_parser::parse;
use smol_str::SmolStr;
use text_size::TextRange;

fn main() {
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
    let input_file: &str = &args[1];
    let src = if let Some(src) = fs::read_to_string(input_file).ok() {
        src
    } else {
        diagnostic_reporter.report(&Diagnostic {
            kind: DiagnosticKind::Error,
            msg: format!("could not open file `{}`", input_file),
            span: Span {
                range: TextRange::new(0.into(), 0.into()),
                file_id: FileId(SmolStr::from("")),
            },
        });
        exit(1);
    };
    let file_id = FileId(SmolStr::from(input_file));
    diagnostic_reporter.add_file(file_id.0.clone(), &src);
    let (decls, expr_arena, errors) = parse(&src, file_id);
    for err in &errors {
        diagnostic_reporter.report(err);
    }
    println!("{:#?}", expr_arena);
    println!("{:#?}", decls);
}
