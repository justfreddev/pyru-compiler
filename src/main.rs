mod codegen;
mod expr;
mod lexer;
mod parser;
mod stmt;
mod token;
mod value;
mod vm;

use codegen::{ Bytecode, CodeGen };
use lexer::Lexer;
use parser::Parser;
use vm::VM;
use std::{ fs, io };

fn main() {
    let mut file_name_input = String::new();
    io::stdin().read_line(&mut file_name_input).expect("Failed to read file name");
    let file_name = file_name_input.trim_end().to_string();

    let contents = fs::read_to_string(file_name).unwrap();

    let lexer = Lexer::new(contents.as_str());
    let tokens = lexer.tokenise();

    // for token in &tokens {
    //     println!("{token}");
    // }

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    let mut codegen = CodeGen::new();
    let bytecode: Vec<Bytecode> = codegen.run(ast);

    // for (idx, bytes) in bytecode.iter().enumerate() {
    //     println!("{idx}: {bytes:#?}");
    // }

    let mut vm = VM::new(bytecode);
    vm.execute();
}
