use std::collections::HashMap;

use super::*;

use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::ast::{Expr, ExprIdx};

// Catch expression, or the end of array
const ARRAY_COMMA_RECOVERY_SET: &[TokenKind] = &[
    TokenKind::IntLit,
    TokenKind::FloatLit,
    TokenKind::StringLit,
    TokenKind::LSquare,
    TokenKind::LCurly,
    TokenKind::RSquare,
];
const ARRAY_CLOSE_BRACKET_RECOVER_SET: &[TokenKind] = GLOBAL_RECOVERY_SET;

// Catch comma, expression, or end of record
const RECORD_KEY_RECOVERY_SET: &[TokenKind] = &[
    TokenKind::Comma,
    TokenKind::IntLit,
    TokenKind::FloatLit,
    TokenKind::StringLit,
    TokenKind::LSquare,
    TokenKind::LCurly,
    TokenKind::RCurly,
];
const RECORD_COLON_RECOVERY_SET: &[TokenKind] = &[
    TokenKind::IntLit,
    TokenKind::FloatLit,
    TokenKind::StringLit,
    TokenKind::LSquare,
    TokenKind::LCurly,
    TokenKind::RCurly,
];
const RECORD_COMMA_RECOVERY_SET: &[TokenKind] = &[TokenKind::Ident, TokenKind::RCurly];
const RECORD_CLOSE_BRACKET_RECOVER_SET: &[TokenKind] = GLOBAL_RECOVERY_SET;

impl<I: Iterator<Item = Token> + Clone> Parser<I> {
    pub(crate) fn expr(&mut self) -> ExprIdx {
        self.primary_expr()
        // self.binop_rhs(0, lhs);
    }

    fn primary_expr(&mut self) -> ExprIdx {
        let expr = if self.at(TokenKind::Ident) {
            self.ident_expr()
        } else if self.at(TokenKind::IntLit) {
            self.int_expr()
        } else if self.at(TokenKind::FloatLit) {
            self.float_lit()
        } else if self.at(TokenKind::StringLit) {
            self.string_expr()
        } else if self.at(TokenKind::LSquare) {
            self.array_expr()
        } else if self.at(TokenKind::LCurly) {
            self.record_expr()
        } else {
            self.error(format!("expected expression"));
            Expr::Error
        };
        self.expr_arena.alloc(expr)
    }

    fn ident_expr(&mut self) -> Expr {
        let text = self.expect(TokenKind::Ident, &[]).unwrap().text;
        Expr::Ident(text)
    }

    fn int_expr(&mut self) -> Expr {
        // remove '_' chars (they are used as separators for improving readability of large numbers)
        let text = self.peek().unwrap().text.replace("_", "");
        let prefix: String = text.chars().take(2).collect();
        let (text, radix): (String, u32) = match prefix.as_str() {
            "0b" => (text.chars().skip(2).collect(), 2),
            "0x" => (text.chars().skip(2).collect(), 16),
            _ => (text, 10),
        };
        let n = u64::from_str_radix(&text, radix);
        if let Some(err) = n.as_ref().err() {
            self.error(format!("could not parse integer: {}", err));
            Expr::Error
        } else {
            self.next();
            Expr::Int(n.unwrap())
        }
    }

    fn float_lit(&mut self) -> Expr {
        let n: Result<f64, _> = self.peek().unwrap().text.parse();
        if let Some(err) = n.as_ref().err() {
            self.error(format!("could not parse float: {}", err));
            Expr::Error
        } else {
            self.next();
            Expr::Float(n.unwrap())
        }
    }

    fn string_expr(&mut self) -> Expr {
        let content = self.expect(TokenKind::StringLit, &[]).unwrap().text;
        let content = &content[1..content.len() - 1];
        Expr::String(SmolStr::from(content))
    }

    fn array_expr(&mut self) -> Expr {
        let mut exprs = SmallVec::new();
        self.expect(TokenKind::LSquare, &[]); // we checked that this was an LSquare before entering this function, so no recover set needed
        while !self.at(TokenKind::RSquare) && !self.at_end() {
            let expr = self.expr();
            if !self.at(TokenKind::RSquare) {
                self.expect(TokenKind::Comma, ARRAY_COMMA_RECOVERY_SET);
            }
            exprs.push(expr);
        }
        self.expect(TokenKind::RSquare, ARRAY_CLOSE_BRACKET_RECOVER_SET);
        Expr::Array(exprs)
    }

    fn record_expr(&mut self) -> Expr {
        let mut fields = HashMap::new();
        self.expect(TokenKind::LCurly, &[]); // see comment in array_expr
        println!("{:?}", self.peek());
        while !self.at(TokenKind::RCurly) && !self.at_end() {
            let key = self
                .expect(TokenKind::Ident, RECORD_KEY_RECOVERY_SET)
                .unwrap()
                .text;
            if self.at(TokenKind::Comma) {
                self.next();
                fields.insert(key.clone(), self.expr_arena.alloc(Expr::Ident(key)));
                continue;
            }
            self.expect(TokenKind::Colon, RECORD_COLON_RECOVERY_SET);
            let value = self.expr();

            if !self.at(TokenKind::RCurly) {
                self.expect(TokenKind::Comma, RECORD_COMMA_RECOVERY_SET);
            }
            fields.insert(key, value);
        }
        self.expect(TokenKind::RCurly, RECORD_CLOSE_BRACKET_RECOVER_SET);
        Expr::Record(fields)
    }
}
