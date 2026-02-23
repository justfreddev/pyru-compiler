use std::collections::HashMap;

use crate::{
    cfg::{ BlockId, FunctionCFG, Terminator },
    cfg_constprop::{ ConstEnv, ConstValue },
    expr::{ BinaryOp, Expr, LogicalOp, UnaryOp },
    stmt::Stmt,
    value::Value,
};

pub struct ConstFolding {
    out_map: HashMap<BlockId, ConstEnv>,
}

impl ConstFolding {
    pub fn new(out_map: HashMap<BlockId, ConstEnv>) -> Self {
        return Self { out_map };
    }

    pub fn fold_cfg(&self, cfg: &mut FunctionCFG) {
        for block in &mut cfg.blocks {
            let env = &self.out_map[&block.id];
            for stmt in &mut block.stmts {
                self.fold_stmt(stmt, env);
            }
            if
                let Some(Terminator::Branch { cond, then_block, else_block }) =
                    &mut block.terminator
            {
                self.fold_expr(cond, env);
                if let Expr::Literal(Value::Bool(b)) = cond {
                    block.terminator = match b {
                        true => Some(Terminator::Goto(*then_block)),
                        false => Some(Terminator::Goto(*else_block)),
                    };
                }
            }
        }
    }

    fn fold_stmt(&self, stmt: &mut Stmt, env: &ConstEnv) {
        match stmt {
            Stmt::Assign { value, .. } => {
                self.fold_expr(value, env);
            }
            | Stmt::Expression(value)
            | Stmt::Print(value)
            | Stmt::Var { name: _, initializer: Some(value) }
            | Stmt::Return(Some(value)) => {
                self.fold_expr(value, env);
            }

            Stmt::If { condition, then_branch, else_branch } => {
                self.fold_expr(condition, env);
                for s in then_branch {
                    self.fold_stmt(s, env);
                }
                if let Some(branch) = else_branch {
                    for s in branch {
                        self.fold_stmt(s, env);
                    }
                }
            }

            Stmt::While { condition, body } => {
                self.fold_expr(condition, env);
                for s in body {
                    self.fold_stmt(s, env);
                }
            }

            Stmt::For { initializer, condition, step, body } => {
                self.fold_stmt(initializer, env);
                self.fold_expr(condition, env);
                self.fold_stmt(step, env);
                for s in body {
                    self.fold_stmt(s, env);
                }
            }

            Stmt::Function { body, .. } => {
                for s in body {
                    self.fold_stmt(s, env);
                }
            }

            _ => {}
        }
    }

    fn fold_expr(&self, expr: &mut Expr, env: &ConstEnv) {
        match expr {
            Expr::Binary { left, operator, right } => {
                self.fold_expr(left, env);
                println!("{left} {operator} {right}");
                self.fold_expr(right, env);

                if let (Expr::Literal(l), Expr::Literal(r)) = (&**left, &**right) {
                    if let Some(res) = self.eval_binary(operator, l, r) {
                        *expr = Expr::Literal(res);
                    }
                }
            }

            Expr::Logical { operator, left, right } => {
                self.fold_expr(left, env);
                self.fold_expr(right, env);

                if let (Expr::Literal(l), Expr::Literal(r)) = (&**left, &**right) {
                    if let Some(res) = self.eval_logical(operator, l, r) {
                        *expr = Expr::Literal(res);
                    }
                }
            }

            Expr::Unary { operator, right } => {
                self.fold_expr(right, env);
                if let Expr::Literal(val) = &**right {
                    if let Some(res) = self.eval_unary(operator, val) {
                        *expr = Expr::Literal(res);
                    }
                }
            }

            Expr::Membership { left, right, .. } => {
                self.fold_expr(left, env);
                self.fold_expr(right, env);
            }

            Expr::Grouping(e) => {
                self.fold_expr(e, env);
                if let Expr::Literal(_) = &**e {
                    *expr = *e.clone();
                }
            }

            Expr::Call { callee, arguments } => {
                self.fold_expr(callee, env);
                for arg in arguments {
                    self.fold_expr(arg, env);
                }
            }

            Expr::List(items) => {
                for item in items {
                    self.fold_expr(item, env);
                }
            }

            Expr::Index { index, .. } => self.fold_expr(index, env),

            Expr::Slice { start, end, .. } => {
                if let Some(s) = start {
                    self.fold_expr(s, env);
                }
                if let Some(e) = end {
                    self.fold_expr(e, env);
                }
            }

            Expr::ListMethodCall { arguments, .. } => {
                for arg in arguments {
                    self.fold_expr(arg, env);
                }
            }

            Expr::Var(name) => {
                if let Some(ConstValue::Const(v)) = env.get(name) {
                    *expr = Expr::Literal(v.clone());
                }
            }

            Expr::Literal(_) => {}
        }
    }

    fn eval_binary(&self, op: &BinaryOp, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Num(a), Value::Num(b)) =>
                match op {
                    BinaryOp::Add => Some(Value::Num(a + b)),
                    BinaryOp::Sub => Some(Value::Num(a - b)),
                    BinaryOp::Mul => Some(Value::Num(a * b)),
                    BinaryOp::Div => Some(Value::Num(a / b)),
                    BinaryOp::Eq => Some(Value::Bool(a == b)),
                    BinaryOp::NotEq => Some(Value::Bool(a != b)),
                    BinaryOp::Greater => Some(Value::Bool(a > b)),
                    BinaryOp::GreaterEq => Some(Value::Bool(a >= b)),
                    BinaryOp::Less => Some(Value::Bool(a < b)),
                    BinaryOp::LessEq => Some(Value::Bool(a <= b)),
                }
            (Value::Bool(a), Value::Bool(b)) =>
                match op {
                    BinaryOp::Eq => Some(Value::Bool(a == b)),
                    BinaryOp::NotEq => Some(Value::Bool(a != b)),
                    BinaryOp::Greater => Some(Value::Bool(a > b)),
                    BinaryOp::GreaterEq => Some(Value::Bool(a >= b)),
                    BinaryOp::Less => Some(Value::Bool(a < b)),
                    BinaryOp::LessEq => Some(Value::Bool(a <= b)),
                    _ => None,
                }
            _ => None,
        }
    }

    fn eval_logical(&self, op: &LogicalOp, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => {
                match op {
                    LogicalOp::And => Some(Value::Bool(*a && *b)),
                    LogicalOp::Or => Some(Value::Bool(*a || *b)),
                }
            }
            _ => None,
        }
    }

    fn eval_unary(&self, op: &UnaryOp, val: &Value) -> Option<Value> {
        match (op, val) {
            (UnaryOp::Neg, Value::Num(n)) => Some(Value::Num(-n)),
            (UnaryOp::Not, Value::Bool(b)) => Some(Value::Bool(!b)),
            _ => None,
        }
    }
}
