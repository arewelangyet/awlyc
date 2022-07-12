use super::*;

impl<I: Iterator<Item = Token> + Clone> Parser<I> {
    pub(super) fn top_level_decl(&mut self) -> Option<FnDecl> {
        if self.at(TokenKind::Fn) {
            Some(self.fn_decl())
        } else {
            self.error(format!("expected top level declaration"));
            self.next();
            None
        }
    }

    fn fn_decl(&mut self) -> FnDecl {
        self.expect(TokenKind::Fn);
        let name = self.expect(TokenKind::Ident).expect("todo").text;
        let params = self.fn_params();
        self.expect(TokenKind::Colon);
        let body = self.expr();
        FnDecl { name, params, body }
    }

    fn fn_params(&mut self) -> FnParams {
        let mut params = vec![];
        self.expect(TokenKind::LParen);
        while !self.at(TokenKind::RParen) && !self.at_end() {
            params.push(self.fn_param());
            if !self.at(TokenKind::RParen) {
                if self.at(TokenKind::Comma) {
                    self.next();
                } else {
                    self.error(format!("expected either `,` or `)`"));
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen);
        FnParams(params)
    }

    fn fn_param(&mut self) -> FnParam {
        let name = self.expect(TokenKind::Ident).unwrap().text;
        FnParam(name)
    }
}
