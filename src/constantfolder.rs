use std::collections::HashMap;

use crate::{
    expr::{ BinaryOp, Expr, LogicalOp, UnaryOp },
    semanticanalyser::SymbolInfo,
    stmt::Stmt,
    value::Value,
};

pub struct ConstantFolder<'a> {
    symbols: &'a HashMap<String, SymbolInfo>,
}

impl<'a> ConstantFolder<'a> {
    pub fn new(symbols: &'a HashMap<String, SymbolInfo>) -> Self {
        Self { symbols }
    }

    pub fn fold_stmt(&self, stmt: &Stmt) -> Stmt {
        match stmt {
            Stmt::Assign { name, value } => {
                let folded_value = self.fold_expr(&*value);
                Stmt::Assign { name: name.clone(), value: Box::new(folded_value) }
            }

            Stmt::Expression(expression) => { Stmt::Expression(self.fold_expr(&expression)) }

            Stmt::For { initializer, condition, step, body } => {
                let folded_initializer = Box::new(self.fold_stmt(&*initializer));
                let folded_condition = self.fold_expr(&condition);
                let folded_step = Box::new(self.fold_stmt(&*step));
                let folded_body: Vec<Stmt> = body
                    .into_iter()
                    .map(|s| self.fold_stmt(s))
                    .collect();

                Stmt::For {
                    initializer: folded_initializer,
                    condition: folded_condition,
                    step: folded_step,
                    body: folded_body,
                }
            }

            Stmt::Function { name, params, body } => {
                let folded_body: Vec<Stmt> = body
                    .into_iter()
                    .map(|s| self.fold_stmt(s))
                    .collect();

                Stmt::Function { name: name.clone(), params: params.clone(), body: folded_body }
            }

            Stmt::If { condition, then_branch, else_branch } => {
                let folded_condition = self.fold_expr(&condition);
                let folded_then = then_branch
                    .into_iter()
                    .map(|s| self.fold_stmt(s))
                    .collect();
                let folded_else = else_branch.clone().map(|b|
                    b
                        .into_iter()
                        .map(|s| self.fold_stmt(&s))
                        .collect()
                );
                Stmt::If {
                    condition: folded_condition,
                    then_branch: folded_then,
                    else_branch: folded_else,
                }
            }

            Stmt::Print(expression) => Stmt::Print(self.fold_expr(&expression)),

            Stmt::Return(value) => {
                if let Some(expr) = value {
                    Stmt::Return(Some(self.fold_expr(&expr)))
                } else {
                    Stmt::Return(None)
                }
            }

            Stmt::Var { name, initializer } => {
                if let Some(expr) = initializer {
                    Stmt::Var { name: name.clone(), initializer: Some(self.fold_expr(&expr)) }
                } else {
                    Stmt::Var { name: name.clone(), initializer: None }
                }
            }

            Stmt::While { condition, body } => {
                let folded_condition = self.fold_expr(&condition);
                let folded_body: Vec<Stmt> = body
                    .into_iter()
                    .map(|s| self.fold_stmt(s))
                    .collect();

                Stmt::While { condition: folded_condition, body: folded_body }
            }

            other => other.clone(),
        }
    }

    pub fn fold_expr(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::Binary { left, operator, right } => {
                let left = self.fold_expr(&*left);
                let right = self.fold_expr(&*right);
                if
                    let (Expr::Literal(Value::Num(l)), Expr::Literal(Value::Num(r))) = (
                        &left,
                        &right,
                    )
                {
                    Expr::Literal(self.eval_nums(l, r, &operator))
                } else {
                    Expr::Binary {
                        left: Box::new(left),
                        operator: *operator,
                        right: Box::new(right),
                    }
                }
            }

            Expr::Grouping(expression) => {
                let folded = self.fold_expr(&*expression);
                if let Expr::Literal(ref value) = folded {
                    Expr::Literal(value.clone())
                } else {
                    Expr::Grouping(Box::new(folded))
                }
            }

            Expr::Index { list, index } => {
                let folded_index = Box::new(self.fold_expr(&*index));
                Expr::Index { list: list.clone(), index: folded_index }
            }

            Expr::List(items) => {
                let folded_items: Vec<Expr> = items
                    .into_iter()
                    .map(|e: &Expr| self.fold_expr(e))
                    .collect();

                Expr::List(folded_items)
            }

            Expr::Logical { operator, left, right } => {
                let left = self.fold_expr(&*left);
                let right = self.fold_expr(&*right);
                if
                    let (Expr::Literal(Value::Bool(l)), Expr::Literal(Value::Bool(r))) = (
                        &left,
                        &right,
                    )
                {
                    Expr::Literal(self.eval_bools(l, r, &operator))
                } else {
                    Expr::Logical {
                        operator: *operator,
                        left: Box::new(left),
                        right: Box::new(right),
                    }
                }
            }

            Expr::Membership { left, not, right } => {
                let folded_left = Box::new(self.fold_expr(&*left));
                let folded_right = Box::new(self.fold_expr(&*right));
                if
                    let (Expr::Literal(value), Expr::Literal(Value::List(l))) = (
                        &*folded_left,
                        &*folded_right,
                    )
                {
                    if l.borrow().contains(value) {
                        Expr::Literal(Value::Bool(!*not))
                    } else {
                        Expr::Literal(Value::Bool(*not))
                    }
                } else {
                    Expr::Membership { left: folded_left, not: *not, right: folded_right }
                }
            }

            Expr::Unary { operator, right } => {
                let val = self.fold_expr(&*right);
                match operator {
                    UnaryOp::Neg => {
                        if let Expr::Literal(Value::Num(n)) = val {
                            Expr::Literal(Value::Num(-n))
                        } else {
                            val
                        }
                    }
                    UnaryOp::Not => {
                        if let Expr::Literal(Value::Bool(b)) = val {
                            Expr::Literal(Value::Bool(!b))
                        } else {
                            val
                        }
                    }
                }
            }

            Expr::Var(name) => {
                if let Some(sym) = self.symbols.get(name) {
                    if let Some(val) = &sym.constant {
                        return Expr::Literal(val.clone());
                    }
                }
                Expr::Var(name.clone())
            }

            expr => expr.clone(),
        }
    }

    fn eval_nums(&self, l: &f64, r: &f64, op: &BinaryOp) -> Value {
        match op {
            BinaryOp::Add => Value::Num(l + r),
            BinaryOp::Sub => Value::Num(l - r),
            BinaryOp::Mul => Value::Num(l * r),
            BinaryOp::Div => Value::Num(l / r),
            BinaryOp::Eq => Value::Bool(l == r),
            BinaryOp::NotEq => Value::Bool(l != r),
            BinaryOp::Less => Value::Bool(l < r),
            BinaryOp::LessEq => Value::Bool(l <= r),
            BinaryOp::Greater => Value::Bool(l > r),
            BinaryOp::GreaterEq => Value::Bool(l >= r),
        }
    }

    fn eval_bools(&self, l: &bool, r: &bool, op: &LogicalOp) -> Value {
        match op {
            LogicalOp::And => Value::Bool(*l && *r),
            LogicalOp::Or => Value::Bool(*l || *r),
        }
    }
}
