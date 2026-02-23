mod cfg;
mod cfg_liveliness;
mod codegen;
mod constprop;
mod dce;
mod error;
mod expr;
mod lexer;
mod list;
mod liveliness;
mod parser;
mod semanticanalyser;
mod stmt;
#[cfg(test)]
mod test;
mod token;
mod value;
mod vm;

use codegen::{ Bytecode, CodeGen };
use constprop::ConstPropagator;
use dce::DeadCodeEliminator;
use lexer::Lexer;
use liveliness::LivelinessOptimiser;
use parser::Parser;
use semanticanalyser::SemanticAnalyser;
use stmt::Stmt;
use vm::VM;

use std::{ fs };
use error::{ Result };
use crate::{ cfg::{ FunctionCFG }, cfg_liveliness::Liveliness, value::Value };

fn main() {
    // let mut file_name_input = String::new();
    // io::stdin().read_line(&mut file_name_input).expect("Failed to read file name");
    // let file_name = file_name_input.trim_end().to_string();

    let contents = fs::read_to_string("test.pr").unwrap();

    match execute_from_source(contents.as_str(), false) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

pub fn execute_from_source(source: &str, testing: bool) -> Result<Vec<Value>> {
    let lexer = Lexer::new(source.trim_start());
    let tokens = lexer.tokenise()?;

    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    let mut semantics = SemanticAnalyser::new();
    semantics.run(&ast)?;

    let mut cfg = FunctionCFG::from_ast("main".to_string(), ast.clone());
    cfg.print();

    let liveliness = Liveliness::new();
    liveliness.eliminate_dead_assignments(&mut cfg);

    cfg.print();

    let mut constprop = ConstPropagator::new();
    let propagated_ast: Vec<Stmt> = ast
        .iter()
        .map(|node| constprop.propagate_stmt(node))
        .collect();

    // println!("------ PROPAGATED -------");
    // for node in &propagated_ast {
    //     println!("{node}");
    // }

    let mut dce = DeadCodeEliminator::new();
    let dce_removed_ast: Vec<Stmt> = dce.eliminate(propagated_ast);

    // println!("------ DCE -------");
    // for node in &dce_removed_ast {
    //     println!("{node}");
    // }

    let mut liveliness_optimiser = LivelinessOptimiser::new();
    let final_ast = liveliness_optimiser.optimise_tree(dce_removed_ast);

    // println!("------ FINAL -------");
    // for node in &final_ast {
    //     println!("{node}");
    // }

    let mut codegen = CodeGen::new();
    let bytecode: Vec<Bytecode> = codegen.run(final_ast);

    // for (i, bc) in bytecode.iter().enumerate() {
    //     println!("{i}: {bc:?}");
    // }

    let mut vm = VM::new(bytecode);
    vm.execute(testing)
}
