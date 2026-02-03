mod codegen;
mod constantfolder;
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
use lexer::Lexer;
use parser::Parser;
use semanticanalyser::SemanticAnalyser;
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

    let mut codegen = CodeGen::new();
    let bytecode: Vec<Bytecode> = codegen.run(ast);

    let mut vm = VM::new(bytecode);
    vm.execute();
}
