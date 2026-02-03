use std::collections::HashMap;

use crate::{ expr::{ BinaryOp, Expr, LogicalOp, UnaryOp }, stmt::{ Stmt }, value::Value };

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct LabelId(usize);

#[derive(Clone, Debug, PartialEq, PartialOrd)]
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
    Jump(usize), // unconditional jump
    JumpIfFalse(usize), // pop stack; jump if false

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

struct LoopContext {
    break_label: LabelId,
    continue_label: LabelId,
}

pub struct CodeGen {
    code: Vec<Bytecode>,
    next_label_id: usize,
    label_positions: HashMap<LabelId, usize>,
    unresolved_jumps: Vec<(usize, LabelId)>,
    loop_stack: Vec<LoopContext>,
}

impl CodeGen {
    pub fn new() -> Self {
        return Self {
            code: vec![],
            next_label_id: 0,
            label_positions: HashMap::<LabelId, usize>::new(),
            unresolved_jumps: vec![],
            loop_stack: vec![],
        };
    }

    pub fn run(&mut self, ast: Vec<Stmt>) -> Vec<Bytecode> {
        for stmt in ast {
            self.visit_stmt(&stmt);
        }

        self.patch_jumps();
        return std::mem::take(&mut self.code);
    }

    fn emit(&mut self, bc: Bytecode) {
        self.code.push(bc);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Binary { left, operator, right } => {
                self.visit_expr(left);
                self.visit_expr(right);

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
                self.emit(op_code);
            }

            Expr::Call { callee, arguments } => {
                self.visit_expr(callee);

                for arg in arguments {
                    self.visit_expr(arg);
                }

                self.emit(Bytecode::Call(arguments.len()));
            }

            Expr::Grouping(expression) => {
                self.visit_expr(expression);
            }

            Expr::Index { list, index } => {
                self.emit(Bytecode::LoadVar(list.clone()));
                self.visit_expr(index);
                self.emit(Bytecode::Index);
            }

            Expr::List(items) => {
                for item in items.iter().rev() {
                    self.visit_expr(item);
                }

                self.emit(Bytecode::MakeList(items.len()));
            }

            Expr::ListMethodCall { object, method_name, arguments } => {
                self.emit(Bytecode::LoadVar(object.clone()));

                for arg in arguments {
                    self.visit_expr(arg);
                }

                self.emit(Bytecode::ListMethodCall(method_name.clone(), arguments.len()));
            }

            Expr::Literal(value) => {
                match value {
                    Value::Num(n) => self.emit(Bytecode::PushNum(*n)),
                    Value::Str(s) => self.emit(Bytecode::PushStr(s.clone())),
                    Value::Bool(true) => self.emit(Bytecode::PushBool(true)),
                    Value::Bool(false) => self.emit(Bytecode::PushBool(false)),
                    Value::Null => self.emit(Bytecode::PushNull),
                    _ => panic!("Value in literal not a literal value"),
                }
            }

            Expr::Logical { operator, left, right } => {
                self.visit_expr(left);
                self.visit_expr(right);

                let op_code = match operator {
                    LogicalOp::And => Bytecode::And,
                    LogicalOp::Or => Bytecode::Or,
                };
                self.emit(op_code);
            }

            Expr::Membership { left, not, right } => {
                self.visit_expr(left);
                self.visit_expr(right);

                self.emit(Bytecode::In);

                if *not {
                    self.emit(Bytecode::Not);
                }
            }

            Expr::Slice { list, start, end } => {
                self.emit(Bytecode::LoadVar(list.clone()));

                if let Some(start_expr) = start {
                    self.visit_expr(start_expr);
                }

                if let Some(end_expr) = end {
                    self.visit_expr(end_expr);
                }

                self.emit(Bytecode::Slice(start.is_some(), end.is_some()));
            }

            Expr::Unary { operator, right } => {
                self.visit_expr(right);

                let op_code = match operator {
                    UnaryOp::Neg => Bytecode::Neg,
                    UnaryOp::Not => Bytecode::Not,
                };

                self.emit(op_code);
            }

            Expr::Var(name) => {
                return self.emit(Bytecode::LoadVar(name.clone()));
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { name, value } => {
                self.visit_expr(value);
                self.emit(Bytecode::StoreVar(name.clone()));
            }

            Stmt::Break => {
                let context = self.loop_stack.last().expect("Break outside loop");

                self.emit_jump(context.break_label);
            }

            Stmt::Continue => {
                let context = self.loop_stack.last().expect("Continue outside loop");

                self.emit_jump(context.continue_label);
            }

            Stmt::Decr(name) => {
                self.emit(Bytecode::LoadVar(name.clone()));
                self.emit(Bytecode::PushNum(-1.0));
                self.emit(Bytecode::Add);
                self.emit(Bytecode::StoreVar(name.clone()));
            }

            Stmt::Expression(expression) => {
                self.visit_expr(expression);
                if let Expr::ListMethodCall { object, .. } = expression {
                    self.emit(Bytecode::StoreVar(object.clone()));
                    return;
                } else {
                    self.emit(Bytecode::Pop);
                }
            }

            Stmt::For { initializer, condition, step, body } => {
                self.visit_stmt(initializer);

                let cond_label = self.new_label();
                let continue_label = self.new_label();
                let break_label = self.new_label();

                self.place_label(cond_label);

                self.visit_expr(condition);
                self.emit_jump_if_false(break_label);

                self.loop_stack.push(LoopContext { break_label, continue_label });

                for stmt in body {
                    self.visit_stmt(stmt);
                }

                self.place_label(continue_label);
                self.visit_stmt(step);

                self.emit_jump(cond_label);

                self.loop_stack.pop();

                self.place_label(break_label);
            }

            Stmt::Function { name, params, body } => {
                let outer_code = std::mem::take(&mut self.code);
                let outer_labels = std::mem::take(&mut self.label_positions);
                let outer_unresolved = std::mem::take(&mut self.unresolved_jumps);
                let outer_loop_stack = std::mem::take(&mut self.loop_stack);

                self.code = vec![];
                self.label_positions = HashMap::new();
                self.unresolved_jumps = vec![];
                self.loop_stack = vec![];

                for stmt in body {
                    self.visit_stmt(stmt);
                }

                self.emit(Bytecode::PushNull);

                self.patch_jumps();

                let fn_code = std::mem::take(&mut self.code);

                self.code = outer_code;
                self.label_positions = outer_labels;
                self.unresolved_jumps = outer_unresolved;
                self.loop_stack = outer_loop_stack;

                self.emit(Bytecode::Function(name.clone(), params.clone(), fn_code));
                self.emit(Bytecode::StoreVar(name.clone()));
            }

            Stmt::If { condition, then_branch, else_branch } => {
                let else_label = self.new_label();
                let end_label = self.new_label();

                self.visit_expr(condition);
                self.emit_jump_if_false(else_label);

                for stmt in then_branch {
                    self.visit_stmt(stmt);
                }

                self.emit_jump(end_label);

                self.place_label(else_label);

                if let Some(else_stmt) = else_branch {
                    for statement in else_stmt {
                        self.visit_stmt(statement);
                    }
                }

                self.place_label(end_label);
            }

            Stmt::Incr(name) => {
                self.emit(Bytecode::LoadVar(name.clone()));
                self.emit(Bytecode::PushNum(1.0));
                self.emit(Bytecode::Add);
                self.emit(Bytecode::StoreVar(name.clone()));
            }

            Stmt::Print(expression) => {
                self.visit_expr(expression);
                self.emit(Bytecode::Print);
            }

            Stmt::Return(value) => {
                if let Some(expr) = value {
                    self.visit_expr(expr);
                } else {
                    self.emit(Bytecode::PushNull);
                }
                self.emit(Bytecode::Return);
            }

            Stmt::Var { name, initializer } => {
                if let Some(init) = initializer {
                    self.visit_expr(init);
                } else {
                    self.emit(Bytecode::PushNull);
                }
                self.emit(Bytecode::StoreVar(name.clone()));
            }

            Stmt::While { condition, body } => {
                let start_label = self.new_label();
                let break_label = self.new_label();

                self.place_label(start_label);

                self.visit_expr(condition);
                self.emit_jump_if_false(break_label);

                self.loop_stack.push(LoopContext { break_label, continue_label: start_label });

                for stmt in body {
                    self.visit_stmt(stmt);
                }

                self.emit_jump(start_label);

                self.loop_stack.pop();

                self.place_label(break_label);
            }
        }
    }

    fn new_label(&mut self) -> LabelId {
        let id = self.next_label_id;
        self.next_label_id += 1;
        return LabelId(id);
    }

    fn place_label(&mut self, label: LabelId) {
        self.label_positions.insert(label, self.code.len());
    }

    fn emit_jump(&mut self, label: LabelId) {
        let pos = self.code.len();
        self.code.push(Bytecode::Jump(0));
        self.unresolved_jumps.push((pos, label));
    }

    fn emit_jump_if_false(&mut self, label: LabelId) {
        let pos = self.code.len();
        self.code.push(Bytecode::JumpIfFalse(0));
        self.unresolved_jumps.push((pos, label));
    }

    fn patch_jumps(&mut self) {
        for (pos, label) in self.unresolved_jumps.drain(..) {
            let target = *self.label_positions.get(&label).expect("Unplaced label");

            match &mut self.code[pos] {
                Bytecode::Jump(t) | Bytecode::JumpIfFalse(t) => {
                    *t = target;
                }
                _ => unreachable!(),
            }
        }
    }
}
