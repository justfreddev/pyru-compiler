use std::collections::HashMap;

use crate::{
    cfg::{ BlockId, FunctionCFG, Terminator },
    expr::{ BinaryOp, Expr, LogicalOp, UnaryOp },
    stmt::Stmt,
    value::Value,
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Bytecode {
    // Stack operations
    PushNum(f64),
    PushStr(String),
    PushBool(bool),
    PushNull,
    Pop,

    // Variable operations
    LoadVar(String),
    StoreVar(String),

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
    Jump(usize),
    JumpIfFalse(usize),

    // Function
    Function(String, Vec<String>, Vec<String>, Vec<Bytecode>),
    Call(usize),
    Return,

    // Print
    Print,

    // List operations
    MakeList(usize),
    Index,
    In,
    Slice(bool, bool),
    ListMethodCall(String, usize),
}

pub struct CodeGen {
    code: Vec<Bytecode>,
    block_offsets: HashMap<BlockId, usize>,
    unresolved_blocks: Vec<(usize, BlockId)>,
    current_cfg: Option<FunctionCFG>,
}

impl CodeGen {
    pub fn new() -> Self {
        return Self {
            code: vec![],
            block_offsets: HashMap::new(),
            unresolved_blocks: vec![],
            current_cfg: None,
        };
    }

    pub fn generate(&mut self, cfg: FunctionCFG) -> Vec<Bytecode> {
        self.current_cfg = Some(cfg.clone());

        for block in &cfg.blocks {
            self.block_offsets.insert(block.id, self.code.len());

            for stmt in &block.stmts {
                self.visit_stmt(stmt);
            }

            if let Some(terminator) = &block.terminator {
                self.visit_terminator(terminator);
            }
        }

        self.patch_block_jumps();

        return std::mem::take(&mut self.code);
    }

    fn visit_terminator(&mut self, term: &Terminator) {
        match term {
            Terminator::Goto(target) => {
                self.emit_block_jump(*target, false);
            }
            Terminator::Branch { cond, then_block, else_block } => {
                self.visit_expr(cond);
                self.emit_block_jump(*else_block, true);
                self.emit_block_jump(*then_block, false);
            }
            Terminator::Return(expr) => {
                if let Some(e) = expr {
                    self.visit_expr(e);
                }
                self.emit(Bytecode::Return);
            }
        }
    }

    fn emit_block_jump(&mut self, target: BlockId, if_false: bool) {
        let pos = self.code.len();
        if if_false {
            self.emit(Bytecode::JumpIfFalse(0));
        } else {
            self.emit(Bytecode::Jump(0));
        }
        self.unresolved_blocks.push((pos, target));
    }

    fn patch_block_jumps(&mut self) {
        for (idx, block_id) in self.unresolved_blocks.drain(..) {
            let target_offset = *self.block_offsets
                .get(&block_id)
                .expect("Block ID not found in offsets");

            match &mut self.code[idx] {
                Bytecode::Jump(t) | Bytecode::JumpIfFalse(t) => {
                    *t = target_offset;
                }
                _ => unreachable!(),
            }
        }
    }

    fn emit(&mut self, bc: Bytecode) {
        self.code.push(bc);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { name, value } => {
                self.visit_expr(value);
                self.emit(Bytecode::StoreVar(name.clone()));
            }

            Stmt::Print(expression) => {
                self.visit_expr(expression);
                self.emit(Bytecode::Print);
            }

            Stmt::Expression(expression) => {
                self.visit_expr(expression);
                if let Expr::ListMethodCall { object, .. } = expression {
                    self.emit(Bytecode::StoreVar(object.clone()));
                } else {
                    self.emit(Bytecode::Pop);
                }
            }

            Stmt::Var { name, initializer } => {
                if let Some(init) = initializer {
                    self.visit_expr(init);
                } else {
                    self.emit(Bytecode::PushNull);
                }
                self.emit(Bytecode::StoreVar(name.clone()));
            }

            Stmt::Incr(name) => {
                self.emit(Bytecode::LoadVar(name.clone()));
                self.emit(Bytecode::PushNum(1.0));
                self.emit(Bytecode::Add);
                self.emit(Bytecode::StoreVar(name.clone()));
            }

            Stmt::Decr(name) => {
                self.emit(Bytecode::LoadVar(name.clone()));
                self.emit(Bytecode::PushNum(-1.0));
                self.emit(Bytecode::Add);
                self.emit(Bytecode::StoreVar(name.clone()));
            }

            | Stmt::If { .. }
            | Stmt::While { .. }
            | Stmt::For { .. }
            | Stmt::Break
            | Stmt::Continue
            | Stmt::Return(_) => {
                panic!("Control flow statement in codegen");
            }

            Stmt::Function { name, .. } => {
                let nested_cfg = self.current_cfg
                    .as_ref()
                    .and_then(|cfg| cfg.nested_functions.get(name))
                    .expect("Function CFG should exist in nested_functions map");

                let mut inner_codegen = CodeGen::new();
                let function_bytecode = inner_codegen.generate(nested_cfg.clone());

                self.emit(
                    Bytecode::Function(
                        name.clone(),
                        nested_cfg.params.clone(),
                        nested_cfg.captures.clone(),
                        function_bytecode
                    )
                );

                self.emit(Bytecode::StoreVar(name.clone()));
            }
        }
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
}
