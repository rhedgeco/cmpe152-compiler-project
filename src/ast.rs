use chumsky::{
    error::Simple,
    primitive::{filter_map, just},
    recovery,
    recursive::recursive,
    Error, Parser,
};

use crate::Token;

#[derive(Debug)]
pub struct Ast {
    pub defs: Vec<Definition>,
}

impl Ast {
    pub fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
        Definition::parser()
            .repeated()
            .at_least(1)
            .map(|defs| Self { defs })
    }
}

#[derive(Debug)]
pub enum Definition {
    Struct {
        name: String,
        params: Vec<Param>,
    },
    Func {
        name: String,
        params: Vec<Param>,
        ret: String,
        body: Vec<Statement>,
    },
}

impl Definition {
    fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
        let r#struct = just(Token::Struct)
            .ignore_then(parse_ident())
            .then(
                Param::parser()
                    .then_ignore(just(Token::Ctrl(';')))
                    .repeated()
                    .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}')))
                    .recover_with(recovery::nested_delimiters(
                        Token::Ctrl('{'),
                        Token::Ctrl('}'),
                        [],
                        |_| Vec::new(),
                    )),
            )
            .then_ignore(just(Token::Ctrl(';')))
            .map(|(name, params)| Definition::Struct { name, params });

        let func = parse_ident()
            .then(parse_ident())
            .then(
                Param::parser()
                    .separated_by(just(Token::Ctrl(',')))
                    .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                    .recover_with(recovery::nested_delimiters(
                        Token::Ctrl('('),
                        Token::Ctrl(')'),
                        [],
                        |_| Vec::new(),
                    )),
            )
            .then(
                Statement::parser()
                    .repeated()
                    .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}')))
                    .recover_with(recovery::nested_delimiters(
                        Token::Ctrl('{'),
                        Token::Ctrl('}'),
                        [],
                        |_| Vec::new(),
                    )),
            )
            .map(|(((ret, name), params), body)| Self::Func {
                name,
                params,
                ret,
                body,
            });

        r#struct.or(func)
    }
}

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub ty: String,
}

impl Param {
    fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
        parse_ident()
            .then(parse_ident())
            .map(|(ty, name)| Self { name, ty })
    }
}

#[derive(Debug)]
pub enum Statement {
    Invalid,
    Return(Box<Expr>),
    Assign {
        ty: String,
        name: String,
        expr: Box<Expr>,
    },
}

impl Statement {
    fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
        let ret = just(Token::Return)
            .ignore_then(Expr::parser())
            .then_ignore(just(Token::Ctrl(';')))
            .map(|expr| Self::Return(Box::new(expr)));

        let assign = parse_ident()
            .then(parse_ident())
            .then_ignore(just(Token::Op('=')))
            .then(Expr::parser())
            .then_ignore(just(Token::Ctrl(';')))
            .map(|((ty, name), expr)| Self::Assign {
                ty,
                name,
                expr: Box::new(expr),
            });

        ret.or(assign)
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Err,
    Int(u32),
    Neg(Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Var(String),
    Call { name: String, params: Vec<Expr> },
}

impl Expr {
    fn parser() -> impl Parser<Token, Self, Error = Simple<Token>> {
        recursive(|expr| {
            let int = filter_map(|span, token| match token {
                Token::Num(value) => Ok(Expr::Int(value.parse::<u32>().unwrap())),
                _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
            });

            let call = parse_ident()
                .then(
                    expr.clone()
                        .separated_by(just(Token::Ctrl(',')))
                        .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                        .recover_with(recovery::nested_delimiters(
                            Token::Ctrl('('),
                            Token::Ctrl(')'),
                            [],
                            |_| Vec::new(),
                        )),
                )
                .map(|(name, params)| Self::Call { name, params });

            let variable = parse_ident().map(|name| Self::Var(name));

            let atom = int
                .or(expr.delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')'))))
                .or(call)
                .or(variable);

            let unary = just(Token::Op('-'))
                .repeated()
                .then(atom)
                .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

            let product = unary
                .clone()
                .then(
                    just(Token::Op('*'))
                        .to(Expr::Mul as fn(_, _) -> _)
                        .or(just(Token::Op('/')).to(Expr::Div as fn(_, _) -> _))
                        .then(unary)
                        .repeated(),
                )
                .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

            let sum = product
                .clone()
                .then(
                    just(Token::Op('+'))
                        .to(Expr::Add as fn(_, _) -> _)
                        .or(just(Token::Op('-')).to(Expr::Sub as fn(_, _) -> _))
                        .then(product)
                        .repeated(),
                )
                .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

            sum
        })
    }
}

fn parse_ident() -> impl Parser<Token, String, Error = Simple<Token>> + Clone {
    filter_map(|span, token| match token {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
    })
}
