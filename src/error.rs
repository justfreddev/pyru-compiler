// FOR THE FUTURE: https://docs.rs/annotate-snippets/latest/annotate_snippets/

use thiserror::Error;

use crate::{ token::TokenKind };

pub type Result<T> = std::result::Result<T, CompileError>;

#[derive(Debug)]
pub enum CompileError {
    Lexer(LexerError),
    Parser(ParserError),
    Semantic(SemanticError),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Lexer(err) => write!(f, "{}", err),
            CompileError::Parser(err) => write!(f, "{}", err),
            CompileError::Semantic(err) => write!(f, "{}", err),
        }
    }
}

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unexpected symbol '{symbol} on like {line}")] UnexpectedSymbol {
        line: usize,
        symbol: char,
    },

    #[error("Invalid indentation on line {line}, column {col}")] InvalidIndentation {
        line: usize,
        col: usize,
    },

    #[error("Unterminated string on line {line}")] UnterminatedString {
        line: usize,
    },
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("The function {func_name} has too many parameters")] TooManyParameters {
        func_name: String,
    },
    #[error("The function call {func_name} has too many arguments")] TooManyArgs {
        func_name: String,
    },
    #[error("Unknown statement unable to be parsed")]
    UnknownStatement,
    #[error("Expected an identifier on line {line}, got {got} instead")] ExpectedIdentifier {
        got: TokenKind,
        line: usize,
    },
    #[error("Expected a binary operator on line {line}, got {got} instead")] ExpectedBinaryOp {
        got: TokenKind,
        line: usize,
    },
    #[error(
        "Expected a comparison operator on line {line}, got {got} instead"
    )] ExpectedComparisonOp {
        got: TokenKind,
        line: usize,
    },
    #[error("Expected a term operator on line {line}, got {got} instead")] ExpectedTermOp {
        got: TokenKind,
        line: usize,
    },
    #[error("Expected a factor operator on line {line}, got {got} instead")] ExpectedFactorOp {
        got: TokenKind,
        line: usize,
    },
    #[error("Expected a unary operator on line {line}, got {got} instead")] ExpectedUnaryOp {
        got: TokenKind,
        line: usize,
    },
    #[error("Attempted to call a list method on a non-list object")]
    MethodCallNotOnList,
    #[error(
        "Missing a parenthesis after list method call on line {line}, got {got} instead"
    )] ListMethodCallMissingParen {
        got: TokenKind,
        line: usize,
    },
    #[error(
        "Unable to parse the literal {literal} to a float on line {line}"
    )] UnableToParseLiteralToFloat {
        literal: String,
        line: usize,
    },
    #[error(
        "Expected a number or a string for a literal on line {line}, got {got} instead"
    )] UnexpectedLiteral {
        got: TokenKind,
        line: usize,
    },
    #[error("Expected an expression on line {line}, got {got} instead")] ExpectedExpression {
        got: TokenKind,
        line: usize,
    },
    #[error(
        "Expected to consume a {expected} token on line {line}, got {got} instead"
    )] CouldntConsumeToken {
        expected: TokenKind,
        got: TokenKind,
        line: usize,
    },
}

#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("The variable {name} hasn't been declared")] UndeclaredVariable {
        name: String,
    },
    #[error("The variable {name} isn't defined")] UndefinedVariable {
        name: String,
    },
    #[error("No scope available to check function")]
    NoScopeAvailable,
    #[error(
        "The function {name} has the same name as another variable/function"
    )] FunctionRedefined {
        name: String,
    },
    #[error("The parameter {name} has already been declared in the scope")] ParameterRedeclared {
        name: String,
    },
    #[error("The variable {name} has been redefined in the same scope")] VariableRedefined {
        name: String,
    },
    #[error("Break used outside loop")]
    BreakOutsideLoop,
    #[error("Continue used outside loop")]
    ContinueOutsideLoop,
    #[error("The list {name} being indexed is unassigned")] UnassignedListIndexed {
        name: String,
    },
    #[error("Unknown list method {method_name} called on list {list}")] UnknownListMethod {
        list: String,
        method_name: String,
    },
    #[error("Method called on the unassigned list {list}")] MethodOnUnassignedList {
        list: String,
    },
    #[error("The undefined list {list} is being sliced")] UndefinedListSliced {
        list: String,
    },
    #[error("The unassigned variable {name} is being used")] UnassignedVariable {
        name: String,
    },
    #[error("Tried to exit global scope")]
    TriedExitingGlobalScope,
}
