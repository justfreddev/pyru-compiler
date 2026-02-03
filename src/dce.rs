use crate::{ expr::Expr, stmt::Stmt, value::Value };

pub struct DeadCodeEliminator;

impl DeadCodeEliminator {
    pub fn new() -> Self {
        return Self;
    }

    fn is_terminator(&self, stmt: &Stmt) -> bool {
        match &stmt {
            Stmt::Return(_) | Stmt::Break | Stmt::Continue => true,
            _ => false,
        }
    }

    pub fn eliminate(&mut self, ast: Vec<Stmt>) -> Vec<Stmt> {
        let mut result = Vec::new();

        for stmt in ast {
            let res_stmt = match stmt {
                Stmt::If { condition, then_branch, else_branch } => {
                    match condition {
                        Expr::Literal(Value::Bool(true)) => {
                            result.extend(self.eliminate(then_branch));
                            continue;
                        }
                        Expr::Literal(Value::Bool(false)) => {
                            if let Some(else_branch) = else_branch {
                                result.extend(self.eliminate(else_branch));
                            }
                            continue;
                        }
                        _ =>
                            Stmt::If {
                                condition,
                                then_branch: self.eliminate(then_branch),
                                else_branch: match else_branch {
                                    Some(branch) => Some(self.eliminate(branch)),
                                    None => None,
                                },
                            },
                    }
                }

                Stmt::While { condition, body } => {
                    match condition {
                        Expr::Literal(Value::Bool(false)) => {
                            continue;
                        }
                        _ =>
                            Stmt::While {
                                condition,
                                body: self.eliminate(body),
                            },
                    }
                }

                Stmt::Function { name, params, body } =>
                    Stmt::Function {
                        name,
                        params,
                        body: self.eliminate(body),
                    },

                Stmt::For { initializer, condition, step, body } =>
                    Stmt::For {
                        initializer: Box::new(self.eliminate(vec![*initializer]).remove(0)),
                        condition,
                        step: Box::new(self.eliminate(vec![*step]).remove(0)),
                        body: self.eliminate(body),
                    },

                other => other,
            };

            result.push(res_stmt);

            if let Some(e) = result.last() {
                if self.is_terminator(e) {
                    break;
                }
            }
        }

        return result;
    }
}
