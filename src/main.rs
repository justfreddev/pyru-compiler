mod lexer;

use std::{ fs, io };
use lexer::Lexer;

fn main() {
    let mut file_name_input = String::new();
    io::stdin().read_line(&mut file_name_input).expect("Failed to read file name");
    let file_name = file_name_input.trim_end().to_string();

    let contents = fs::read_to_string(file_name).unwrap();

    let lexer = Lexer::new(contents.as_str());
    let tokens = lexer.tokenise();
    println!("{:#?}", tokens);
}
