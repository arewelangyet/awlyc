use ast::{Expr, FnDecl, FnParam, FnParams};
use awlyc_error::{Diagnostic, DiagnosticKind, FileId, Span};
use awlyc_lexer::{lex, Token, TokenKind};
use la_arena::Arena;

mod ast;
mod decl;
mod expr;

struct Parser<'a> {
    tokens: &'a [Token],
    errors: Vec<Diagnostic>,
    /// Token kinds we expect to find are stored here to be displayed in error diagnostics
    expected_tokens: Vec<TokenKind>,
    expr_arena: Arena<Expr>,
    idx: usize,
    file_id: FileId,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(tokens: &'a [Token], file_id: FileId) -> Self {
        Self {
            tokens,
            errors: vec![],
            expected_tokens: vec![],
            expr_arena: Arena::default(),
            idx: 0,
            file_id,
        }
    }

    fn next(&mut self) -> Option<&Token> {
        self.expected_tokens.clear();
        self.idx += 1;
        self.tokens.get(self.idx)
    }

    fn expect(&mut self, kind: TokenKind) -> Option<Token> {
        let tok = self.cur_tok().cloned();
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
        }
        tok
    }

    fn error(&mut self, msg: String) {
        let range = if let Some(Token { range, .. }) = self.cur_tok() {
            *range
        } else {
            // if we've even started parsing it means that there are tokens, so this unwrap is safe
            self.tokens.last().unwrap().range
        };

        self.errors.push(Diagnostic {
            kind: DiagnosticKind::Error,
            msg,
            span: Span {
                range,
                file_id: self.file_id.clone(), // TODO: store file ids
            },
        });
    }

    #[inline]
    fn at_end(&self) -> bool {
        !(self.idx < self.tokens.len())
    }

    #[inline]
    fn cur_tok(&self) -> Option<&Token> {
        self.tokens.get(self.idx)
    }

    #[inline]
    fn cur_kind(&self) -> Option<TokenKind> {
        self.tokens.get(self.idx).map(|Token { kind, .. }| *kind)
    }

    fn at(&mut self, kind: TokenKind) -> bool {
        self.expected_tokens.push(kind);
        self.cur_kind() == Some(kind)
    }

    pub(crate) fn parse(&mut self) -> Vec<FnDecl> {
        let mut decls = vec![];
        while self.idx < self.tokens.len() {
            if let Some(decl) = self.top_level_decl() {
                decls.push(decl);
            }
        }
        decls
    }
}

pub fn parse(src: &str, file_id: FileId) -> (Vec<FnDecl>, Arena<Expr>, Vec<Diagnostic>) {
    let tokens: Vec<_> = lex(src).collect();
    let mut parser = Parser::new(&tokens, file_id);
    let decls = parser.parse();
    (decls, parser.expr_arena, parser.errors)
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
