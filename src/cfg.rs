use crate::{ expr::Expr, stmt::Stmt, value::Value };

type BlockId = usize;

#[derive(Debug)]
pub struct BasicBlock {
    stmts: Vec<Stmt>,
    terminator: Option<Terminator>,
}

impl BasicBlock {
    pub fn successors(&self) -> Vec<BlockId> {
        match &self.terminator {
            Some(Terminator::Goto(t)) => vec![*t],
            Some(Terminator::Branch { then_block, else_block, .. }) => {
                vec![*then_block, *else_block]
            }
            _ => vec![],
        }
    }
}

#[derive(Debug)]
enum Terminator {
    Goto(BlockId),
    Branch {
        cond: Expr,
        then_block: BlockId,
        else_block: BlockId,
    },
    Return(Option<Expr>),
}

pub struct FunctionCFG {
    name: String,
    entry: BlockId,
    exit: BlockId,
    blocks: Vec<BasicBlock>,
}

impl FunctionCFG {
    pub fn from_ast(name: String, body: Vec<Stmt>) -> Self {
        let builder = CFGBuilder::new();
        return builder.build_function(name, body);
    }

    pub fn compute_predecessors(&self) -> Vec<Vec<BlockId>> {
        let mut preds: Vec<Vec<BlockId>> = vec![vec![]; self.blocks.len()];

        for (idx, block) in self.blocks.iter().enumerate() {
            let from = idx;

            for succ in block.successors() {
                preds[succ].push(from);
            }
        }

        preds
    }

    pub fn print(&self) {
        println!("Function: {}", self.name);
        println!("Entry: {}", self.entry);
        println!("--------------------------------");

        for (idx, block) in self.blocks.iter().enumerate() {
            println!("Block {}:", idx);

            println!("    successors: {:?}", block.successors());

            for stmt in &block.stmts {
                println!("    {:?}", stmt);
            }

            match &block.terminator {
                Some(Terminator::Goto(target)) => {
                    println!("    -> Goto {}", target);
                }
                Some(Terminator::Branch { cond, then_block, else_block }) => {
                    println!("    -> Branch {:?} ? {} : {}", cond, then_block, else_block);
                }
                Some(Terminator::Return(expr)) => {
                    println!("    -> Return {:?}", expr);
                }
                None => {
                    println!("    -> <no terminator>");
                }
            }

            println!();
        }
    }
}

pub struct CFGBuilder {
    entry: BlockId,
    blocks: Vec<BasicBlock>,
    curr_block: BlockId,
    exit_block: BlockId,
    break_stack: Vec<BlockId>,
    continue_stack: Vec<BlockId>,
    functions: Vec<FunctionCFG>,
}

impl CFGBuilder {
    pub fn new() -> Self {
        return Self {
            entry: 0,
            blocks: vec![],
            curr_block: 0,
            exit_block: 0,
            break_stack: vec![],
            continue_stack: vec![],
            functions: vec![],
        };
    }

    pub fn build_function(mut self, name: String, body: Vec<Stmt>) -> FunctionCFG {
        let entry = self.new_block();
        self.entry = entry;
        self.curr_block = entry;

        let exit = self.new_block();
        self.exit_block = exit;

        for stmt in body {
            self.visit_stmt(stmt);
        }

        if self.is_current_block_open() {
            self.end_block(Terminator::Goto(exit));
        }

        self.set_current(exit);
        self.end_block(Terminator::Return(None));

        return FunctionCFG { name, entry, exit, blocks: self.blocks };
    }

    fn new_block(&mut self) -> BlockId {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            stmts: vec![],
            terminator: None,
        });
        return id;
    }

    fn emit_stmt(&mut self, stmt: Stmt) {
        self.blocks[self.curr_block].stmts.push(stmt);
    }

    fn end_block(&mut self, terminator: Terminator) {
        let block = &mut self.blocks[self.curr_block];
        if block.terminator.is_some() {
            panic!("Block already terminated");
        }
        block.terminator = Some(terminator);
    }

    fn is_current_block_open(&self) -> bool {
        self.blocks[self.curr_block].terminator.is_none()
    }

    fn set_current(&mut self, block_id: BlockId) {
        self.curr_block = block_id;
    }

    fn visit_stmt(&mut self, stmt: Stmt) {
        if !self.is_current_block_open() {
            return;
        }

        match stmt {
            | Stmt::Assign { .. }
            | Stmt::Decr(_)
            | Stmt::Expression(_)
            | Stmt::Incr(_)
            | Stmt::Print(_)
            | Stmt::Var { .. } => {
                self.emit_stmt(stmt);
            }
            Stmt::Break => {
                self.blocks[self.curr_block].terminator = Some(
                    Terminator::Goto(
                        *self.break_stack
                            .last()
                            .unwrap_or_else(|| panic!("Tried to break outside of loop"))
                    )
                );
            }
            Stmt::Continue => {
                self.blocks[self.curr_block].terminator = Some(
                    Terminator::Goto(
                        *self.continue_stack
                            .last()
                            .unwrap_or_else(|| panic!("Tried to continue outside of loop"))
                    )
                );
            }
            Stmt::For { initializer, condition, step, body } => {
                self.emit_stmt(*initializer);

                let cond_bb = self.new_block();
                let body_bb = self.new_block();
                let exit_bb = self.new_block();

                self.end_block(Terminator::Goto(cond_bb));

                self.set_current(cond_bb);
                self.end_block(Terminator::Branch {
                    cond: condition,
                    then_block: body_bb,
                    else_block: exit_bb,
                });

                self.continue_stack.push(cond_bb);
                self.break_stack.push(exit_bb);

                self.set_current(body_bb);

                for stmt in body {
                    self.visit_stmt(stmt);
                }

                if self.is_current_block_open() {
                    self.emit_stmt(*step);
                    self.end_block(Terminator::Goto(cond_bb));
                }

                self.continue_stack.pop();
                self.break_stack.pop();

                self.set_current(exit_bb);
            }
            Stmt::Function { name, params, body, captures } => {
                self.emit_stmt(Stmt::Function {
                    name: name.clone(),
                    params,
                    body: vec![],
                    captures,
                });

                let fn_builder = CFGBuilder::new();
                let func_cfg = fn_builder.build_function(name, body);

                self.functions.push(func_cfg)
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let then_bb = self.new_block();
                let join_bb = self.new_block();
                let else_bb = if else_branch.is_some() { self.new_block() } else { join_bb };

                self.end_block(Terminator::Branch {
                    cond: condition,
                    then_block: then_bb,
                    else_block: else_bb,
                });

                self.set_current(then_bb);
                for stmt in then_branch {
                    self.visit_stmt(stmt);
                }

                if self.is_current_block_open() {
                    self.end_block(Terminator::Goto(join_bb));
                }

                if let Some(body) = else_branch {
                    self.set_current(else_bb);
                    for stmt in body {
                        self.visit_stmt(stmt);
                    }

                    if self.is_current_block_open() {
                        self.end_block(Terminator::Goto(join_bb));
                    }
                }

                self.set_current(join_bb);
            }
            Stmt::Return(expr) => {
                let ret_expr = match expr {
                    Some(e) => e,
                    None => Expr::Literal(Value::Null),
                };

                self.emit_stmt(Stmt::Assign {
                    name: "__ret".to_string(),
                    value: Box::new(ret_expr),
                });

                self.end_block(Terminator::Goto(self.exit_block));
            }
            Stmt::While { condition, body } => {
                let cond_bb = self.new_block();
                let body_bb = self.new_block();
                let exit_bb = self.new_block();

                self.end_block(Terminator::Goto(cond_bb));

                self.set_current(cond_bb);
                self.end_block(Terminator::Branch {
                    cond: condition,
                    then_block: body_bb,
                    else_block: exit_bb,
                });

                self.continue_stack.push(cond_bb);
                self.break_stack.push(exit_bb);

                self.set_current(body_bb);

                for stmt in body {
                    self.visit_stmt(stmt);
                }

                if self.is_current_block_open() {
                    self.end_block(Terminator::Goto(cond_bb));
                }

                self.continue_stack.pop();
                self.break_stack.pop();

                self.set_current(exit_bb);
            }
        }
    }
}
