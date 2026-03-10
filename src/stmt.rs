use std::fmt;

use crate::{ expr::Expr };

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Break,
    Continue,
    Decr(String),
    Expression(Expr),
    For {
        initializer: Box<Stmt>,
        condition: Expr,
        step: Box<Stmt>,
        body: Vec<Stmt>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        captures: Vec<String>,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    Incr(String),
    Print(Expr),
    Return(Option<Expr>),
    Var {
        name: String,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Assign { name, value } => write!(f, "Assign({name} = {value}"),
            Stmt::Break => write!(f, "Break"),
            Stmt::Continue => write!(f, "Continue"),
            Stmt::Decr(name) => write!(f, "{name}--"),
            Stmt::Expression(expression) => write!(f, "Expression({expression})"),
            Stmt::For { initializer, condition, step, body } => {
                return write!(f, "For({initializer:?} {condition} {step:?} {body:?})");
            }
            Stmt::Function { name, params, body, captures } => {
                return write!(f, "Function({name} {params:?} {body:?} {captures:?})");
            }
            Stmt::If { condition, then_branch, else_branch } => {
                if else_branch.is_some() {
                    return write!(
                        f,
                        "If({condition} {then_branch:?} {:?})",
                        else_branch.as_ref().unwrap()
                    );
                } else {
                    return write!(f, "If({condition} {then_branch:?})");
                }
            }
            Stmt::Incr(name) => write!(f, "{name}++"),
            Stmt::Print(expression) => write!(f, "Print({expression})"),
            Stmt::Return(value) => {
                return write!(f, "Return({value:?})");
            }
            Stmt::Var { name, initializer } => {
                if initializer.is_some() {
                    return write!(f, "Var({name}={})", initializer.as_ref().unwrap());
                } else {
                    return write!(f, "Var({name})");
                }
            }
            Stmt::While { condition, body } => {
                return write!(f, "While({condition} {body:?})");
            }
        }
    }
}
