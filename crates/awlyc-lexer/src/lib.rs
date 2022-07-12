use logos::Logos;
use smol_str::SmolStr;
use text_size::{TextRange, TextSize};

pub fn lex<'src>(src: &'src str) -> Lexer<'src> {
    Lexer::new(src)
}

#[derive(Debug, Clone)]
pub struct Lexer<'src> {
    inner: logos::Lexer<'src, TokenKind>,
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.inner.next()?;
        let text = SmolStr::from(self.inner.slice());

        let range = {
            let range = self.inner.span();
            let start = TextSize::try_from(range.start).unwrap();
            let end = TextSize::try_from(range.end).unwrap();
            TextRange::new(start, end)
        };

        Some(Self::Item { kind, text, range })
    }
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            inner: TokenKind::lexer(src),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: SmolStr,
    pub range: TextRange,
}

#[derive(Logos, Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    #[token("fn")]
    Fn,
    #[token("import")]
    Import,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LCurly,
    #[token("}")]
    RCurly,
    #[token("[")]
    LSquare,
    #[token("]")]
    RSquare,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("\"")]
    DoubleQuote,
    #[token("+")]
    Plus,
    #[token(".")]
    Period,

    #[regex("[A-Za-z_][A-Za-z0-9_]*")]
    Ident,

    #[regex(r#""(?:[^"]|\\")*""#)]
    StringLit,

    #[regex("0x[0-9a-fA-F]+(_[0-9a-fA-F]+)*")]
    #[regex("0b[0-9]+(_[0-9]+)*")]
    #[regex("[0-9]+(_[0-9]+)*")]
    IntLit,

    #[regex(r"[0-9]+\.[0-9]+(_[0-9]+)*")]
    FloatLit,

    #[regex("#.*", logos::skip)]
    Comment,

    #[regex(r"/\*([^*]|\*+[^*/])*\*?")]
    #[regex(r"[ \n\r\t\f]+", logos::skip)]
    #[error]
    Error,
}

#[cfg(test)]
mod tests {
    use crate::lex;

    #[macro_export]
    #[cfg(test)]
    macro_rules! lex_str {
        ($name:ident, $src:literal) => {
            paste::paste! {
                    #[test]
                    fn [<test_lex_ $name>]() {
                        let tokens: Vec<_> = lex($src).collect();
                        let s = format!("{:#?}", tokens);
                        insta::assert_snapshot!(s);
                    }
            }
        };
    }

    lex_str!(basic_input, "testing 1.30 249 _hi02");
    lex_str!(keywords, "fn");
    lex_str!(separators, "(){}[],");
}
