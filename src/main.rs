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
mod api;

use lexer::Lexer;
use parser::Parser;
use semanticanalyser::SemanticAnalyser;
use vm::VM;
use actix_web::{ web, App, HttpServer };
use actix_cors::Cors;

use error::{ Result };
use crate::{
    cfg::FunctionCFG,
    codgen::CodeGen,
    constfolding::ConstFolding,
    constprop::ConstProp,
    liveliness::Liveliness,
    value::Value,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Pyru Compiler API on http://127.0.0.1:8080");
    println!("POST /compile - Compile source code");
    println!("GET /health - Health check");

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::default().allow_any_origin().allow_any_method().allow_any_header())
            .route("/health", web::get().to(api::health_check))
            .route("/compile", web::post().to(api::compile))
    })
        .bind("127.0.0.1:8080")?
        .run().await
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
