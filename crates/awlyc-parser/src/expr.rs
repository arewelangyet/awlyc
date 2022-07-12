use smol_str::SmolStr;

use crate::ast::{Expr, ExprIdx};

use super::*;

impl<I: Iterator<Item = Token> + Clone> Parser<I> {
    pub(crate) fn expr(&mut self) -> ExprIdx {
        self.primary_expr()
        // self.binop_rhs(0, lhs);
    }

    fn primary_expr(&mut self) -> ExprIdx {
        let expr = if self.at(TokenKind::String) {
            self.string_expr()
        } else {
            Expr::Error
        };
        self.expr_arena.alloc(expr)
    }

    fn string_expr(&mut self) -> Expr {
        let content = self.expect(TokenKind::String, &[]).unwrap().text;
        let content = &content[1..content.len() - 1];
        Expr::String(SmolStr::from(content))
    }
}
