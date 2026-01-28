use crate::{
    expr::{ BinaryOp, Expr, LogicalOp, UnaryOp },
    stmt::Stmt,
    token::{ Token, TokenKind },
    value::LiteralType,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        return Self { tokens, current: 0 };
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.declaration());
        }

        return statements;
    }

    fn declaration(&mut self) -> Stmt {
        if self.match_all(&[TokenKind::Def]) {
            return self.function();
        } else if self.match_all(&[TokenKind::Let]) {
            return self.var_declaration();
        } else {
            return self.statement();
        }
    }

    fn function(&mut self) -> Stmt {
        let name = self.consume(TokenKind::Identifier).span.literal.clone();
        self.consume(TokenKind::LParen);

        let mut params: Vec<String> = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                if params.len() >= 255 {
                    panic!("Too many parameters");
                }

                let parameter = self.consume(TokenKind::Identifier).span.literal.clone();
                params.push(parameter);
                if !self.match_all(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RParen);
        let body = self.body();

        return Stmt::Function { name, params, body };
    }

    fn var_declaration(&mut self) -> Stmt {
        let name = self.consume(TokenKind::Identifier).span.literal.clone();

        let initializer = if self.match_all(&[TokenKind::Equal]) {
            let expr = self.expression();
            Some(expr)
        } else {
            None
        };

        self.consume(TokenKind::Semicolon);

        return Stmt::Var { name, initializer };
    }

    fn statement(&mut self) -> Stmt {
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

    fn for_statement(&mut self) -> Stmt {
        let name = self.consume(TokenKind::Identifier).span.literal.clone();

        self.consume(TokenKind::In);

        // Could turn this into a range parsing
        let start = self.expression();
        self.consume(TokenKind::DotDot);
        let end = self.expression();

        let step = if self.match_all(&[TokenKind::Step]) {
            let value = self.expression();
            Box::new(Stmt::Assign {
                name: name.clone(),
                value: Box::new(Expr::Binary {
                    operator: BinaryOp::Add,
                    left: Box::new(Expr::Var { name: name.clone() }),
                    right: Box::new(value),
                }),
            })
        } else {
            Box::new(Stmt::Incr { name: name.clone() })
        };

        let initializer = Stmt::Var { name: name.clone(), initializer: Some(start) };

        let condition = Expr::Binary {
            operator: BinaryOp::Less,
            left: Box::new(Expr::Var { name }),
            right: Box::new(end),
        };

        let body = self.body();

        return Stmt::For {
            initializer: Box::new(initializer),
            condition,
            step,
            body,
        };
    }

    fn if_statement(&mut self) -> Stmt {
        let condition = self.expression();

        let then_branch = self.body();

        // let else_branch = if self.match_all(&[TokenKind::Else]) {
        //     if self.check(TokenKind::If) {
        //         Some(Box::new(self.statement()))
        //     } else {
        //         let result = Some(self.body());
        //         if self.match_all(&[TokenKind::Eof]) {
        //         } else {
        //             self.consume(TokenKind::Dedent);
        //         }
        //         result
        //     }
        // } else {
        //     None
        // };

        let else_branch = if self.match_all(&[TokenKind::Else]) { Some(self.body()) } else { None };

        return Stmt::If {
            condition,
            then_branch,
            else_branch,
        };
    }

    fn print_statement(&mut self) -> Stmt {
        self.consume(TokenKind::LParen);
        let expression = self.expression();
        self.consume(TokenKind::RParen);
        self.consume(TokenKind::Semicolon);
        return Stmt::Print { expression };
    }

    fn return_statement(&mut self) -> Stmt {
        let mut value = None;
        if !self.check(TokenKind::Semicolon) {
            value = Some(self.expression());
        }
        self.consume(TokenKind::Semicolon);

        return Stmt::Return { value };
    }

    fn while_statement(&mut self) -> Stmt {
        let condition = self.expression();

        let body = self.body();

        return Stmt::While { condition, body };
    }

    fn assignment_statement(&mut self) -> Stmt {
        let name = (
            match self.previous().kind {
                TokenKind::Identifier => &self.previous().span.literal,
                _ => panic!("Expected identifier"),
            }
        ).clone();

        if self.match_all(&[TokenKind::Incr]) {
            self.consume(TokenKind::Semicolon);
            return Stmt::Incr { name };
        }
        if self.match_all(&[TokenKind::Decr]) {
            self.consume(TokenKind::Semicolon);
            return Stmt::Decr { name };
        }

        if self.match_all(&[TokenKind::Equal]) {
            let value = self.expression();
            self.consume(TokenKind::Semicolon);
            return Stmt::Assign {
                name,
                value: Box::new(value),
            };
        }
        panic!("IT BREAKS HERE CHATGPT")
    }

    fn body(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        self.consume(TokenKind::Colon);
        self.consume(TokenKind::Indent);

        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            let stmt = self.declaration();
            statements.push(stmt);
        }
        if self.peek().kind == TokenKind::Eof {
        } else {
            self.consume(TokenKind::Dedent);
        }

        return statements;
    }

    fn expression(&mut self) -> Expr {
        return self.or();
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();

        while self.match_all(&[TokenKind::Or]) {
            let right = self.and();
            expr = Expr::Logical {
                operator: LogicalOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn and(&mut self) -> Expr {
        let mut expr = self.equality();

        while self.match_all(&[TokenKind::And]) {
            let right = self.equality();
            expr = Expr::Logical {
                operator: LogicalOp::And,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        // May need to be just Bang
        while self.match_all(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().kind;
            let right = self.comparison();
            expr = Expr::Binary {
                operator: match operator {
                    TokenKind::BangEqual => BinaryOp::NotEq,
                    TokenKind::EqualEqual => BinaryOp::Eq,
                    _ => panic!("Unexpected binary operator in equality"),
                },
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.membership();

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
            let right = self.membership();
            expr = Expr::Binary {
                operator: match operator {
                    TokenKind::Greater => BinaryOp::Greater,
                    TokenKind::GreaterEqual => BinaryOp::GreaterEq,
                    TokenKind::Less => BinaryOp::Less,
                    TokenKind::LessEqual => BinaryOp::LessEq,
                    TokenKind::BangEqual => BinaryOp::NotEq,
                    TokenKind::EqualEqual => BinaryOp::Eq,
                    _ => panic!("Unexpected token kind in comparison"),
                },
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn membership(&mut self) -> Expr {
        let mut expr = self.term();
        let mut not = false;

        if self.match_all(&[TokenKind::Not]) {
            not = true;
        }

        while self.match_all(&[TokenKind::In]) {
            let right = self.term();
            expr = Expr::Membership {
                left: Box::new(expr),
                not,
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.match_all(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous().kind;
            let right = self.factor();
            expr = Expr::Binary {
                operator: match operator {
                    TokenKind::Minus => BinaryOp::Sub,
                    TokenKind::Plus => BinaryOp::Add,
                    _ => panic!("Unexpected token kind in term expression"),
                },
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_all(&[TokenKind::FSlash, TokenKind::Asterisk]) {
            let operator = self.previous().kind;
            let right = self.unary();
            expr = Expr::Binary {
                operator: match operator {
                    TokenKind::FSlash => BinaryOp::Div,
                    TokenKind::Asterisk => BinaryOp::Mul,
                    _ => panic!("Unexpected token kind in factor expression"),
                },
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        return expr;
    }

    fn unary(&mut self) -> Expr {
        if self.match_all(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().kind;
            let right = self.unary();
            return Expr::Unary {
                operator: match operator {
                    TokenKind::Bang => UnaryOp::Not,
                    TokenKind::Minus => UnaryOp::Neg,
                    _ => panic!("Unexpected token kind in unary expression"),
                },
                right: Box::new(right),
            };
        }

        return self.call();
    }

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();

        loop {
            if self.match_all(&[TokenKind::LParen]) {
                expr = self.finish_call(expr);
            } else if self.match_all(&[TokenKind::Dot]) {
                let call = self.call();
                let name = match expr {
                    Expr::Var { name } => name,
                    _ => panic!("Can only call identifiers"),
                };
                return Expr::ListMethodCall { object: name, call: Box::new(call) };
            } else {
                break;
            }
        }

        return expr;
    }

    fn finish_call(&mut self, callee: Expr) -> Expr {
        let mut arguments: Vec<Expr> = Vec::new();

        if !self.check(TokenKind::RParen) {
            loop {
                if arguments.len() >= 255 {
                    panic!("Too many arguments");
                }
                let expr = self.expression();
                arguments.push(expr);
                if !self.match_all(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RParen);

        return Expr::Call {
            callee: Box::new(callee),
            arguments,
        };
    }

    fn primary(&mut self) -> Expr {
        if self.match_all(&[TokenKind::True]) {
            return Expr::Literal { value: LiteralType::True };
        }
        if self.match_all(&[TokenKind::False]) {
            return Expr::Literal { value: LiteralType::False };
        }
        if self.match_all(&[TokenKind::Null]) {
            return Expr::Literal { value: LiteralType::Null };
        }

        if self.match_all(&[TokenKind::Num, TokenKind::String]) {
            let value = self.previous().span.literal.clone();
            match self.previous().kind {
                TokenKind::String => {
                    return Expr::Literal { value: LiteralType::Str(value) };
                }
                TokenKind::Num => {
                    let n = match self.previous().span.literal.trim().parse() {
                        Ok(v) => v,
                        Err(_) => panic!("Unable to parse literal to float"),
                    };
                    return Expr::Literal { value: LiteralType::Num(n) };
                }
                _ => panic!("Expected a string or a number for a literal"),
            }
        }

        if self.match_all(&[TokenKind::Identifier]) {
            let name = self.previous().span.literal.clone();
            let expr = if self.match_all(&[TokenKind::LBrack]) {
                let mut start: Option<Box<Expr>> = None;
                let mut end: Option<Box<Expr>> = None;

                if self.peek().kind != TokenKind::Colon {
                    start = Some(Box::new(self.expression()));
                }
                start = if start.is_some() { Some(start.unwrap()) } else { None };

                if self.match_all(&[TokenKind::Colon]) {
                    if self.peek().kind != TokenKind::RBrack {
                        end = Some(Box::new(self.expression()));
                    }
                    end = if end.is_some() { Some(end.unwrap()) } else { None };
                } else {
                    self.consume(TokenKind::RBrack);
                    return Expr::Index { list: name, index: start.unwrap() };
                }

                self.consume(TokenKind::RBrack);

                Expr::Slice { list: name, start, end }
            } else {
                Expr::Var { name }
            };
            return expr;
        }

        if self.match_all(&[TokenKind::LParen]) {
            let expr = self.expression();
            self.consume(TokenKind::RParen);
            return Expr::Grouping { expression: Box::new(expr) };
        }

        if self.match_all(&[TokenKind::LBrack]) {
            let mut items: Vec<Expr> = Vec::new();
            loop {
                if self.match_all(&[TokenKind::RBrack]) {
                    break;
                }
                items.push(self.expression());
                if !self.match_all(&[TokenKind::Comma]) {
                    break;
                }
            }

            self.consume(TokenKind::RBrack);

            return Expr::List { items };
        }

        println!("{}", self.tokens[self.current]);
        panic!("Expected expression")
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();

        self.consume(TokenKind::Semicolon);

        return Stmt::Expression { expression: expr };
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

    fn consume(&mut self, kind: TokenKind) -> &Token {
        if self.check(kind) {
            return self.advance();
        }
        panic!("Couldn't consume token")
    }
}
