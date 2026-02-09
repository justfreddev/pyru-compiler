use crate::{
    error::{ CompileError, ParserError, Result },
    expr::{ BinaryOp, Expr, LogicalOp, UnaryOp },
    stmt::Stmt,
    token::{ Token, TokenKind },
    value::Value,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        return Self { tokens, current: 0 };
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        return Ok(statements);
    }

    fn declaration(&mut self) -> Result<Stmt> {
        if self.match_all(&[TokenKind::Def]) {
            return self.function();
        } else if self.match_all(&[TokenKind::Let]) {
            return self.var_declaration();
        } else {
            return self.statement();
        }
    }

    fn function(&mut self) -> Result<Stmt> {
        let name = self.consume(TokenKind::Identifier)?.span.literal.clone();
        self.consume(TokenKind::LParen)?;

        let mut params: Vec<String> = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                if params.len() >= 255 {
                    return Err(
                        CompileError::Parser(ParserError::TooManyParameters {
                            func_name: name,
                        })
                    );
                }

                let parameter = self.consume(TokenKind::Identifier)?.span.literal.clone();
                params.push(parameter);
                if !self.match_all(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RParen)?;
        let body = self.body()?;

        return Ok(Stmt::Function { name, params, body, captures: vec![] });
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(TokenKind::Identifier)?.span.literal.clone();

        let initializer = if self.match_all(&[TokenKind::Equal]) {
            let expr = self.expression()?;
            Some(expr)
        } else {
            None
        };

        self.consume(TokenKind::Semicolon)?;

        return Ok(Stmt::Var { name, initializer });
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_all(&[TokenKind::Break]) {
            self.consume(TokenKind::Semicolon)?;
            return Ok(Stmt::Break);
        }
        if self.match_all(&[TokenKind::Continue]) {
            self.consume(TokenKind::Semicolon)?;
            return Ok(Stmt::Continue);
        }
        if self.match_all(&[TokenKind::For]) {
            return self.for_statement();
        }
        if self.match_all(&[TokenKind::If]) {
            return self.if_statement();
        }
        if self.match_all(&[TokenKind::Print]) {
            return self.print_statement();
        }
        if self.match_all(&[TokenKind::Return]) {
            return self.return_statement();
        }
        if self.match_all(&[TokenKind::While]) {
            return self.while_statement();
        }
        if self.check(TokenKind::Identifier) {
            if
                vec![TokenKind::Equal, TokenKind::Incr, TokenKind::Decr].contains(
                    &self.tokens[self.current + 1].kind
                )
            {
                self.advance();
                return self.assignment_statement();
            }
        }

        return self.expression_statement();
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        let name = self.consume(TokenKind::Identifier)?.span.literal.clone();

        self.consume(TokenKind::In)?;

        // Could turn this into a range parsing
        let start = self.expression()?;
        self.consume(TokenKind::DotDot)?;
        let end = self.expression()?;

        let step = if self.match_all(&[TokenKind::Step]) {
            let value = self.expression()?;
            Box::new(Stmt::Assign {
                name: name.clone(),
                value: Box::new(Expr::Binary {
                    operator: BinaryOp::Add,
                    left: Box::new(Expr::Var(name.clone())),
                    right: Box::new(value),
                }),
            })
        } else {
            Box::new(Stmt::Incr(name.clone()))
        };

        let initializer = Stmt::Var { name: name.clone(), initializer: Some(start) };

        let condition = Expr::Binary {
            operator: BinaryOp::Less,
            left: Box::new(Expr::Var(name)),
            right: Box::new(end),
        };

        let body = self.body()?;

        return Ok(Stmt::For {
            initializer: Box::new(initializer),
            condition,
            step,
            body,
        });
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        let condition = self.expression()?;

        let then_branch = self.body()?;

        let else_branch = if self.match_all(&[TokenKind::Else]) {
            if self.match_all(&[TokenKind::If]) {
                Some(vec![self.if_statement()?])
            } else {
                Some(self.body()?)
            }
        } else {
            None
        };

        return Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        });
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenKind::LParen)?;
        let expression = self.expression()?;
        self.consume(TokenKind::RParen)?;
        self.consume(TokenKind::Semicolon)?;
        return Ok(Stmt::Print(expression));
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        let mut value = None;
        if !self.check(TokenKind::Semicolon) {
            value = Some(self.expression()?);
        }
        self.consume(TokenKind::Semicolon)?;

        return Ok(Stmt::Return(value));
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        let condition = self.expression()?;

        let body = self.body()?;

        return Ok(Stmt::While { condition, body });
    }

    fn assignment_statement(&mut self) -> Result<Stmt> {
        let name = (
            match self.previous().kind {
                TokenKind::Identifier => &self.previous().span.literal,
                _ => {
                    let prev = self.previous();
                    return Err(
                        CompileError::Parser(ParserError::ExpectedIdentifier {
                            got: prev.kind,
                            line: prev.line,
                        })
                    );
                }
            }
        ).clone();

        if self.match_all(&[TokenKind::Incr]) {
            self.consume(TokenKind::Semicolon)?;
            return Ok(Stmt::Incr(name));
        }
        if self.match_all(&[TokenKind::Decr]) {
            self.consume(TokenKind::Semicolon)?;
            return Ok(Stmt::Decr(name));
        }

        if self.match_all(&[TokenKind::Equal]) {
            let value = self.expression()?;
            self.consume(TokenKind::Semicolon)?;
            return Ok(Stmt::Assign {
                name,
                value: Box::new(value),
            });
        }

        return Err(CompileError::Parser(ParserError::UnknownStatement));
    }

    fn body(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        self.consume(TokenKind::Colon)?;
        self.consume(TokenKind::Indent)?;

        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            let stmt = self.declaration()?;
            statements.push(stmt);
        }
        if self.peek().kind == TokenKind::Eof {
        } else {
            self.consume(TokenKind::Dedent)?;
        }

        return Ok(statements);
    }

    fn expression(&mut self) -> Result<Expr> {
        return self.or();
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.match_all(&[TokenKind::Or]) {
            let right = self.and()?;
            expr = Expr::Logical {
                operator: LogicalOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.match_all(&[TokenKind::And]) {
            let right = self.equality()?;
            expr = Expr::Logical {
                operator: LogicalOp::And,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.match_all(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().kind;
            let right = self.comparison()?;
            expr = Expr::Binary {
                operator: match operator {
                    TokenKind::BangEqual => BinaryOp::NotEq,
                    TokenKind::EqualEqual => BinaryOp::Eq,
                    _ => {
                        let prev = self.previous();
                        return Err(
                            CompileError::Parser(ParserError::ExpectedBinaryOp {
                                got: prev.kind,
                                line: prev.line,
                            })
                        );
                    }
                },
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.membership()?;

        while
            self.match_all(
                &[
                    TokenKind::Greater,
                    TokenKind::GreaterEqual,
                    TokenKind::Less,
                    TokenKind::LessEqual,
                    TokenKind::BangEqual,
                    TokenKind::EqualEqual,
                ]
            )
        {
            let operator = self.previous().kind;
            let right = self.membership()?;
            expr = Expr::Binary {
                operator: match operator {
                    TokenKind::Greater => BinaryOp::Greater,
                    TokenKind::GreaterEqual => BinaryOp::GreaterEq,
                    TokenKind::Less => BinaryOp::Less,
                    TokenKind::LessEqual => BinaryOp::LessEq,
                    TokenKind::BangEqual => BinaryOp::NotEq,
                    TokenKind::EqualEqual => BinaryOp::Eq,
                    _ => {
                        let prev = self.previous();
                        return Err(
                            CompileError::Parser(ParserError::ExpectedComparisonOp {
                                got: prev.kind,
                                line: prev.line,
                            })
                        );
                    }
                },
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn membership(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;
        let mut not = false;

        if self.match_all(&[TokenKind::Not]) {
            not = true;
        }

        while self.match_all(&[TokenKind::In]) {
            let right = self.term()?;
            expr = Expr::Membership {
                left: Box::new(expr),
                not,
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.match_all(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous().kind;
            let right = self.factor()?;
            expr = Expr::Binary {
                operator: match operator {
                    TokenKind::Minus => BinaryOp::Sub,
                    TokenKind::Plus => BinaryOp::Add,
                    _ => {
                        let prev = self.previous();
                        return Err(
                            CompileError::Parser(ParserError::ExpectedTermOp {
                                got: prev.kind,
                                line: prev.line,
                            })
                        );
                    }
                },
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.match_all(&[TokenKind::FSlash, TokenKind::Asterisk]) {
            let operator = self.previous().kind;
            let right = self.unary()?;
            expr = Expr::Binary {
                operator: match operator {
                    TokenKind::FSlash => BinaryOp::Div,
                    TokenKind::Asterisk => BinaryOp::Mul,
                    _ => {
                        let prev = self.previous();
                        return Err(
                            CompileError::Parser(ParserError::ExpectedFactorOp {
                                got: prev.kind,
                                line: prev.line,
                            })
                        );
                    }
                },
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_all(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().kind;
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator: match operator {
                    TokenKind::Bang => UnaryOp::Not,
                    TokenKind::Minus => UnaryOp::Neg,
                    _ => {
                        let prev = self.previous();
                        return Err(
                            CompileError::Parser(ParserError::ExpectedUnaryOp {
                                got: prev.kind,
                                line: prev.line,
                            })
                        );
                    }
                },
                right: Box::new(right),
            });
        }

        return Ok(self.call()?);
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_all(&[TokenKind::LParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_all(&[TokenKind::Dot]) {
                let object = match expr {
                    Expr::Var(ref name) => name.clone(),
                    _ => {
                        return Err(CompileError::Parser(ParserError::MethodCallNotOnList));
                    }
                };

                let method_name = self.consume(TokenKind::Identifier)?.span.literal.clone();

                if self.match_all(&[TokenKind::LParen]) {
                    let mut args = Vec::new();
                    if !self.check(TokenKind::RParen) {
                        loop {
                            args.push(self.expression()?);
                            if !self.match_all(&[TokenKind::Comma]) {
                                break;
                            }
                        }
                    }
                    self.consume(TokenKind::RParen)?;

                    expr = Expr::ListMethodCall {
                        object: object,
                        method_name,
                        arguments: args,
                    };
                } else {
                    let curr = &self.tokens[self.current];
                    return Err(
                        CompileError::Parser(ParserError::ListMethodCallMissingParen {
                            got: curr.kind,
                            line: curr.line,
                        })
                    );
                }
            } else {
                break;
            }
        }

        return Ok(expr);
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments: Vec<Expr> = Vec::new();

        if !self.check(TokenKind::RParen) {
            loop {
                if arguments.len() >= 255 {
                    if let Expr::Var(func_name) = callee {
                        return Err(CompileError::Parser(ParserError::TooManyArgs { func_name }));
                    } else {
                        return Err(
                            CompileError::Parser(ParserError::TooManyArgs {
                                func_name: "Unknown".into(),
                            })
                        );
                    }
                }
                let expr = self.expression()?;
                arguments.push(expr);
                if !self.match_all(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RParen)?;

        return Ok(Expr::Call {
            callee: Box::new(callee),
            arguments,
        });
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.match_all(&[TokenKind::True]) {
            return Ok(Expr::Literal(Value::Bool(true)));
        }
        if self.match_all(&[TokenKind::False]) {
            return Ok(Expr::Literal(Value::Bool(false)));
        }
        if self.match_all(&[TokenKind::Null]) {
            return Ok(Expr::Literal(Value::Null));
        }

        if self.match_all(&[TokenKind::Num, TokenKind::String]) {
            let value = self.previous().span.literal.clone();
            match self.previous().kind {
                TokenKind::String => {
                    return Ok(Expr::Literal(Value::Str(value)));
                }
                TokenKind::Num => {
                    let n = match self.previous().span.literal.trim().parse() {
                        Ok(v) => v,
                        Err(_) => {
                            let prev = self.previous();
                            return Err(
                                CompileError::Parser(ParserError::UnableToParseLiteralToFloat {
                                    literal: prev.span.literal.clone(),
                                    line: prev.line,
                                })
                            );
                        }
                    };
                    return Ok(Expr::Literal(Value::Num(n)));
                }
                _ => {
                    let prev = self.previous();
                    return Err(
                        CompileError::Parser(ParserError::UnexpectedLiteral {
                            got: prev.kind,
                            line: prev.line,
                        })
                    );
                }
            }
        }

        if self.match_all(&[TokenKind::Identifier]) {
            let name = self.previous().span.literal.clone();
            let expr = if self.match_all(&[TokenKind::LBrack]) {
                let mut start: Option<Box<Expr>> = None;
                let mut end: Option<Box<Expr>> = None;

                if self.peek().kind != TokenKind::Colon {
                    start = Some(Box::new(self.expression()?));
                }
                start = if start.is_some() { Some(start.unwrap()) } else { None };

                if self.match_all(&[TokenKind::Colon]) {
                    if self.peek().kind != TokenKind::RBrack {
                        end = Some(Box::new(self.expression()?));
                    }
                    end = if end.is_some() { Some(end.unwrap()) } else { None };
                } else {
                    self.consume(TokenKind::RBrack)?;
                    return Ok(Expr::Index { list: name, index: start.unwrap() });
                }

                self.consume(TokenKind::RBrack)?;

                Expr::Slice { list: name, start, end }
            } else {
                Expr::Var(name)
            };
            return Ok(expr);
        }

        if self.match_all(&[TokenKind::LParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RParen)?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        if self.match_all(&[TokenKind::LBrack]) {
            let mut items: Vec<Expr> = Vec::new();
            loop {
                if self.match_all(&[TokenKind::RBrack]) {
                    break;
                }
                items.push(self.expression()?);
                if !self.match_all(&[TokenKind::Comma]) {
                    break;
                }
            }

            self.consume(TokenKind::RBrack)?;

            return Ok(Expr::List(items));
        }

        let curr = &self.tokens[self.current];
        return Err(
            CompileError::Parser(ParserError::ExpectedExpression {
                got: curr.kind,
                line: curr.line,
            })
        );
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;

        self.consume(TokenKind::Semicolon)?;

        return Ok(Stmt::Expression(expr));
    }

    fn match_all(&mut self, types: &[TokenKind]) -> bool {
        for token_kind in types {
            if self.check(*token_kind) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&mut self, token_kind: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }

        return self.peek().kind == token_kind;
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        return self.previous();
    }

    fn previous(&self) -> &Token {
        return &self.tokens[self.current - 1];
    }

    fn peek(&self) -> &Token {
        return &self.tokens[self.current];
    }

    fn is_at_end(&self) -> bool {
        return self.peek().kind == TokenKind::Eof;
    }

    fn consume(&mut self, kind: TokenKind) -> Result<&Token> {
        if self.check(kind) {
            return Ok(self.advance());
        }
        let curr = &self.tokens[self.current];
        return Err(
            CompileError::Parser(ParserError::CouldntConsumeToken {
                expected: kind,
                got: curr.kind,
                line: curr.line,
            })
        );
    }
}
