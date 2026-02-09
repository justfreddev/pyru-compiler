use crate::{ execute_from_source, value::Value };

macro_rules! assert_output {
    ($src:expr, $expected:expr $(,)?) => {
        {
        match execute_from_source($src, true) {
            Ok(output) => {
                assert_eq!(output, $expected);
            }
            Err(err) => {
                panic!("expected success, but got error: {}", err);
            }
        }
        }
    };
}

macro_rules! assert_no_output {
    ($src:expr $(,)?) => {
        {
        match execute_from_source($src, true) {
            Ok(output) => {
                assert!(
                    output.is_empty(),
                    "expected no output, but got {:?}",
                    output
                );
            }
            Err(err) => {
                panic!("expected success, but got error: {}", err);
            }
        }
        }
    };
}

macro_rules! assert_error {
    ($src:expr) => {
        {
        match execute_from_source($src, true) {
            Ok(output) => {
                panic!(
                    "Expected error in the test execution but outputted {:?} instead",
                    output
                );
            }
            Err(_) => {
                true
            }
        }
        }
    };
}

#[test]
fn smoke_print() {
    assert_output!("print(1);\n", vec![Value::Num(1.0)]);
}

#[test]
fn smoke_no_output() {
    assert_no_output!("let a = 1;\n");
}

#[test]
fn invalid_assignment_target() {
    assert_error!("(a) = 1;\n");
}

#[test]
fn if_else() {
    assert_output!("if false:\n    print(1);\nelse:\n    print(2);\n", vec![Value::Num(2.0)]);
}

#[test]
fn unreachable_code_not_executed() {
    assert_output!("def f():\n    return 1;\n    print(2);\nprint(f());\n", vec![Value::Num(1.0)]);
}

#[test]
fn while_loop() {
    assert_output!(
        "let i = 0;\nwhile i < 3:\n    print(i);\n    i++;\n",
        vec![Value::Num(0.0), Value::Num(1.0), Value::Num(2.0)]
    );
}

#[test]
fn closure_captures_variable() {
    assert_output!(
        "def make():\n    let x = 1;\n    def f():\n        print(x);\n    return f;\nlet g = make();\ng();\n",
        vec![Value::Num(1.0)]
    );
}

#[test]
fn closure_mutation() {
    assert_output!(
        "def counter():\n    let i = 0;\n    def inc():\n        i++;\n        print(i);\n    return inc;\nlet c = counter();\nc();\nc();\n",
        vec![Value::Num(1.0), Value::Num(2.0)]
    );
}

#[test]
fn undefined_variable_is_error() {
    assert_error!("print(x);\n");
}

#[test]
fn error_in_unreachable_code_still_error() {
    assert_error!("if false:\n    print(x);\n");
}

#[test]
fn constant_folding_preserves_result() {
    assert_output!("print(2 + 3 * 4);\n", vec![Value::Num(14.0)]);
}

#[test]
fn constant_condition_removes_branch_but_keeps_effects() {
    assert_output!("if true:\n    print(1);\nelse:\n    print(2);\n", vec![Value::Num(1.0)]);
}
