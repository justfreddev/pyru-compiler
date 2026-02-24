use std::collections::{ HashMap, HashSet };

use crate::{ expr::Expr, stmt::Stmt, value::Value };

pub type BlockId = usize;

#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub id: BlockId,
    pub stmts: Vec<Stmt>,
    pub terminator: Option<Terminator>,
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

#[derive(Clone, Debug)]
pub enum Terminator {
    Goto(BlockId),
    Branch {
        cond: Expr,
        then_block: BlockId,
        else_block: BlockId,
    },
    Return(Option<Expr>),
}

#[derive(Clone)]
pub struct FunctionCFG {
    name: String,
    pub params: Vec<String>,
    pub captures: Vec<String>,
    entry: BlockId,
    exit: BlockId,
    pub blocks: Vec<BasicBlock>,
    pub nested_functions: HashMap<String, FunctionCFG>,
}

impl FunctionCFG {
    pub fn from_ast(name: String, body: Vec<Stmt>) -> Self {
        let builder = CFGBuilder::new();
        return builder.build_function(name, vec![], vec![], body);
    }

    pub fn clean_up(&mut self) {
        let mut visited = HashSet::new();
        let mut stack = vec![self.entry];

        while let Some(block_id) = stack.pop() {
            if visited.insert(block_id) {
                if let Some(block) = self.blocks.iter().find(|b| b.id == block_id) {
                    for succ in block.successors() {
                        stack.push(succ);
                    }
                }
            }
        }

        self.blocks.retain(|b| visited.contains(&b.id));
    }

    fn compute_predecessors(&self) -> Vec<Vec<BlockId>> {
        let mut preds: Vec<Vec<BlockId>> = vec![vec![]; self.blocks.len()];

        for (idx, block) in self.blocks.iter().enumerate() {
            let from = idx;

            for succ in block.successors() {
                preds[succ].push(from);
            }
        }

        return preds;
    }

    pub fn compute_block_predecessors(&self, id: BlockId) -> Vec<BlockId> {
        let mut preds: Vec<BlockId> = Vec::new();

        for block in &self.blocks {
            for succ in block.successors() {
                if succ == id {
                    preds.push(block.id);
                }
            }
        }

        return preds;
    }

    pub fn compute_dominators(&self) -> Vec<HashSet<BlockId>> {
        let n = self.blocks.len();
        let preds = self.compute_predecessors();

        let mut dom: Vec<HashSet<BlockId>> = vec![HashSet::new(); n];

        for i in 0..n {
            if i == self.entry {
                dom[i].insert(self.entry); // Entry block dominates itself
            } else {
                dom[i] = (0..n).collect(); // Each block is initially dominated by all other blocks
            }
        }

        let mut changed = true;
        while changed {
            changed = false;

            for b in 0..n {
                if b == self.entry {
                    continue;
                }

                let mut new_dom = (0..n).collect::<HashSet<BlockId>>();
                for &p in &preds[b] {
                    new_dom = new_dom.intersection(&dom[p]).copied().collect();
                }

                new_dom.insert(b);

                if new_dom != dom[b] {
                    dom[b] = new_dom;
                    changed = true;
                }
            }
        }

        return dom;
    }

    pub fn print(&self) {
        println!("Function: {}", self.name);
        println!("Entry: {}", self.entry);
        println!("--------------------------------");

        for block in &self.blocks {
            println!("Block {}:", block.id);

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
    nested_functions: HashMap<String, FunctionCFG>,
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
            nested_functions: HashMap::new(),
        };
    }

    pub fn build_function(
        mut self,
        name: String,
        params: Vec<String>,
        captures: Vec<String>,
        body: Vec<Stmt>
    ) -> FunctionCFG {
        let entry = self.new_block();
        self.entry = entry;
        self.curr_block = entry;

        self.emit_stmt(Stmt::Var {
            name: "__ret".to_string(),
            initializer: Some(Expr::Literal(Value::Null)),
        });

        let exit = self.new_block();
        self.exit_block = exit;

        for stmt in body {
            self.visit_stmt(stmt);
        }

        if self.is_current_block_open() {
            self.emit_stmt(Stmt::Assign {
                name: "__ret".to_string(),
                value: Box::new(Expr::Literal(Value::Null)),
            });
            self.end_block(Terminator::Goto(exit));
        }

        self.set_current(exit);
        self.end_block(Terminator::Return(Some(Expr::Var("__ret".to_string()))));

        return FunctionCFG {
            name,
            params,
            captures,
            entry,
            exit,
            blocks: self.blocks,
            nested_functions: std::mem::take(&mut self.nested_functions),
        };
    }

    fn new_block(&mut self) -> BlockId {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            id,
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
                let fn_builder = CFGBuilder::new();
                let func_cfg = fn_builder.build_function(
                    name.clone(),
                    params.clone(),
                    captures.clone(),
                    body
                );

                self.nested_functions.insert(name.clone(), func_cfg);
                self.emit_stmt(Stmt::Function { name, params, body: vec![], captures });
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
                let ret_expr = expr.unwrap_or(Expr::Literal(Value::Null));

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
