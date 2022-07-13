use smol_str::SmolStr;

use crate::ast::{ImportDecl, Spanned};

use super::*;

const IMPORT_NAME_RECOVERY_SET: &[TokenKind] = &[TokenKind::StringLit];
const IMPORT_PATH_RECOVERY_SET: &[TokenKind] = GLOBAL_RECOVERY_SET;
const FN_NAME_RECOVERY_SET: &[TokenKind] = &[TokenKind::LParen, TokenKind::RParen];
const FN_PARAMS_BEGIN_RECOVERY_SET: &[TokenKind] = &[TokenKind::RParen, TokenKind::Colon];
const FN_PARAMS_END_RECOVERY_SET: &[TokenKind] = &[TokenKind::Colon];
const FN_PARAMS_COMMA_RECOVERY_SET: &[TokenKind] = &[TokenKind::RParen, TokenKind::Colon];
const FN_PARAM_RECOVERY_SET: &[TokenKind] = &[TokenKind::Comma, TokenKind::RParen];
const FN_COLON_RECOVERY_SET: &[TokenKind] = GLOBAL_RECOVERY_SET;

impl<'src, I: Iterator<Item = Token> + Clone> Parser<'src, I> {
    pub(super) fn top_level_decls(&mut self) -> Module {
        let mut imports = vec![];
        let mut functions = vec![];
        let mut expr = None;
        while !self.at_end() {
            if self.at(TokenKind::Import) {
                imports.push(self.import_decl());
            } else if self.at(TokenKind::Fn) {
                functions.push(self.fn_decl());
            } else {
                if expr.is_some() {
                    self.error(format!("awlyc files can only contain one expression"));
                    continue;
                }
                expr = Some(self.expr());
            }
        }
        Module {
            imports,
            functions,
            expr,
        }
    }

    fn import_decl(&mut self) -> ImportDecl {
        self.expect(TokenKind::Import, &[]);
        let name = self
            .expect(TokenKind::Ident, IMPORT_NAME_RECOVERY_SET)
            .unwrap()
            .text
            .into();
        let path: SmolStr = self
            .expect(TokenKind::StringLit, IMPORT_PATH_RECOVERY_SET)
            .unwrap()
            .text
            .into();
        let path = &path[1..path.len() - 1]; // TODO: this should probably be handled during lexing
        ImportDecl {
            name,
            path: SmolStr::from(path),
        }
    }

    fn fn_decl(&mut self) -> FnDecl {
        self.expect(TokenKind::Fn, &[]);
        let name = self.expect(TokenKind::Ident, FN_NAME_RECOVERY_SET).unwrap();
        let name = Spanned {
            inner: name.text,
            span: Span {
                range: name.range,
                file_id: self.file_id.clone(),
            },
        };
        let params = self.fn_params();
        self.expect(TokenKind::Colon, FN_COLON_RECOVERY_SET);
        let body = self.expr();
        FnDecl { name, params, body }
    }

    fn fn_params(&mut self) -> Spanned<FnParams> {
        let mut params = vec![];
        let start = self.peek_range().start();
        self.expect(TokenKind::LParen, FN_PARAMS_BEGIN_RECOVERY_SET);
        while !self.at(TokenKind::RParen) && !self.at_end() {
            params.push(self.fn_param());
            if !self.at(TokenKind::RParen) {
                self.expect(TokenKind::Comma, FN_PARAMS_COMMA_RECOVERY_SET);
                params.push(self.fn_param());
            }
        }
        self.expect(TokenKind::RParen, FN_PARAMS_END_RECOVERY_SET);
        let end = self.peek_range().end();
        Spanned {
            inner: FnParams(params),
            span: Span {
                range: TextRange::new(start, end),
                file_id: self.file_id.clone(),
            },
        }
    }

    fn fn_param(&mut self) -> FnParam {
        let name = self
            .expect(TokenKind::Ident, FN_PARAM_RECOVERY_SET)
            .unwrap()
            .text;
        FnParam(name)
    }
}
