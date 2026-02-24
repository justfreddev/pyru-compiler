mod cfg;
mod codgen;
mod constprop;
mod constfolding;
mod liveliness;
mod error;
mod expr;
mod lexer;
mod list;
mod parser;
mod semanticanalyser;
mod stmt;
#[cfg(test)]
mod test;
mod token;
mod value;
mod vm;

use lexer::Lexer;
use parser::Parser;
use semanticanalyser::SemanticAnalyser;
use vm::VM;

use std::{ fs };
use error::{ Result };
use crate::{
    cfg::FunctionCFG,
    codgen::CodeGen,
    constfolding::ConstFolding,
    constprop::ConstProp,
    liveliness::Liveliness,
    value::Value,
};

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

    let constpropagation = ConstProp::new();
    let (in_map, out_map) = constpropagation.compute_constants(&cfg);
    constpropagation.rewrite_with_constants(&mut cfg, &in_map);

    let constfolding = ConstFolding::new(out_map);
    constfolding.fold_cfg(&mut cfg);

    let liveliness = Liveliness::new();
    liveliness.eliminate_dead_assignments(&mut cfg);

    cfg.clean_up();

    let mut codegen = CodeGen::new();
    let bytecode = codegen.generate(cfg);

    let mut vm = VM::new(bytecode);
    vm.execute(testing)
}
