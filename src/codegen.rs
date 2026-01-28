use crate::{
    expr::{ BinaryOp, Expr, LogicalOp, UnaryOp },
    stmt::{ Stmt },
    token::TokenKind,
    value::LiteralType,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Bytecode {
    // Stack operations
    PushNum(f64),
    PushStr(String),
    PushBool(bool),
    PushNull,
    Pop,

    // Variable operations
    LoadVar(String), // push variable value onto stack
    StoreVar(String), // pop stack and assign to variable

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Neg,

    // Comparison
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Eq,
    NotEq,

    // Logical
    And,
    Or,
    Not,

    // Control flow
    Jump(isize), // unconditional jump
    JumpIfFalse(isize), // pop stack; jump if false

    // Function
    Function(String, Vec<String>, Vec<Bytecode>),
    Call(usize), // call function by name, with N args
    Return,

    // Print
    Print,

    // List operations
    MakeList(usize), // pop N items, make list
    Index, // pop index and list, push list[index]
    In, // Membership
    Slice(bool, bool), // pop end, start, list, push slice
    ListMethodCall(String, usize), // call method on list
}

pub struct CodeGen {}

impl CodeGen {
    pub fn new() -> Self {
        return Self {};
    }

    pub fn run(&mut self, ast: Vec<Stmt>) -> Vec<Bytecode> {
        let mut bytecode: Vec<Bytecode> = vec![];

        for stmt in ast {
            let bc = self.visit_stmt(&stmt);
            bytecode.extend(bc);
        }

        return bytecode;
    }

    fn visit_expr(&mut self, expr: &Expr) -> Vec<Bytecode> {
        match expr {
            Expr::Binary { left, operator, right } => {
                let mut code = vec![];

                code.extend(self.visit_expr(left));
                code.extend(self.visit_expr(right));

                let op_code = match operator {
                    BinaryOp::Add => Bytecode::Add,
                    BinaryOp::Sub => Bytecode::Sub,
                    BinaryOp::Mul => Bytecode::Mul,
                    BinaryOp::Div => Bytecode::Div,
                    BinaryOp::Less => Bytecode::Less,
                    BinaryOp::LessEq => Bytecode::LessEq,
                    BinaryOp::Greater => Bytecode::Greater,
                    BinaryOp::GreaterEq => Bytecode::GreaterEq,
                    BinaryOp::Eq => Bytecode::Eq,
                    BinaryOp::NotEq => Bytecode::NotEq,
                };
                code.push(op_code);

                return code;
            }

            Expr::Call { callee, arguments } => {
                let mut code = vec![];

                code.extend(self.visit_expr(callee));

                for arg in arguments {
                    code.extend(self.visit_expr(arg));
                }

                code.push(Bytecode::Call(arguments.len()));

                return code;
            }

            Expr::Grouping { expression } => {
                let mut code = vec![];

                code.extend(self.visit_expr(expression));

                return code;
            }

            Expr::Index { list, index } => {
                let mut code = vec![];

                code.push(Bytecode::LoadVar(list.clone()));
                code.extend(self.visit_expr(index));

                return code;
            }

            Expr::List { items } => {
                let mut code = vec![];

                for item in items {
                    code.extend(self.visit_expr(item));
                }

                code.push(Bytecode::MakeList(items.len()));

                return code;
            }

            Expr::ListMethodCall { object, call } => {
                let mut code = vec![];

                code.push(Bytecode::LoadVar(object.clone()));

                match call.as_ref() {
                    Expr::Call { callee, arguments } => {
                        let method_name = match callee.as_ref() {
                            Expr::Var { name } => name.clone(),
                            _ => panic!("Invalid list method call"),
                        };

                        for arg in arguments {
                            code.extend(self.visit_expr(arg));
                        }

                        code.push(Bytecode::ListMethodCall(method_name, arguments.len()));
                    }
                    _ => panic!("Unexpected call in ListMethodCall"),
                }

                return code;
            }

            Expr::Literal { value } => {
                let mut code = vec![];

                match value {
                    LiteralType::Num(n) => code.push(Bytecode::PushNum(*n)),
                    LiteralType::Str(s) => code.push(Bytecode::PushStr(s.clone())),
                    LiteralType::True => code.push(Bytecode::PushBool(true)),
                    LiteralType::False => code.push(Bytecode::PushBool(false)),
                    LiteralType::Null => code.push(Bytecode::PushNull),
                }

                return code;
            }

            Expr::Logical { operator, left, right } => {
                let mut code = vec![];

                code.extend(self.visit_expr(left));
                code.extend(self.visit_expr(right));

                let op_code = match operator {
                    LogicalOp::And => Bytecode::And,
                    LogicalOp::Or => Bytecode::Or,
                };
                code.push(op_code);

                return code;
            }

            Expr::Membership { left, not, right } => {
                let mut code = vec![];

                code.extend(self.visit_expr(left));
                code.extend(self.visit_expr(right));

                code.push(Bytecode::In);

                if *not {
                    code.push(Bytecode::Not);
                }

                return code;
            }

            Expr::Slice { list, start, end } => {
                let mut code = vec![];

                code.push(Bytecode::LoadVar(list.clone()));

                if let Some(start_expr) = start {
                    code.extend(self.visit_expr(start_expr));
                }

                if let Some(end_expr) = end {
                    code.extend(self.visit_expr(end_expr));
                }

                code.push(Bytecode::Slice(start.is_some(), end.is_some()));

                return code;
            }

            Expr::Unary { operator, right } => {
                let mut code = vec![];

                code.extend(self.visit_expr(right));

                let op_code = match operator {
                    UnaryOp::Neg => Bytecode::Neg,
                    UnaryOp::Not => Bytecode::Not,
                };

                code.push(op_code);

                return code;
            }

            Expr::Var { name } => {
                return vec![Bytecode::LoadVar(name.clone())];
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> Vec<Bytecode> {
        match stmt {
            Stmt::Assign { name, value } => {
                let mut code = vec![Bytecode::LoadVar(name.clone())];

                code.extend(self.visit_expr(value));

                code.push(Bytecode::StoreVar(name.clone()));

                return code;
            }

            Stmt::Block { stmts } => {
                let mut code = vec![];

                for statement in stmts {
                    code.extend(self.visit_stmt(statement));
                }

                return code;
            }

            Stmt::Decr { name } => {
                return vec![
                    Bytecode::LoadVar(name.clone()),
                    Bytecode::PushNum(-1.0),
                    Bytecode::Add,
                    Bytecode::StoreVar(name.clone())
                ];
            }

            Stmt::Expression { expression } => {
                let mut code = self.visit_expr(expression);
                code.push(Bytecode::Pop);
                return code;
            }

            Stmt::For { initializer, condition, step, body } => {
                let mut code = vec![];

                code.extend(self.visit_stmt(initializer));

                let loop_start = code.len();

                code.extend(self.visit_expr(condition));

                let jif_index = code.len();
                code.push(Bytecode::JumpIfFalse(0));

                for stmt in body {
                    code.extend(self.visit_stmt(stmt));
                }

                code.extend(self.visit_stmt(step));

                let back_offset = (loop_start as isize) - (code.len() as isize) - 1;
                code.push(Bytecode::Jump(back_offset));

                let loop_end = code.len() as isize;
                if let Bytecode::JumpIfFalse(ref mut offset) = code[jif_index] {
                    *offset = loop_end - (jif_index as isize) - 1;
                }

                return code;
            }

            Stmt::Function { name, params, body } => {
                // Compile function body into its own bytecode chunk
                let mut fn_code = vec![];
                for stmt in body {
                    fn_code.extend(self.visit_stmt(stmt));
                }
                fn_code.push(Bytecode::PushNull); // ensure functions always return a value

                let mut code = vec![];
                code.push(Bytecode::Function(name.clone(), params.clone(), fn_code));
                code.push(Bytecode::StoreVar(name.clone())); // bind function to name

                return code;
            }

            Stmt::If { condition, then_branch, else_branch } => {
                let mut code = vec![];

                code.extend(self.visit_expr(condition));

                let jif_index = code.len();
                code.push(Bytecode::JumpIfFalse(0));

                for stmt in then_branch {
                    code.extend(self.visit_stmt(stmt));
                }

                let jmp_index = code.len();
                code.push(Bytecode::Jump(0));

                let else_start = code.len() as isize;
                if let Bytecode::JumpIfFalse(ref mut offset) = code[jif_index] {
                    *offset = else_start - (jif_index as isize) - 1;
                }

                if let Some(else_stmt) = else_branch {
                    for statement in else_stmt {
                        code.extend(self.visit_stmt(statement));
                    }
                }

                let end = code.len();
                if let Bytecode::Jump(ref mut offset) = code[jmp_index] {
                    *offset = (end as isize) - (jmp_index as isize) - 1;
                }

                return code;
            }

            Stmt::Incr { name } => {
                return vec![
                    Bytecode::LoadVar(name.clone()),
                    Bytecode::PushNum(1.0),
                    Bytecode::Add,
                    Bytecode::StoreVar(name.clone())
                ];
            }

            Stmt::Print { expression } => {
                let mut code = self.visit_expr(expression);
                code.push(Bytecode::Print);
                return code;
            }

            Stmt::Return { value } => {
                let mut code = vec![];
                if let Some(expr) = value {
                    code.extend(self.visit_expr(expr));
                } else {
                    code.push(Bytecode::PushNull);
                }
                code.push(Bytecode::Return);
                return code;
            }

            Stmt::Var { name, initializer } => {
                let mut code = vec![];
                if let Some(init) = initializer {
                    code.extend(self.visit_expr(init));
                } else {
                    code.push(Bytecode::PushNull);
                }
                code.push(Bytecode::StoreVar(name.clone()));
                return code;
            }

            Stmt::While { condition, body } => {
                let mut code = vec![];

                let loop_start = code.len() as isize;
                code.extend(self.visit_expr(condition));

                let jif_index = code.len();
                code.push(Bytecode::JumpIfFalse(0));

                for stmt in body {
                    code.extend(self.visit_stmt(stmt));
                }

                let back_offset = (loop_start as isize) - (code.len() as isize) - 1;
                code.push(Bytecode::Jump(back_offset));

                let loop_end = code.len() as isize;
                if let Bytecode::JumpIfFalse(ref mut offset) = code[jif_index] {
                    *offset = loop_end - (jif_index as isize);
                }

                return code;
            }
        }
    }
}
