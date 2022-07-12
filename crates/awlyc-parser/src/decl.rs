use super::*;

const FN_KW_RECOVERY_SET: &[TokenKind] = &[TokenKind::Ident, TokenKind::LParen, TokenKind::RParen];
const FN_NAME_RECOVERY_SET: &[TokenKind] = &[TokenKind::LParen, TokenKind::RParen];
const FN_PARAMS_BEGIN_RECOVERY_SET: &[TokenKind] = &[TokenKind::RParen, TokenKind::Colon];
const FN_PARAMS_END_RECOVERY_SET: &[TokenKind] = &[TokenKind::Colon];
const FN_PARAMS_COMMA_RECOVERY_SET: &[TokenKind] = &[TokenKind::RParen, TokenKind::Colon];
const FN_PARAM_RECOVERY_SET: &[TokenKind] = &[TokenKind::Comma, TokenKind::RParen];
const FN_COLON_RECOVERY_SET: &[TokenKind] = GLOBAL_RECOVERY_SET;

impl<I: Iterator<Item = Token> + Clone> Parser<I> {
    pub(super) fn top_level_decl(&mut self) -> Option<FnDecl> {
        if self.at(TokenKind::Fn) {
            Some(self.fn_decl())
        } else {
            self.error(format!("expected top level declaration"));
            None
        }
    }

    fn fn_decl(&mut self) -> FnDecl {
        self.expect(TokenKind::Fn, FN_KW_RECOVERY_SET);
        let name = self
            .expect(TokenKind::Ident, FN_NAME_RECOVERY_SET)
            .unwrap()
            .text;
        let params = self.fn_params();
        self.expect(TokenKind::Colon, FN_COLON_RECOVERY_SET);
        let body = self.expr();
        FnDecl { name, params, body }
    }

    fn fn_params(&mut self) -> FnParams {
        let mut params = vec![];
        self.expect(TokenKind::LParen, FN_PARAMS_BEGIN_RECOVERY_SET);
        while !self.at(TokenKind::RParen) && !self.at_end() {
            params.push(self.fn_param());
            if !self.at(TokenKind::RParen) {
                self.expect(TokenKind::Comma, FN_PARAMS_COMMA_RECOVERY_SET);
                params.push(self.fn_param());
            }
        }
        self.expect(TokenKind::RParen, FN_PARAMS_END_RECOVERY_SET);
        FnParams(params)
    }

    fn fn_param(&mut self) -> FnParam {
        let name = self
            .expect(TokenKind::Ident, FN_PARAM_RECOVERY_SET)
            .unwrap()
            .text;
        FnParam(name)
    }
}
