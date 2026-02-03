use std::collections::HashMap;

use crate::{ expr::{ BinaryOp, Expr, LogicalOp, UnaryOp }, stmt::Stmt, value::Value };

type ConstEnv = HashMap<String, Option<Value>>;

pub struct ConstPropagator {
    env: ConstEnv,
}

impl ConstPropagator {
    pub fn new() -> Self {
        Self { env: HashMap::new() }
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

    pub fn propagate_stmt(&mut self, stmt: &Stmt) -> Stmt {
        match stmt {
            Stmt::Assign { name, value } => {
                let new_value = self.propagate_expr(&*value);

                match &new_value {
                    Expr::Literal(v) => {
                        self.env.insert(name.clone(), Some(v.clone()));
                    }
                    _ => {
                        self.env.insert(name.clone(), None);
                    }
                }

                Stmt::Assign { name: name.clone(), value: Box::new(new_value) }
            }

            Stmt::Decr(name) => {
                match self.env.get(name) {
                    Some(Some(Value::Num(n))) => {
                        self.env.insert(name.clone(), Some(Value::Num(n - 1.0)));
                    }
                    _ => {
                        self.env.insert(name.clone(), None);
                    }
                }

                Stmt::Decr(name.clone())
            }

            Stmt::Expression(expr) => {
                let folded_expr = self.propagate_expr(expr);

                Stmt::Expression(folded_expr)
            }

            Stmt::For { initializer, condition, step, body } => {
                let folded_initializer = self.propagate_stmt(&*initializer);
                let folded_condition = self.propagate_expr(condition);

                let mut before_loop_env = self.env.clone();

                let mut temp_env = self.env.clone();
                std::mem::swap(&mut self.env, &mut temp_env);

                let folded_step = self.propagate_stmt(&*step);
                let folded_body = body
                    .iter()
                    .map(|s| self.propagate_stmt(s))
                    .collect();

                for (name, value) in self.env.iter() {
                    if before_loop_env.get(name) != Some(value) {
                        before_loop_env.insert(name.clone(), None);
                    }
                }

                self.env = before_loop_env;

                Stmt::For {
                    initializer: Box::new(folded_initializer),
                    condition: folded_condition,
                    step: Box::new(folded_step),
                    body: folded_body,
                }
            }

            Stmt::Function { name, params, body } => {
                let before_func_env = self.env.clone();

                self.env = HashMap::new();

                for param in params {
                    self.env.insert(param.clone(), None);
                }

                let folded_body = body
                    .iter()
                    .map(|s| self.propagate_stmt(s))
                    .collect();

                self.env = before_func_env;

                Stmt::Function { name: name.clone(), params: params.clone(), body: folded_body }
            }

            Stmt::If { condition, then_branch, else_branch } => {
                let folded_condition = self.propagate_expr(&condition);

                let before_if_env = self.env.clone();

                let mut then_env = self.env.clone();
                self.env = then_env;
                let folded_then = then_branch
                    .into_iter()
                    .map(|s| self.propagate_stmt(s))
                    .collect();
                then_env = self.env.clone();

                let folded_else = if let Some(else_branch) = else_branch {
                    let mut else_env = self.env.clone();
                    self.env = else_env;
                    let folded_else_branch: Vec<Stmt> = else_branch
                        .iter()
                        .map(|s| self.propagate_stmt(s))
                        .collect();
                    else_env = self.env.clone();
                    self.env = before_if_env.clone();
                    Some(folded_else_branch)
                } else {
                    self.env = before_if_env.clone();
                    None
                };

                if let Some(_) = &else_branch {
                    for (name, _) in before_if_env.iter() {
                        let then_val = then_env.get(name);
                        let else_val = self.env.get(name);
                        let merged = match (then_val, else_val) {
                            (Some(Some(v1)), Some(Some(v2))) if v1 == v2 => Some(v1.clone()),
                            _ => None,
                        };
                        self.env.insert(name.clone(), merged);
                    }
                } else {
                    for (name, then_val) in then_env.iter() {
                        if then_val.is_some() {
                            self.env.insert(name.clone(), None);
                        }
                    }
                }

                Stmt::If {
                    condition: folded_condition,
                    then_branch: folded_then,
                    else_branch: folded_else,
                }
            }

            Stmt::Incr(name) => {
                match self.env.get(name) {
                    Some(Some(Value::Num(n))) => {
                        self.env.insert(name.clone(), Some(Value::Num(n + 1.0)));
                    }
                    _ => {
                        self.env.insert(name.clone(), None);
                    }
                }

                Stmt::Incr(name.clone())
            }

            Stmt::Print(expr) => {
                let folded_expr = self.propagate_expr(expr);
                Stmt::Print(folded_expr)
            }

            Stmt::Return(v) => {
                if let Some(expr) = v {
                    let folded_expr = self.propagate_expr(expr);
                    Stmt::Return(Some(folded_expr))
                } else {
                    Stmt::Return(None)
                }
            }

            Stmt::Var { name, initializer } => {
                let folded_initializer = initializer.as_ref().map(|e| self.propagate_expr(e));

                let constant = if let Some(Expr::Literal(v)) = folded_initializer.as_ref() {
                    Some(v.clone())
                } else {
                    None
                };

                self.env.insert(name.clone(), constant);

                Stmt::Var {
                    name: name.clone(),
                    initializer: folded_initializer,
                }
            }

            Stmt::While { condition, body } => {
                let folded_condition = self.propagate_expr(condition);

                let mut before_loop_env = self.env.clone();

                let mut temp_env = self.env.clone();
                std::mem::swap(&mut self.env, &mut temp_env);

                let folded_body: Vec<Stmt> = body
                    .iter()
                    .map(|s| self.propagate_stmt(s))
                    .collect();

                for (name, value) in self.env.iter() {
                    if before_loop_env.get(name) != Some(value) {
                        before_loop_env.insert(name.clone(), None);
                    }
                }

                self.env = before_loop_env;

                Stmt::While {
                    condition: folded_condition,
                    body: folded_body,
                }
            }

            _ => stmt.clone(),
        }
    }

    fn propagate_expr(&mut self, expr: &Expr) -> Expr {
        match expr {
            Expr::Binary { left, operator, right } => {
                let left = Box::new(self.propagate_expr(left));
                let right = Box::new(self.propagate_expr(right));

                match (&*left, &*right) {
                    (Expr::Literal(Value::Num(l)), Expr::Literal(Value::Num(r))) => {
                        Expr::Literal(self.eval_nums(l, r, operator))
                    }
                    _ => Expr::Binary { left, operator: *operator, right },
                }
            }

            Expr::Grouping(expr) => {
                let folded = Box::new(self.propagate_expr(expr));
                if let Expr::Literal(_) = *folded {
                    Expr::Grouping(folded)
                } else {
                    Expr::Grouping(folded)
                }
            }

            Expr::Index { list, index } => {
                let folded_index = Box::new(self.propagate_expr(index));
                Expr::Index { list: list.clone(), index: folded_index }
            }

            Expr::Logical { operator, left, right } => {
                let left = Box::new(self.propagate_expr(left));
                let right = Box::new(self.propagate_expr(right));

                match (&*left, &*right) {
                    (Expr::Literal(Value::Bool(l)), Expr::Literal(Value::Bool(r))) => {
                        Expr::Literal(self.eval_bools(l, r, operator))
                    }
                    _ => Expr::Logical { left, operator: *operator, right },
                }
            }

            Expr::List(items) => {
                let folded_items: Vec<Expr> = items
                    .into_iter()
                    .map(|e: &Expr| self.propagate_expr(e))
                    .collect();

                Expr::List(folded_items)
            }

            Expr::Membership { left, not, right } => {
                let left = Box::new(self.propagate_expr(left));
                let right = Box::new(self.propagate_expr(right));

                if let (Expr::Literal(value), Expr::Literal(Value::List(l))) = (&*left, &*right) {
                    if l.borrow().contains(value) {
                        Expr::Literal(Value::Bool(!*not))
                    } else {
                        Expr::Literal(Value::Bool(*not))
                    }
                } else {
                    Expr::Membership { left, not: *not, right }
                }
            }

            Expr::Unary { operator, right } => {
                let right = Box::new(self.propagate_expr(right));

                if let Expr::Literal(v) = &*right {
                    match (operator, v) {
                        (UnaryOp::Neg, Value::Num(n)) => Expr::Literal(Value::Num(-n)),
                        (UnaryOp::Not, Value::Bool(b)) => Expr::Literal(Value::Bool(!b)),
                        _ => Expr::Unary { operator: *operator, right },
                    }
                } else {
                    Expr::Unary { operator: *operator, right }
                }
            }

            Expr::Var(name) => {
                if let Some(Some(val)) = self.env.get(name) {
                    Expr::Literal(val.clone())
                } else {
                    Expr::Var(name.clone())
                }
            }

            _ => expr.clone(),
        }
    }
}
