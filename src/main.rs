mod codegen;
mod constprop;
mod dce;
mod expr;
mod lexer;
mod list;
mod parser;
mod semanticanalyser;
mod stmt;
mod token;
mod value;
mod vm;

use codegen::{ Bytecode, CodeGen };
use constprop::ConstPropagator;
use dce::DeadCodeEliminator;
use lexer::Lexer;
use parser::Parser;
use semanticanalyser::SemanticAnalyser;
use stmt::Stmt;
use vm::VM;

use std::{ fs, io };

fn main() {
    let mut file_name_input = String::new();
    io::stdin().read_line(&mut file_name_input).expect("Failed to read file name");
    let file_name = file_name_input.trim_end().to_string();

    let contents = fs::read_to_string(file_name).unwrap();

    let lexer = Lexer::new(contents.as_str());
    let tokens = lexer.tokenise();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    let mut semantics = SemanticAnalyser::new();
    semantics.run(&ast);

    let mut constprop = ConstPropagator::new();
    let propagated_ast: Vec<Stmt> = ast
        .iter()
        .map(|node| constprop.propagate_stmt(node))
        .collect();

    let mut dce = DeadCodeEliminator::new();
    let dce_removed_ast: Vec<Stmt> = dce.eliminate(propagated_ast);

    let mut codegen = CodeGen::new();
    let bytecode: Vec<Bytecode> = codegen.run(dce_removed_ast);

    let mut vm = VM::new(bytecode);
    vm.execute();
}
