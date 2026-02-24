use actix_web::{ web, HttpResponse };
use serde::{ Deserialize, Serialize };
use crate::{
    lexer::Lexer,
    parser::Parser,
    semanticanalyser::SemanticAnalyser,
    cfg::FunctionCFG,
    constprop::ConstProp,
    constfolding::ConstFolding,
    liveliness::Liveliness,
    codgen::CodeGen,
    vm::VM,
    value::Value,
};

#[derive(Deserialize)]
pub struct CompileRequest {
    pub source: String,
}

#[derive(Serialize, Debug)]
pub struct CompileResponse {
    pub tokens: String,
    pub ast: serde_json::Value,
    pub pre_optimization_cfg: String,
    pub post_optimization_cfg: String,
    pub bytecode: String,
    pub output: Vec<String>,
    pub execution_successful: bool,
    pub error: Option<String>,
}

fn serialize_token(token: &crate::token::Token) -> String {
    format!(
        "Token {{ kind: {:?}, lexeme: {}, line: {} }}",
        token.kind,
        token.span.literal,
        token.line
    )
}

fn serialize_ast(ast: &[crate::stmt::Stmt]) -> serde_json::Value {
    serde_json
        ::to_value(
            ast
                .iter()
                .map(|stmt| format!("{:#?}", stmt))
                .collect::<Vec<_>>()
        )
        .unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize AST"}))
}

fn serialize_cfg(cfg: &FunctionCFG, name: &str) -> String {
    let mut output = format!("Function: {}\n", name);
    output.push_str(&format!("Parameters: {:?}\n", cfg.params));
    output.push_str("Blocks:\n");

    for block in &cfg.blocks {
        output.push_str(&format!("  Block {}:\n", block.id));

        if block.stmts.is_empty() {
            output.push_str("    Statements: (none)\n");
        } else {
            output.push_str("    Statements:\n");
            for stmt in &block.stmts {
                output.push_str(&format!("      - {:?}\n", stmt));
            }
        }

        output.push_str(&format!("    Terminator: {:?}\n", block.terminator));
        output.push_str(&format!("    Successors: {:?}\n\n", block.successors()));
    }

    output
}

fn serialize_bytecode(bytecode: &[crate::codgen::Bytecode]) -> String {
    bytecode
        .iter()
        .enumerate()
        .map(|(i, bc)| format!("{}: {:?}", i, bc))
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn compile(req: web::Json<CompileRequest>) -> HttpResponse {
    let source = &req.source;

    let mut response = CompileResponse {
        tokens: String::new(),
        ast: serde_json::json!({}),
        pre_optimization_cfg: String::new(),
        post_optimization_cfg: String::new(),
        bytecode: String::new(),
        output: Vec::new(),
        execution_successful: false,
        error: None,
    };

    let lexer = Lexer::new(source.trim_start());
    let tokens = match lexer.tokenise() {
        Ok(t) => t,
        Err(e) => {
            response.error = Some(format!("Lexer error: {}", e));
            return HttpResponse::BadRequest().json(response);
        }
    };

    response.tokens = tokens.iter().map(serialize_token).collect::<Vec<_>>().join("\n");

    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            response.error = Some(format!("Parser error: {}", e));
            return HttpResponse::BadRequest().json(response);
        }
    };

    response.ast = serialize_ast(&ast);

    let mut semantics = SemanticAnalyser::new();
    if let Err(e) = semantics.run(&ast) {
        response.error = Some(format!("Semantic error: {}", e));
        return HttpResponse::BadRequest().json(response);
    }

    let mut cfg = FunctionCFG::from_ast("main".to_string(), ast);
    response.pre_optimization_cfg = serialize_cfg(&cfg, "main");

    let constpropagation = ConstProp::new();
    let (in_map, out_map) = constpropagation.compute_constants(&cfg);
    constpropagation.rewrite_with_constants(&mut cfg, &in_map);

    let constfolding = ConstFolding::new(out_map);
    constfolding.fold_cfg(&mut cfg);

    let liveliness = Liveliness::new();
    liveliness.eliminate_dead_assignments(&mut cfg);

    cfg.clean_up();

    response.post_optimization_cfg = serialize_cfg(&cfg, "main");

    let mut codegen = CodeGen::new();
    let bytecode = codegen.generate(cfg);
    response.bytecode = serialize_bytecode(&bytecode);

    let mut vm = VM::new(bytecode);
    match vm.execute(true) {
        Ok(values) => {
            response.execution_successful = true;
            response.output = values
                .iter()
                .map(|v| v.to_string())
                .collect();
        }
        Err(e) => {
            response.error = Some(format!("Runtime error: {}", e));
        }
    }

    HttpResponse::Ok().json(response)
}

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(
        serde_json::json!({
        "status": "ok",
        "message": "Pyru Compiler API is running"
    })
    )
}
