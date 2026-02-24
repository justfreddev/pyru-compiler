use std::collections::{ HashMap, HashSet };

use crate::{
    cfg::{ BasicBlock, BlockId, FunctionCFG },
    liveliness,
    expr::{ BinaryOp, Expr, LogicalOp, UnaryOp },
    stmt::Stmt,
    value::Value,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ConstValue {
    Undef,
    Const(Value),
    Nac, // Not a constant
}

pub type ConstEnv = HashMap<String, ConstValue>;

pub struct ConstProp;

impl ConstProp {
    pub fn new() -> Self {
        return Self;
    }

    pub fn rewrite_with_constants(
        &self,
        cfg: &mut FunctionCFG,
        in_map: &HashMap<BlockId, ConstEnv>
    ) {
        for block in &mut cfg.blocks {
            let env = &in_map[&block.id];

            for stmt in &mut block.stmts {
                self.rewrite_stmt(stmt, env);
            }
        }
    }

    fn rewrite_stmt(&self, stmt: &mut Stmt, env: &ConstEnv) {
        match stmt {
            Stmt::Expression(value) | Stmt::Print(value) => self.rewrite_expr(value, env),

            Stmt::Assign { value, .. } => self.rewrite_expr(value, env),

            Stmt::Var { initializer: Some(value), .. } => self.rewrite_expr(value, env),

            Stmt::Return(Some(value)) => self.rewrite_expr(value, env),

            Stmt::If { condition, then_branch, else_branch } => {
                self.rewrite_expr(condition, env);
                for s in then_branch {
                    self.rewrite_stmt(s, env);
                }
                if let Some(else_branch) = else_branch {
                    for s in else_branch {
                        self.rewrite_stmt(s, env);
                    }
                }
            }

            Stmt::While { condition, body } | Stmt::For { condition, body, .. } => {
                self.rewrite_expr(condition, env);
                for s in body {
                    self.rewrite_stmt(s, env);
                }
            }
            _ => {}
        }
    }

    fn rewrite_expr(&self, expr: &mut Expr, env: &ConstEnv) {
        match expr {
            Expr::Var(name) => {
                if let Some(ConstValue::Const(v)) = env.get(name) {
                    *expr = Expr::Literal(v.clone());
                }
            }

            | Expr::Binary { left, right, .. }
            | Expr::Logical { left, right, .. }
            | Expr::Membership { left, right, .. } => {
                self.rewrite_expr(left, env);
                self.rewrite_expr(right, env);
            }

            Expr::Unary { right, .. } | Expr::Grouping(right) => {
                self.rewrite_expr(right, env);
            }

            Expr::Call { callee, arguments } => {
                self.rewrite_expr(callee, env);
                for arg in arguments {
                    self.rewrite_expr(arg, env);
                }
            }

            Expr::List(items) => {
                for item in items {
                    self.rewrite_expr(item, env);
                }
            }

            Expr::Index { index, .. } => self.rewrite_expr(index, env),

            Expr::Slice { start, end, .. } => {
                if let Some(s) = start {
                    self.rewrite_expr(s, env);
                }
                if let Some(e) = end {
                    self.rewrite_expr(e, env);
                }
            }

            Expr::ListMethodCall { arguments, .. } => {
                for arg in arguments {
                    self.rewrite_expr(arg, env);
                }
            }

            Expr::Literal(_) => {}
        }
    }

    pub fn compute_constants(
        &self,
        cfg: &FunctionCFG
    ) -> (HashMap<BlockId, ConstEnv>, HashMap<BlockId, ConstEnv>) {
        let mut in_map = HashMap::new();
        let mut out_map = HashMap::new();

        for block in &cfg.blocks {
            in_map.insert(block.id, HashMap::new());
            out_map.insert(block.id, HashMap::new());
        }

        loop {
            let mut changed = false;

            for block in &cfg.blocks {
                let id = block.id;

                let mut in_prime = HashMap::new();
                for pred in cfg.compute_block_predecessors(id) {
                    let pred_out = &out_map[&pred];
                    in_prime = if in_prime.is_empty() {
                        pred_out.clone()
                    } else {
                        self.meet_env(&in_prime, pred_out)
                    };
                }

                let out_prime = self.transfer_block(block, &in_prime);

                if in_prime != in_map[&id] || out_prime != out_map[&id] {
                    changed = true;
                }

                in_map.insert(id, in_prime);
                out_map.insert(id, out_prime);
            }

            if !changed {
                break;
            }
        }

        return (in_map, out_map);
    }

    fn transfer_block(&self, block: &BasicBlock, in_env: &ConstEnv) -> ConstEnv {
        let mut out_env = in_env.clone();

        for stmt in &block.stmts {
            match stmt {
                Stmt::Assign { name, value } => {
                    let val = self.eval_expr(value, &out_env);
                    out_env.insert(name.clone(), val);
                }

                Stmt::Var { name, initializer } => {
                    if let Some(expr) = initializer {
                        let val = self.eval_expr(expr, &out_env);
                        out_env.insert(name.clone(), val);
                    } else {
                        out_env.insert(name.clone(), ConstValue::Undef);
                    }
                }

                Stmt::Expression(expr) => {
                    if liveliness::expr_has_side_effects(expr) {
                        for v in out_env.values_mut() {
                            *v = ConstValue::Nac;
                        }
                    }
                }

                _ => {}
            }
        }

        return out_env;
    }

    fn eval_expr(&self, expr: &Expr, env: &ConstEnv) -> ConstValue {
        match expr {
            Expr::Literal(v) => ConstValue::Const(v.clone()),

            Expr::Var(name) => { env.get(name).cloned().unwrap_or(ConstValue::Undef) }

            Expr::Binary { left, operator, right } => {
                let l = self.eval_expr(left, env);
                let r = self.eval_expr(right, env);

                match (l, r) {
                    (ConstValue::Const(Value::Num(a)), ConstValue::Const(Value::Num(b))) => {
                        match operator {
                            BinaryOp::Add => ConstValue::Const(Value::Num(a + b)),
                            BinaryOp::Sub => ConstValue::Const(Value::Num(a - b)),
                            BinaryOp::Mul => ConstValue::Const(Value::Num(a * b)),
                            BinaryOp::Div => ConstValue::Const(Value::Num(a / b)),
                            BinaryOp::Eq => ConstValue::Const(Value::Bool(a == b)),
                            BinaryOp::NotEq => ConstValue::Const(Value::Bool(a != b)),
                            BinaryOp::Greater => ConstValue::Const(Value::Bool(a > b)),
                            BinaryOp::GreaterEq => ConstValue::Const(Value::Bool(a >= b)),
                            BinaryOp::Less => ConstValue::Const(Value::Bool(a < b)),
                            BinaryOp::LessEq => ConstValue::Const(Value::Bool(a <= b)),
                        }
                    }
                    (ConstValue::Const(Value::Str(a)), ConstValue::Const(Value::Str(b))) => {
                        if let BinaryOp::Add = operator {
                            ConstValue::Const(Value::Str(a + &b))
                        } else {
                            ConstValue::Nac
                        }
                    }
                    (ConstValue::Const(Value::Bool(a)), ConstValue::Const(Value::Bool(b))) => {
                        match operator {
                            BinaryOp::Eq => ConstValue::Const(Value::Bool(a == b)),
                            BinaryOp::NotEq => ConstValue::Const(Value::Bool(a != b)),
                            BinaryOp::Greater => ConstValue::Const(Value::Bool(a > b)),
                            BinaryOp::GreaterEq => ConstValue::Const(Value::Bool(a >= b)),
                            BinaryOp::Less => ConstValue::Const(Value::Bool(a < b)),
                            BinaryOp::LessEq => ConstValue::Const(Value::Bool(a <= b)),
                            _ => ConstValue::Nac,
                        }
                    }
                    _ => ConstValue::Nac,
                }
            }

            Expr::Logical { operator, left, right } => {
                let l = self.eval_expr(left, env);
                let r = self.eval_expr(right, env);

                match (l, r) {
                    (ConstValue::Const(Value::Bool(a)), ConstValue::Const(Value::Bool(b))) => {
                        match operator {
                            LogicalOp::And => ConstValue::Const(Value::Bool(a && b)),
                            LogicalOp::Or => ConstValue::Const(Value::Bool(a || b)),
                        }
                    }
                    _ => ConstValue::Nac,
                }
            }

            Expr::Unary { operator, right } => {
                let v = self.eval_expr(right, env);

                match v {
                    ConstValue::Const(Value::Num(n)) => {
                        match operator {
                            UnaryOp::Neg => ConstValue::Const(Value::Num(-n)),
                            _ => ConstValue::Nac,
                        }
                    }
                    ConstValue::Const(Value::Bool(b)) => {
                        match operator {
                            UnaryOp::Not => ConstValue::Const(Value::Bool(!b)),
                            _ => ConstValue::Nac,
                        }
                    }
                    _ => ConstValue::Nac,
                }
            }

            _ => ConstValue::Nac,
        }
    }

    fn meet_env(&self, a: &ConstEnv, b: &ConstEnv) -> ConstEnv {
        let mut result = HashMap::new();

        let keys: HashSet<&String> = a.keys().chain(b.keys()).collect();

        for key in keys {
            let v1 = a.get(key).unwrap_or(&ConstValue::Undef);
            let v2 = b.get(key).unwrap_or(&ConstValue::Undef);
            result.insert(key.clone(), self.meet_value(v1, v2));
        }

        result
    }

    fn meet_value(&self, a: &ConstValue, b: &ConstValue) -> ConstValue {
        match (a, b) {
            (ConstValue::Undef, ConstValue::Undef) => ConstValue::Undef,

            (ConstValue::Undef, _) | (_, ConstValue::Undef) => ConstValue::Undef,

            (ConstValue::Const(v1), ConstValue::Const(v2)) if v1 == v2 => { a.clone() }

            (ConstValue::Const(_), ConstValue::Const(_)) => ConstValue::Nac,

            _ => ConstValue::Nac,
        }
    }
}
