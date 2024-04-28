use chumsky::{
    error::Simple,
    primitive::{end, just, one_of, take_until},
    recovery::skip_then_retry_until,
    text::{self, TextParser},
    Parser,
};
use derive_more::Display;

pub type Span = std::ops::Range<usize>;

#[derive(Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    Return,
    Struct,
    Op(char),
    Ident(String),
    Ctrl(char),
    Num(String),
}

impl Token {
    pub fn lexer() -> impl Parser<char, Vec<(Token, Span)>, Error = Simple<char>> {
        // A parser for numbers
        let num = text::int(10)
            .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
            .collect::<String>()
            .map(Token::Num);

        // A parser for operators
        let op = one_of("+-*/!=").map(|c| Token::Op(c));

        // A parser for control characters (delimiters, semicolons, etc.)
        let ctrl = one_of("()[]{};,").map(|c| Token::Ctrl(c));

        // parser for identifiers
        let ident = text::ident().map(|ident: String| match ident.as_str() {
            "return" => Token::Return,
            "struct" => Token::Struct,
            _ => Token::Ident(ident),
        });

        // combine parsers into single token parser
        let token = num
            .or(op)
            .or(ctrl)
            .or(ident)
            .recover_with(skip_then_retry_until([]));

        // create a parser for comments
        let comment = just("//").then(take_until(just('\n'))).padded();

        // combine all parsers with span and allow for comments
        token
            .map_with_span(|token, span| (token, span))
            .padded_by(comment.repeated())
            .padded()
            .repeated()
            .then_ignore(end())
    }
}
