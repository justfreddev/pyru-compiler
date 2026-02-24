// https://www.youtube.com/watch?v=eeXk_ec1n6g

use std::collections::{ HashMap, HashSet };

use crate::{ cfg::{ BlockId, FunctionCFG }, expr::Expr, stmt::Stmt };

type Variable = String;

pub struct Liveliness;

impl Liveliness {
    pub fn new() -> Self {
        return Self {};
    }

    pub fn eliminate_dead_assignments(&self, cfg: &mut FunctionCFG) {
        let (_, live_out) = self.compute_liveliness(cfg);

        for block in &mut cfg.blocks {
            let mut live = live_out[&block.id].clone();

            let mut dead_stmt_indexes: HashSet<usize> = HashSet::new();

            for i in (0..block.stmts.len()).rev() {
                let stmt = &mut block.stmts[i];
                match stmt {
                    Stmt::Assign { name, value } => {
                        // Variable is dead
                        if !live.contains(name) {
                            if expr_has_side_effects(value) {
                                *stmt = Stmt::Expression((**value).clone());
                            } else {
                                dead_stmt_indexes.insert(i);
                            }

                            continue;
                        }

                        // Variable is live
                        live.remove(name);

                        // Add variables from RHS
                        for var in self.read_stmt_vars(stmt) {
                            live.insert(var);
                        }
                    }

                    Stmt::Var { name, initializer } => {
                        if !live.contains(name) {
                            if let Some(expr) = initializer {
                                if expr_has_side_effects(expr) {
                                    *stmt = Stmt::Expression((*expr).clone());
                                } else {
                                    dead_stmt_indexes.insert(i);
                                }

                                continue;
                            }
                        }

                        live.remove(name);

                        if let Some(_) = initializer {
                            for var in self.read_stmt_vars(stmt) {
                                live.insert(var);
                            }
                        }
                    }

                    _ => {
                        for var in self.read_stmt_vars(stmt) {
                            live.insert(var);
                        }
                    }
                }
            }

            block.stmts = block.stmts
                .drain(..)
                .enumerate()
                .filter(|(i, _)| !dead_stmt_indexes.contains(i))
                .map(|(_, stmt)| stmt)
                .collect();
        }
    }

    pub fn compute_liveliness(
        &self,
        cfg: &FunctionCFG
    ) -> (HashMap<BlockId, HashSet<Variable>>, HashMap<BlockId, HashSet<Variable>>) {
        let (use_map, def_map) = self.compute_use_def(cfg);

        let mut live_in: HashMap<BlockId, HashSet<Variable>> = HashMap::new();
        let mut live_out: HashMap<BlockId, HashSet<Variable>> = HashMap::new();

        // Set in[v] and out[v] to Ø
        for block in &cfg.blocks {
            live_in.insert(block.id, HashSet::new());
            live_out.insert(block.id, HashSet::new());
        }

        // Fixed point iteration
        loop {
            let mut changed = false;

            // Computing backwards (out to in) is more efficient
            for block in cfg.blocks.iter().rev() {
                let id = block.id;

                let in_prime = live_in[&id].clone(); // in'[v] <- in[v]
                let out_prime = live_out[&id].clone(); // out'[v] <- out[v]

                // out[v] <- union of in[w] where w is all the successors of v
                let mut out = HashSet::new();
                for succ in &block.successors() {
                    if let Some(s_live_in) = live_in.get(succ) {
                        out.extend(s_live_in.iter().cloned());
                    }
                }

                // in[v] <- use(v) U (out[v] \ def(v))
                let mut in_ = out.clone();
                if let Some(defs) = def_map.get(&id) {
                    for d in defs {
                        in_.remove(d);
                    }
                }

                if let Some(uses) = use_map.get(&id) {
                    for u in uses {
                        in_.insert(u.clone());
                    }
                }

                if in_ != in_prime || out != out_prime {
                    changed = true;
                }

                live_in.insert(id, in_);
                live_out.insert(id, out);
            }

            if !changed {
                break;
            }
        }

        return (live_in, live_out);
    }

    fn compute_use_def(
        &self,
        cfg: &FunctionCFG
    ) -> (HashMap<BlockId, HashSet<Variable>>, HashMap<BlockId, HashSet<Variable>>) {
        let mut use_map = HashMap::new();
        let mut def_map = HashMap::new();

        for block in &cfg.blocks {
            let mut uses = HashSet::new();
            let mut defs = HashSet::new();
            let mut seen_defs = HashSet::new();

            for stmt in &block.stmts {
                // Variables read
                for var in self.read_stmt_vars(stmt) {
                    if !seen_defs.contains(&var) {
                        uses.insert(var);
                    }
                }

                // Variables written
                for var in self.written_stmt_vars(stmt) {
                    defs.insert(var.clone());
                    seen_defs.insert(var);
                }
            }

            use_map.insert(block.id, uses);
            def_map.insert(block.id, defs);
        }
        return (use_map, def_map);
    }

    fn read_stmt_vars(&self, stmt: &Stmt) -> HashSet<Variable> {
        let mut out = HashSet::new();

        match stmt {
            Stmt::Assign { value, .. } => {
                self.read_expr_vars(value, &mut out);
            }

            Stmt::Expression(expr) | Stmt::Print(expr) => {
                self.read_expr_vars(expr, &mut out);
            }

            Stmt::Return(Some(expr)) => {
                self.read_expr_vars(expr, &mut out);
            }

            | Stmt::If { condition, .. }
            | Stmt::While { condition, .. }
            | Stmt::For { condition, .. } => {
                self.read_expr_vars(condition, &mut out);
            }

            Stmt::Incr(name) | Stmt::Decr(name) => {
                out.insert(name.clone());
            }

            Stmt::Var { initializer: Some(expr), .. } => {
                self.read_expr_vars(expr, &mut out);
            }

            _ => {}
        }

        return out;
    }

    fn read_expr_vars(&self, expr: &Expr, out: &mut HashSet<Variable>) {
        match expr {
            Expr::Var(name) => {
                out.insert(name.clone());
            }

            Expr::Index { list, index } => {
                out.insert(list.clone());
                self.read_expr_vars(index, out);
            }

            Expr::Slice { list, start, end } => {
                out.insert(list.clone());
                if let Some(s) = start {
                    self.read_expr_vars(s, out);
                }
                if let Some(e) = end {
                    self.read_expr_vars(e, out);
                }
            }

            Expr::ListMethodCall { object, arguments, .. } => {
                out.insert(object.clone());
                for arg in arguments {
                    self.read_expr_vars(arg, out);
                }
            }

            | Expr::Binary { left, right, .. }
            | Expr::Logical { left, right, .. }
            | Expr::Membership { left, right, .. } => {
                self.read_expr_vars(left, out);
                self.read_expr_vars(right, out);
            }

            Expr::Unary { right, .. } | Expr::Grouping(right) => {
                self.read_expr_vars(right, out);
            }

            Expr::Call { callee, arguments } => {
                self.read_expr_vars(callee, out);
                for arg in arguments {
                    self.read_expr_vars(arg, out);
                }
            }

            Expr::List(items) => {
                for item in items {
                    self.read_expr_vars(item, out);
                }
            }

            Expr::Literal(_) => {}
        }
    }

    fn written_stmt_vars(&self, stmt: &Stmt) -> HashSet<Variable> {
        let mut out = HashSet::new();

        match stmt {
            Stmt::Assign { name, .. } => {
                out.insert(name.clone());
            }

            Stmt::Var { name, .. } => {
                out.insert(name.clone());
            }

            Stmt::Incr(name) | Stmt::Decr(name) => {
                out.insert(name.clone());
            }

            _ => {}
        }

        return out;
    }
}

pub fn expr_has_side_effects(expr: &Expr) -> bool {
    match expr {
        Expr::Call { .. } | Expr::ListMethodCall { .. } => true,

        | Expr::Binary { left, right, .. }
        | Expr::Logical { left, right, .. }
        | Expr::Membership { left, right, .. } => {
            expr_has_side_effects(left) || expr_has_side_effects(right)
        }

        Expr::Unary { right, .. } | Expr::Grouping(right) => { expr_has_side_effects(right) }

        Expr::List(items) => {
            for item in items {
                if expr_has_side_effects(item) {
                    return true;
                }
            }
            return false;
        }

        Expr::Index { index, .. } => { expr_has_side_effects(index) }

        Expr::Slice { start, end, .. } => {
            if let Some(expr) = start {
                if expr_has_side_effects(expr) {
                    return true;
                }
            }

            if let Some(expr) = end {
                if expr_has_side_effects(expr) {
                    return true;
                }
            }

            return false;
        }

        Expr::Var(_) | Expr::Literal(_) => false,
    }
}
