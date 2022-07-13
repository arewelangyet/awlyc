use std::{collections::HashMap, path::PathBuf};

use awlyc_error::{Diagnostic, DiagnosticKind, FileId, Span};
use awlyc_parser::{
    ast::{Expr, ExprIdx, FnDecl},
    Module,
};
use la_arena::Arena;
use smol_str::SmolStr;
use text_size::TextRange;

type ValueResult = Result<AwlycValue, Diagnostic>;

#[derive(Debug)]
pub enum AwlycValue {
    String(SmolStr),
    Int(u64),
    Float(f64),
    Array(Vec<AwlycValue>),
    Record(HashMap<SmolStr, AwlycValue>),
}

struct LoweringCtx<'a> {
    entry: &'a PathBuf,
    modules: &'a HashMap<FileId, Module>,
    expr_arena: &'a Arena<Expr>,
}

impl<'a> LoweringCtx<'a> {
    pub fn new(
        entry: &'a PathBuf,
        modules: &'a HashMap<FileId, Module>,
        expr_arena: &'a Arena<Expr>,
    ) -> Self {
        Self {
            entry,
            modules,
            expr_arena,
        }
    }

    fn file_id(&self, path: &PathBuf) -> FileId {
        FileId(SmolStr::from(path.to_str().unwrap()))
    }

    pub(crate) fn lower(&self) -> ValueResult {
        let module = &self.modules[&self.file_id(self.entry)];
        if let Some(expr) = module.expr {
            self.lower_expr(expr, module, None)
        } else {
            return Err(Diagnostic {
                kind: DiagnosticKind::Error,
                msg: format!("missing expression (nothing to evaluate)"),
                span: Span {
                    range: TextRange::new(0.into(), 0.into()),
                    file_id: self.file_id(self.entry),
                },
            });
        }
    }

    /// Index of the expression to lower, optionally pass substitutions
    /// Substitutions are basically the function parameters
    /// When a function is called, uses of the parameters are just substitutions of the arguments
    fn lower_expr(
        &self,
        idx: ExprIdx,
        module: &Module,
        substitutions: Option<&[(SmolStr, ExprIdx)]>,
    ) -> ValueResult {
        let val = match &self.expr_arena[idx] {
            // This is gonna be a substitution
            Expr::Path(path) => {
                assert_eq!(path.len(), 1);
                let path = path.first().unwrap();
                if let Some(substitutions) = substitutions {
                    for (name, expr) in substitutions {
                        if name == path {
                            return self.lower_expr(*expr, module, Some(substitutions));
                        }
                    }
                    return Err(Diagnostic {
                        kind: DiagnosticKind::Error,
                        msg: format!("unknown identifier referenced `{}`", path),
                        span: Span {
                            range: TextRange::new(0.into(), 0.into()),
                            file_id: self.file_id(self.entry),
                        },
                    });
                } else {
                    return Err(Diagnostic {
                        kind: DiagnosticKind::Error,
                        msg: format!("unknown identifier referenced `{}`", path),
                        span: Span {
                            range: TextRange::new(0.into(), 0.into()),
                            file_id: self.file_id(self.entry),
                        },
                    });
                }
            }
            Expr::Int(n) => AwlycValue::Int(*n),
            Expr::Float(n) => AwlycValue::Float(*n),
            Expr::String(v) => AwlycValue::String(v.clone()),
            Expr::Array(els) => {
                let mut arr = vec![];
                for el in els {
                    arr.push(self.lower_expr(*el, module, substitutions)?);
                }
                AwlycValue::Array(arr)
            }
            Expr::Record(fields) => {
                let mut record = HashMap::new();
                for (k, v) in fields.iter() {
                    record.insert(k.clone(), self.lower_expr(*v, module, substitutions)?);
                }
                AwlycValue::Record(record)
            }
            Expr::Call(call) => {
                let callee = match &self.expr_arena[call.callee] {
                    Expr::Path(path) => path,
                    _ => unreachable!(),
                };

                // only allow foo() or foo.bar()
                if callee.len() > 2 {
                    return Err(Diagnostic {
                        kind: DiagnosticKind::Error,
                        msg: format!("unknown function referenced `{}`", callee.join(".")),
                        span: Span {
                            range: TextRange::new(0.into(), 0.into()),
                            file_id: self.file_id(self.entry),
                        },
                    });
                }

                // There's no import
                if callee.len() == 1 {
                    let f_name = callee.first().unwrap();
                    let f = self.find_function_in_module(f_name, module)?;
                    return self.expand_function(f, &call.args, module);
                }

                let import_alias = callee.first().unwrap(); // import foo "path.awlyc" -- foo is the import_alias
                let import_idx = module
                    .imports
                    .iter()
                    .position(|import| import.name == *import_alias);
                if let Some(idx) = import_idx {
                    let mut path = self.entry.clone();
                    path.pop();
                    path.push(module.imports[idx].path.as_str());
                    let module_id = self.file_id(&path);
                    let module = &self.modules[&module_id];
                    let f = self.find_function_in_module(callee.last().unwrap(), module)?;
                    return self.expand_function(f, &call.args, module);
                } else {
                    return Err(Diagnostic {
                        kind: DiagnosticKind::Error,
                        msg: format!("unknown module referenced `{}`", callee.first().unwrap()),
                        span: Span {
                            range: TextRange::new(0.into(), 0.into()),
                            file_id: self.file_id(self.entry),
                        },
                    });
                }
            }
            _ => todo!(),
        };
        Ok(val)
    }

    fn find_function_in_module(
        &self,
        name: &SmolStr,
        module: &'a Module,
    ) -> Result<&'a FnDecl, Diagnostic> {
        let f_idx = module.functions.iter().position(|f| f.name == *name);
        if let Some(idx) = f_idx {
            return Ok(&module.functions[idx]);
        } else {
            return Err(Diagnostic {
                kind: DiagnosticKind::Error,
                msg: format!("unknown function referenced `{}`", name),
                span: Span {
                    range: TextRange::new(0.into(), 0.into()),
                    file_id: self.file_id(self.entry),
                },
            });
        }
    }

    fn expand_function(&self, f: &FnDecl, args: &[ExprIdx], module: &Module) -> ValueResult {
        let params_len = f.params.0.len();
        let args_len = args.len();
        if params_len != args_len {
            return Err(Diagnostic {
                kind: DiagnosticKind::Error,
                msg: format!("incorrect number of arguments supplied to `{}`", f.name),
                span: Span {
                    range: TextRange::new(0.into(), 0.into()),
                    file_id: self.file_id(self.entry),
                },
            });
        }
        let substitutions: &[(SmolStr, ExprIdx)] = &f
            .params
            .0
            .iter()
            .zip(args)
            .map(|(param, arg)| (param.0.clone(), *arg))
            .collect::<Vec<_>>();
        self.lower_expr(f.body, module, Some(substitutions))
    }
}

pub fn lower(
    entry: &str,
    modules: &HashMap<FileId, Module>,
    expr_arena: &Arena<Expr>,
) -> ValueResult {
    // let entry_file_id = FileId(SmolStr::from(
    // std::fs::canonicalize(entry).unwrap().to_str().unwrap(),
    // ));
    let entry = std::fs::canonicalize(entry).unwrap();
    let ctx = LoweringCtx::new(&entry, modules, expr_arena);
    let res = ctx.lower();
    println!("{:#?}", res);
    res
}
