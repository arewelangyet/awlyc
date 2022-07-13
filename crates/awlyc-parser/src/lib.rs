use std::iter::Peekable;

use ast::{Expr, ExprIdx, FnDecl, FnParam, FnParams, ImportDecl};
use awlyc_error::{Diagnostic, DiagnosticKind, FileId, Span};
use awlyc_lexer::{lex, Token, TokenKind};
use la_arena::Arena;
use text_size::TextRange;

mod ast;
mod decl;
mod expr;

#[derive(Debug)]
pub struct Module {
    pub imports: Vec<ImportDecl>,
    pub functions: Vec<FnDecl>,
    pub expr: ExprIdx,
}

const GLOBAL_RECOVERY_SET: &[TokenKind] = &[TokenKind::Fn];

struct Parser<'src, I: Iterator<Item = Token> + Clone> {
    tokens: Peekable<I>,
    errors: Vec<Diagnostic>,
    /// Token kinds we expect to find are stored here to be displayed in error diagnostics
    expected_tokens: Vec<TokenKind>,
    expr_arena: Arena<Expr>,
    file_id: FileId,
    src: &'src str,
}

impl<'src, I: Iterator<Item = Token> + Clone> Parser<'src, I> {
    pub(crate) fn new(tokens: Peekable<I>, src: &'src str, file_id: FileId) -> Self {
        Self {
            tokens,
            errors: vec![],
            expected_tokens: vec![],
            expr_arena: Arena::default(),
            src,
            file_id,
        }
    }

    fn next(&mut self) -> Option<Token> {
        self.expected_tokens.clear();
        self.tokens.next()
    }

    fn expect(&mut self, kind: TokenKind, recovery_set: &[TokenKind]) -> Option<Token> {
        let tok = self.peek().cloned();
        if self.at(kind) {
            self.next();
        } else {
            self.error(format!(
                "expected `{}`",
                self.expected_tokens
                    .iter()
                    .map(|kind| format!("{:?}", kind)) // TODO: impl Display
                    .reduce(|s, kind| format!("{}, {}", s, kind))
                    .unwrap()
            ));

            while !self.at_set(recovery_set) && !self.at_end() {
                self.next();
            }
        }
        tok
    }

    fn error(&mut self, msg: String) {
        let range = if let Some(Token { range, .. }) = self.peek() {
            *range
        } else {
            if let Some(range) = self.tokens.clone().last().map(|Token { range, .. }| range) {
                range
            } else {
                let len = self.src.len();
                if len == 0 {
                    TextRange::new(0.into(), 0.into())
                } else {
                    TextRange::new(((len - 1) as u32).into(), (len as u32).into())
                }
            }
        };

        self.errors.push(Diagnostic {
            kind: DiagnosticKind::Error,
            msg,
            span: Span {
                range,
                file_id: self.file_id.clone(),
            },
        });

        self.next();
    }

    #[inline]
    fn at_end(&mut self) -> bool {
        self.tokens.peek().is_none()
    }

    #[inline]
    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    #[inline]
    fn peek_kind(&mut self) -> Option<TokenKind> {
        self.peek().map(|Token { kind, .. }| *kind)
    }

    fn at(&mut self, kind: TokenKind) -> bool {
        self.expected_tokens.push(kind);
        self.peek_kind() == Some(kind)
    }

    #[inline]
    fn at_set(&mut self, set: &[TokenKind]) -> bool {
        self.peek().map_or(false, |k| set.contains(&k.kind))
    }

    fn tok_prec(&mut self) -> i32 {
        if let Some(tok) = self.peek() {
            match tok.kind {
                _ => -1,
            }
        } else {
            -1
        }
    }

    pub(crate) fn parse(&mut self) -> Module {
        self.top_level_decls()
    }
}

pub fn parse(src: &str, file_id: FileId) -> (Module, Arena<Expr>, Vec<Diagnostic>) {
    let tokens = lex(src).peekable();
    let mut parser = Parser::new(tokens, src, file_id);
    let module = parser.parse();
    (module, parser.expr_arena, parser.errors)
}

#[cfg(test)]
mod tests {
    #[macro_export]
    #[cfg(test)]
    macro_rules! parse_success {
        ($name:ident, $src:literal) => {
            paste::paste! {
                    #[test]
                    fn [<test_parse_ $name>]() {
                        let (decls, expr_arena, errors) = crate::parse($src, awlyc_error::FileId(smol_str::SmolStr::from("main")));
                        let s = format!("{:#?}\n{:#?}\n{:#?}", expr_arena, decls, errors);
                        insta::assert_snapshot!(s);
                    }
            }
        };
    }

    parse_success!(
        basic_fn_decl,
        r#"fn host(foo, bar): "https://arewelangyet.com""#
    );
}
