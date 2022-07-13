use std::collections::HashMap;

use super::*;

use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::ast::{Binop, BinopKind, Call, Expr, ExprIdx, Spanned};

// Catch expression, or the end of array
const ARRAY_COMMA_RECOVERY_SET: &[TokenKind] = &[
    TokenKind::Ident,
    TokenKind::IntLit,
    TokenKind::FloatLit,
    TokenKind::StringLit,
    TokenKind::LSquare,
    TokenKind::LCurly,
    TokenKind::RSquare,
];
const ARRAY_CLOSE_BRACKET_RECOVERY_SET: &[TokenKind] = GLOBAL_RECOVERY_SET;

// Catch comma, expression, or end of record
const RECORD_KEY_RECOVERY_SET: &[TokenKind] = &[
    TokenKind::Comma,
    TokenKind::Ident,
    TokenKind::IntLit,
    TokenKind::FloatLit,
    TokenKind::StringLit,
    TokenKind::LSquare,
    TokenKind::LCurly,
    TokenKind::RCurly,
];
const RECORD_COLON_RECOVERY_SET: &[TokenKind] = &[
    TokenKind::Ident,
    TokenKind::IntLit,
    TokenKind::FloatLit,
    TokenKind::StringLit,
    TokenKind::LSquare,
    TokenKind::LCurly,
    TokenKind::RCurly,
];
const RECORD_COMMA_RECOVERY_SET: &[TokenKind] = &[TokenKind::Ident, TokenKind::RCurly];
const RECORD_CLOSE_BRACKET_RECOVERY_SET: &[TokenKind] = GLOBAL_RECOVERY_SET;
const CALL_OPEN_PAREN_RECOVERY_SET: &[TokenKind] = &[
    TokenKind::Ident,
    TokenKind::IntLit,
    TokenKind::FloatLit,
    TokenKind::StringLit,
    TokenKind::LSquare,
    TokenKind::LCurly,
    TokenKind::Comma,
    TokenKind::RParen,
];
const CALL_ARGS_COMMA_RECOVERY_SET: &[TokenKind] = CALL_OPEN_PAREN_RECOVERY_SET;
const CALL_CLOSE_PAREN_RECOVERY_SET: &[TokenKind] = &[TokenKind::RCurly, TokenKind::RSquare]; // not rlly sure what to put for the following two. maybe global recovery set but i think that'd skip way too much
const PATH_RECOVERY_SET: &[TokenKind] = &[TokenKind::RCurly, TokenKind::RSquare];

impl<'src, I: Iterator<Item = Token> + Clone> Parser<'src, I> {
    pub(crate) fn expr(&mut self) -> ExprIdx {
        let lhs = self.primary_expr();
        let expr = self.binop_rhs(0, lhs);
        self.postfix(expr)
    }

    fn primary_expr(&mut self) -> ExprIdx {
        let expr = if self.at(TokenKind::Ident) {
            self.path_expr()
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
            Spanned {
                inner: Expr::Error,
                span: Span {
                    range: self.peek_range(),
                    file_id: self.file_id.clone(),
                },
            }
        };
        self.expr_arena.alloc(expr)
    }

    fn binop_rhs(&mut self, expr_prec: i32, mut lhs: ExprIdx) -> ExprIdx {
        loop {
            let tok_prec = self.tok_prec();

            if tok_prec < expr_prec {
                return lhs;
            }

            let binop = self.peek().unwrap().kind;
            self.next();

            let mut rhs = self.primary_expr();

            let next_prec = self.tok_prec();
            if tok_prec < next_prec {
                rhs = self.binop_rhs(tok_prec + 1, rhs.clone());
            }
            let end = self.peek_range().end();

            lhs = self.expr_arena.alloc(Spanned {
                inner: Expr::Binop(Binop {
                    lhs,
                    op: match binop {
                        TokenKind::Plus => BinopKind::Add,
                        _ => unreachable!(),
                    },
                    rhs,
                }),
                span: Span {
                    range: TextRange::new(self.expr_arena[lhs].span.range.end(), end),
                    file_id: self.file_id.clone(),
                },
            });
        }
    }

    fn postfix(&mut self, expr: ExprIdx) -> ExprIdx {
        if self.at(TokenKind::LParen) {
            let call = self.call_expr(expr);
            self.expr_arena.alloc(call)
        } else {
            expr
        }
    }

    fn path_expr(&mut self) -> Spanned<Expr> {
        let start = self.peek_range().start();
        let mut parts = vec![];
        let part = self.expect(TokenKind::Ident, &[]).unwrap();
        parts.push(Spanned {
            inner: part.text,
            span: Span {
                range: part.range,
                file_id: self.file_id.clone(),
            },
        });

        while self.at(TokenKind::Period) {
            self.next();
            let part = self.expect(TokenKind::Ident, PATH_RECOVERY_SET).unwrap();
            parts.push(Spanned {
                inner: part.text,
                span: Span {
                    range: part.range,
                    file_id: self.file_id.clone(),
                },
            });
        }
        let end = self.peek_range().end();
        Spanned {
            inner: Expr::Path(parts),
            span: Span {
                range: TextRange::new(start, end),
                file_id: self.file_id.clone(),
            },
        }
    }

    fn int_expr(&mut self) -> Spanned<Expr> {
        let start = self.peek_range().start();
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
            let end = self.peek_range().end();
            Spanned {
                inner: Expr::Error,
                span: Span {
                    range: TextRange::new(start, end),
                    file_id: self.file_id.clone(),
                },
            }
        } else {
            self.next();
            let end = self.peek_range().end();
            Spanned {
                inner: Expr::Int(n.unwrap()),
                span: Span {
                    range: TextRange::new(start, end),
                    file_id: self.file_id.clone(),
                },
            }
        }
    }

    fn float_lit(&mut self) -> Spanned<Expr> {
        let start = self.peek_range().start();
        let n: Result<f64, _> = self.peek().unwrap().text.parse();
        if let Some(err) = n.as_ref().err() {
            self.error(format!("could not parse float: {}", err));
            let end = self.peek_range().end();
            Spanned {
                inner: Expr::Error,
                span: Span {
                    range: TextRange::new(start, end),
                    file_id: self.file_id.clone(),
                },
            }
        } else {
            self.next();
            let end = self.peek_range().end();
            Spanned {
                inner: Expr::Float(n.unwrap()),
                span: Span {
                    range: TextRange::new(start, end),
                    file_id: self.file_id.clone(),
                },
            }
        }
    }

    fn string_expr(&mut self) -> Spanned<Expr> {
        let start = self.peek_range().start();
        let content = self.expect(TokenKind::StringLit, &[]).unwrap().text;
        let content = &content[1..content.len() - 1];
        let end = self.peek_range().end();
        Spanned {
            inner: Expr::String(SmolStr::from(content)),
            span: Span {
                range: TextRange::new(start, end),
                file_id: self.file_id.clone(),
            },
        }
    }

    fn array_expr(&mut self) -> Spanned<Expr> {
        let start = self.peek_range().start();
        let mut exprs = SmallVec::new();
        self.expect(TokenKind::LSquare, &[]); // we checked that this was an LSquare before entering this function, so no recover set needed
        while !self.at(TokenKind::RSquare) && !self.at_end() {
            let expr = self.expr();
            if !self.at(TokenKind::RSquare) {
                self.expect(TokenKind::Comma, ARRAY_COMMA_RECOVERY_SET);
            }
            exprs.push(expr);
        }
        self.expect(TokenKind::RSquare, ARRAY_CLOSE_BRACKET_RECOVERY_SET);
        let end = self.peek_range().end();
        Spanned {
            inner: Expr::Array(exprs),
            span: Span {
                range: TextRange::new(start, end),
                file_id: self.file_id.clone(),
            },
        }
    }

    fn record_expr(&mut self) -> Spanned<Expr> {
        let start = self.peek_range().start();
        let mut fields: HashMap<SmolStr, ExprIdx> = HashMap::new();
        self.expect(TokenKind::LCurly, &[]); // see comment in array_expr
        while !self.at(TokenKind::RCurly) && !self.at_end() {
            let key = self
                .expect(TokenKind::Ident, RECORD_KEY_RECOVERY_SET)
                .unwrap();
            let key = Spanned {
                inner: key.text,
                span: Span {
                    range: key.range,
                    file_id: self.file_id.clone(),
                },
            };
            if self.at(TokenKind::Comma) {
                self.next();
                fields.insert(
                    key.inner.clone(),
                    self.expr_arena.alloc(Spanned {
                        inner: Expr::Path(vec![Spanned {
                            inner: key.inner.clone(),
                            span: key.span.clone(), // this literally makes no sense but for some reason i cant just do key.clone()...
                        }]),
                        span: key.span.clone(),
                    }),
                );
                continue;
            }
            self.expect(TokenKind::Colon, RECORD_COLON_RECOVERY_SET);
            let value = self.expr();

            if !self.at(TokenKind::RCurly) {
                self.expect(TokenKind::Comma, RECORD_COMMA_RECOVERY_SET);
            }
            fields.insert(key.inner, value);
        }
        self.expect(TokenKind::RCurly, RECORD_CLOSE_BRACKET_RECOVERY_SET);
        let end = self.peek_range().end();
        Spanned {
            inner: Expr::Record(fields),
            span: Span {
                range: TextRange::new(start, end),
                file_id: self.file_id.clone(),
            },
        }
    }

    fn call_expr(&mut self, callee: ExprIdx) -> Spanned<Expr> {
        let start = self.expr_arena[callee].span.range.start();
        let args_start = self.peek_range().start();
        let mut args = vec![];
        self.expect(TokenKind::LParen, CALL_OPEN_PAREN_RECOVERY_SET);
        while !self.at(TokenKind::RParen) && !self.at_end() {
            args.push(self.expr());
            if !self.at(TokenKind::RParen) {
                self.expect(TokenKind::Comma, CALL_ARGS_COMMA_RECOVERY_SET);
            }
        }
        self.expect(TokenKind::RParen, CALL_CLOSE_PAREN_RECOVERY_SET);
        let end = self.peek_range().end();
        Spanned {
            inner: Expr::Call(Call {
                callee,
                args: Spanned {
                    inner: args,
                    span: Span {
                        range: TextRange::new(args_start, end),
                        file_id: self.file_id.clone(),
                    },
                },
            }),
            span: Span {
                range: TextRange::new(start, end),
                file_id: self.file_id.clone(),
            },
        }
    }
}
