use std::fmt;

use crate::token::{ TokenKind };
use crate::value::LiteralType;

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Eq,
    NotEq,
}

#[derive(Debug)]
pub enum LogicalOp {
    Or,
    And,
}

#[derive(Debug)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug)]
pub enum Expr {
    Alteration {
        name: String,
        alteration_type: TokenKind,
    },
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    List {
        items: Vec<Expr>,
    },
    ListMethodCall {
        object: String,
        call: Box<Expr>,
    },
    Literal {
        value: LiteralType,
    },
    Logical {
        operator: LogicalOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Membership {
        left: Box<Expr>,
        not: bool,
        right: Box<Expr>,
    },
    Splice {
        list: String,
        is_splice: bool,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    Unary {
        operator: UnaryOp,
        right: Box<Expr>,
    },
    Var {
        name: String,
    },
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            BinaryOp::Add => write!(f, "Add"),
            BinaryOp::Sub => write!(f, "Sub"),
            BinaryOp::Mul => write!(f, "Mul"),
            BinaryOp::Div => write!(f, "Div"),
            BinaryOp::Less => write!(f, "Less"),
            BinaryOp::LessEq => write!(f, "LessEq"),
            BinaryOp::Greater => write!(f, "Greater"),
            BinaryOp::GreaterEq => write!(f, "GreaterEq"),
            BinaryOp::Eq => write!(f, "Eq"),
            BinaryOp::NotEq => write!(f, "NotEq"),
        };
    }
}

impl fmt::Display for LogicalOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            LogicalOp::And => write!(f, "And"),
            LogicalOp::Or => write!(f, "Or"),
        };
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            UnaryOp::Not => write!(f, "Not"),
            UnaryOp::Neg => write!(f, "Neg"),
        };
    }
}

impl fmt::Display for Expr {
    /// Implements the `Display` trait for `Expr` to provide a string representation
    /// of each expression variant.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Expr::Alteration { name, alteration_type } => {
                write!(f, "Alteration({name} {alteration_type})")
            }
            Expr::Assign { name, value } => write!(f, "Assign({name} = {value}"),
            Expr::Binary { left, operator, right } => {
                write!(f, "Binary({left} {operator} {right})")
            }
            Expr::Call { callee, arguments } => write!(f, "Call({callee} {arguments:?})"),
            Expr::Grouping { expression } => write!(f, "Grouping({expression})"),
            Expr::List { items } => write!(f, "[{items:?}]"),
            Expr::ListMethodCall { object, call } => write!(f, "{object}.{call}"),
            Expr::Literal { value } => write!(f, "{value}"),
            Expr::Logical { left, operator, right } => {
                write!(f, "Logical({left} {operator} {right})")
            }
            Expr::Membership { left, not, right } => {
                if *not {
                    return write!(f, "{left} not in {right}");
                }
                write!(f, "{left} in {right}")
            }
            Expr::Splice { list, is_splice: _, start, end } => {
                write!(f, "{list}[{start:?}:{end:?}]")
            }
            Expr::Unary { operator, right } => write!(f, "Unary({operator} {right})"),
            Expr::Var { name } => write!(f, "Var({name})"),
        };
    }
}
