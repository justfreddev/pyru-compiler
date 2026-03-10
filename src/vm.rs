use std::{ cell::RefCell, collections::HashMap, rc::Rc };

use crate::{ codgen::Bytecode, error::CompileError, list::call_list_method, value::{ Env, Value } };

#[derive(Debug)]
struct CallFrame {
    ip: usize,
    bytecode: Rc<Vec<Bytecode>>,
    stack_base: usize,
    locals: HashMap<String, usize>,
    env: Env,
}

pub struct VM {
    bytecode: Rc<Vec<Bytecode>>,
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    rte: HashMap<String, Value>,
    ip: usize,
}

impl VM {
    pub fn new(bytecode: Vec<Bytecode>) -> Self {
        let mut rte = HashMap::new();
        rte.insert("__ret".to_string(), Value::Null);

        let entry_frame = CallFrame {
            ip: bytecode.len(),
            bytecode: Rc::new(bytecode.clone()),
            stack_base: 0,
            locals: HashMap::new(),
            env: Rc::new(RefCell::new(HashMap::new())),
        };

        return Self {
            bytecode: Rc::new(bytecode),
            stack: vec![],
            frames: vec![entry_frame],
            rte,
            ip: 0,
        };
    }

    pub fn execute(&mut self, testing: bool) -> Result<Vec<Value>, CompileError> {
        let mut output: Vec<Value> = Vec::new();

        while self.ip < self.bytecode.len() {
            let mut advance = true;

            match &self.bytecode[self.ip as usize] {
                Bytecode::PushNum(n) => self.stack.push(Value::Num(*n)),
                Bytecode::PushStr(s) => self.stack.push(Value::Str(s.clone())),
                Bytecode::PushBool(b) => self.stack.push(Value::Bool(*b)),
                Bytecode::PushNull => self.stack.push(Value::Null),
                Bytecode::Pop => {
                    self.stack.pop().expect("Stack underflow when popping from it");
                }

                Bytecode::LoadVar(name) => {
                    let mut res = None;
                    if let Some(frame) = self.frames.last() {
                        if let Some(&offset) = frame.locals.get(name) {
                            res = Some(self.stack[frame.stack_base + offset].clone());
                        } else if let Some(value) = frame.env.borrow().get(name) {
                            res = Some(value.clone());
                        }
                    }

                    if res.is_none() {
                        res = self.rte.get(name).cloned();
                    }

                    let value = res.unwrap_or_else(|| panic!("Undefined variable {}", name));

                    self.stack.push(value);
                }
                Bytecode::StoreVar(name) => {
                    let value = self.stack
                        .pop()
                        .expect("Stack underflow when storing variable value");

                    let mut stored = false;

                    if let Some(frame) = self.frames.last() {
                        if let Some(&offset) = frame.locals.get(name) {
                            self.stack[frame.stack_base + offset] = value.clone();
                            stored = true;
                        } else if frame.env.borrow().contains_key(name) {
                            frame.env.borrow_mut().insert(name.clone(), value.clone());
                            stored = true;
                        }
                    }

                    if !stored {
                        self.rte.insert(name.clone(), value);
                    }
                }

                Bytecode::Add => {
                    let b = self.stack.pop().expect("Stack underflow when adding");
                    let a = self.stack.pop().expect("Stack underflow when adding");

                    match (a, b) {
                        (Value::Num(x), Value::Num(y)) => self.stack.push(Value::Num(x + y)),
                        (Value::Str(x), Value::Str(y)) => self.stack.push(Value::Str(x + &y)),
                        _ => panic!("Type error in Add"),
                    };
                }
                Bytecode::Sub => {
                    let b = self.stack.pop().expect("Stack underflow when subracting");
                    let a = self.stack.pop().expect("Stack underflow when subracting");

                    match (a, b) {
                        (Value::Num(x), Value::Num(y)) => self.stack.push(Value::Num(x - y)),
                        _ => panic!("Type error in Subtract"),
                    };
                }
                Bytecode::Mul => {
                    let b = self.stack.pop().expect("Stack underflow when multiplying");
                    let a = self.stack.pop().expect("Stack underflow when multiplying");

                    match (a, b) {
                        (Value::Num(x), Value::Num(y)) => self.stack.push(Value::Num(x * y)),
                        _ => panic!("Type error in Multiply"),
                    }
                }
                Bytecode::Div => {
                    let b = self.stack.pop().expect("Stack underflow when dividing");
                    let a = self.stack.pop().expect("Stack underflow when dividing");

                    match (a, b) {
                        (Value::Num(x), Value::Num(y)) => self.stack.push(Value::Num(x / y)),
                        _ => panic!("Type error in Divide"),
                    }
                }
                Bytecode::Neg => {
                    let a: Value = self.stack
                        .pop()
                        .expect("Stack underflow when negating a number");

                    match a {
                        Value::Num(x) => self.stack.push(Value::Num(-x)),
                        _ => panic!("Type error in Negate"),
                    }
                }

                Bytecode::Less => {
                    let b = self.stack.pop().expect("Stack underflow when comparing less than");
                    let a = self.stack.pop().expect("Stack underflow when comparing less than");

                    match (a, b) {
                        (Value::Num(x), Value::Num(y)) => self.stack.push(Value::Bool(x < y)),
                        _ => panic!("Type error in Less"),
                    }
                }
                Bytecode::LessEq => {
                    let b = self.stack.pop().expect("Stack underflow when comparing less eq than");
                    let a = self.stack.pop().expect("Stack underflow when comparing less eq than");

                    match (a, b) {
                        (Value::Num(x), Value::Num(y)) => self.stack.push(Value::Bool(x <= y)),
                        _ => panic!("Type error in Less Equal"),
                    }
                }
                Bytecode::Greater => {
                    let b = self.stack.pop().expect("Stack underflow when comparing greater than");
                    let a = self.stack.pop().expect("Stack underflow when comparing greater than");

                    match (a, b) {
                        (Value::Num(x), Value::Num(y)) => self.stack.push(Value::Bool(x > y)),
                        _ => panic!("Type error in Greater"),
                    }
                }
                Bytecode::GreaterEq => {
                    let b = self.stack
                        .pop()
                        .expect("Stack underflow when comparing greater eq than");
                    let a = self.stack
                        .pop()
                        .expect("Stack underflow when comparing greater eq than");

                    match (a, b) {
                        (Value::Num(x), Value::Num(y)) => self.stack.push(Value::Bool(x >= y)),
                        _ => panic!("Type error in Greater Equal"),
                    }
                }
                Bytecode::Eq => {
                    let b = self.stack.pop().expect("Stack underflow when comparing equal to");
                    let a = self.stack.pop().expect("Stack underflow when comparing equal to");

                    match (a, b) {
                        (Value::Num(x), Value::Num(y)) => self.stack.push(Value::Bool(x == y)),
                        (Value::Str(x), Value::Str(y)) => self.stack.push(Value::Bool(x == y)),
                        (Value::Bool(x), Value::Bool(y)) => self.stack.push(Value::Bool(x == y)),
                        _ => panic!("Type error in Equal"),
                    }
                }
                Bytecode::NotEq => {
                    let b = self.stack.pop().expect("Stack underflow when comparing not equal to");
                    let a = self.stack.pop().expect("Stack underflow when comparing not equal to");

                    match (a, b) {
                        (Value::Num(x), Value::Num(y)) => self.stack.push(Value::Bool(x != y)),
                        _ => panic!("Type error in Not Equal"),
                    }
                }

                Bytecode::And => {
                    let b = self.stack.pop().expect("Stack underflow when doing logical AND");
                    let a = self.stack.pop().expect("Stack underflow when doing logical AND");

                    match (a, b) {
                        (Value::Bool(x), Value::Bool(y)) => self.stack.push(Value::Bool(x && y)),
                        _ => panic!("Type error in And"),
                    }
                }
                Bytecode::Or => {
                    let b = self.stack.pop().expect("Stack underflow when doing logical OR");
                    let a = self.stack.pop().expect("Stack underflow when doing logical OR");

                    match (a, b) {
                        (Value::Bool(x), Value::Bool(y)) => self.stack.push(Value::Bool(x || y)),
                        _ => panic!("Type error in Or"),
                    }
                }
                Bytecode::Not => {
                    let a = self.stack.pop().expect("Stack underflow when doing logical NOT");

                    if let Value::Bool(b) = a {
                        self.stack.push(Value::Bool(!b));
                    } else {
                        panic!("Type error in Not");
                    }
                }

                Bytecode::Jump(n) => {
                    self.ip = *n;
                    advance = false;
                }
                Bytecode::JumpIfFalse(n) => {
                    if
                        let Value::Bool(x) = self.stack
                            .pop()
                            .expect("Stack underflow when jumping if false")
                    {
                        if !x {
                            self.ip = *n;
                            advance = false;
                        }
                    } else {
                        panic!("Jump if false value isn't a boolean");
                    }
                }

                Bytecode::Function(_, params, captures, body) => {
                    let mut env = HashMap::new();

                    for name in captures {
                        if let Some(frame) = self.frames.last() {
                            if let Some(&offset) = frame.locals.get(name) {
                                env.insert(
                                    name.clone(),
                                    self.stack[frame.stack_base + offset].clone()
                                );
                                continue;
                            }

                            if let Some(v) = frame.env.borrow().get(name) {
                                env.insert(name.clone(), v.clone());
                                continue;
                            }
                        }

                        if let Some(v) = self.rte.get(name) {
                            env.insert(name.clone(), v.clone());
                        }
                    }

                    let func = Value::Function {
                        params: params.clone(),
                        arity: params.len(),
                        body: Rc::new(body.clone()),
                        env: Rc::new(RefCell::new(env)),
                    };

                    self.stack.push(func);
                }
                Bytecode::Call(argc) => {
                    let func_index = self.stack.len() - argc - 1;
                    let func = self.stack.remove(func_index);

                    let Value::Function { params, arity, body, env } = func else {
                        println!("{}", func);
                        panic!("Attempted to call non-function");
                    };

                    if *argc != arity {
                        panic!("Arity mismatch");
                    }

                    let mut locals = HashMap::new();
                    for (i, param) in params.iter().enumerate() {
                        locals.insert(param.clone(), i);
                    }

                    let frame = CallFrame {
                        ip: self.ip + 1,
                        bytecode: Rc::clone(&self.bytecode),
                        stack_base: self.stack.len() - argc,
                        locals,
                        env: Rc::clone(&env),
                    };
                    self.frames.push(frame);
                    self.bytecode = Rc::clone(&body);
                    self.ip = 0;
                    advance = false;
                }
                Bytecode::Return => {
                    let return_value = self.stack.pop().unwrap_or(Value::Null);
                    let frame = self.frames.pop().expect("Return outside function");

                    self.stack.truncate(frame.stack_base);

                    self.bytecode = frame.bytecode;
                    self.ip = frame.ip;
                    self.stack.push(return_value);
                    advance = false;
                }

                Bytecode::Print => {
                    let val = self.stack.pop().expect("Stack underflow when printing");
                    if testing {
                        output.push(val);
                    } else {
                        match val {
                            Value::Str(s) => println!("{s}"),
                            Value::Num(_) => println!("{}", VM::stringify(&val)),
                            Value::Bool(_) => println!("{}", VM::stringify(&val)),
                            Value::Null => println!("null"),
                            Value::List(_) => println!("{}", val),
                            _ => panic!("Can't print function"),
                        }
                    }
                }

                Bytecode::MakeList(n) => {
                    let mut list = vec![];
                    for _ in 0..*n {
                        list.push(self.stack.pop().expect("Stack underflow when making a list"));
                    }
                    self.stack.push(Value::List(Rc::new(RefCell::new(list))));
                }
                Bytecode::Index => {
                    let index = self.stack.pop().expect("Stack underflow when indexing list");
                    let list = self.stack.pop().expect("Stack underflow when indexing list");

                    let i = match index {
                        Value::Num(n) => n as usize,
                        _ => panic!("Num used to index isn't a num"),
                    };

                    match list {
                        Value::List(l) => {
                            let vec = l.borrow();
                            self.stack.push(vec[i].clone());
                        }
                        _ => panic!("List being indexed isn't a list"),
                    }
                }
                Bytecode::In => {
                    let list = self.stack.pop().expect("Stack underflow for In");
                    let item = self.stack.pop().expect("Stack underflow for In");

                    let result = match list {
                        Value::List(l) => {
                            let vec = l.borrow();
                            vec.iter().any(|v| *v == item)
                        }
                        _ => panic!("Invalid operands for in"),
                    };

                    self.stack.push(Value::Bool(result));
                }
                Bytecode::Slice(start_exists, end_exists) => {
                    let end = if *end_exists {
                        Some(self.stack.pop().expect("Stack underflow for slice end"))
                    } else {
                        None
                    };

                    let start = if *start_exists {
                        Some(self.stack.pop().expect("Stack underflow for slice start"))
                    } else {
                        None
                    };

                    let list = self.stack.pop().expect("Stack underflow for slice list");

                    let result = match list {
                        Value::List(l) => {
                            let vec = l.borrow();
                            let start_index = match start {
                                Some(Value::Num(n)) => n as usize,
                                None => 0,
                                Some(_) => panic!("Slice start must be a number"),
                            };

                            let end_index = match end {
                                Some(Value::Num(n)) => n as usize,
                                None => vec.len(),
                                Some(_) => panic!("Slice end must be a number"),
                            };

                            let sliced = vec[start_index..end_index.min(vec.len())].to_vec();
                            Value::List(Rc::new(RefCell::new(sliced)))
                        }
                        _ => panic!("Slice applied to non-list value"),
                    };

                    self.stack.push(result);
                }
                Bytecode::ListMethodCall(method_name, argc) => {
                    let mut args = Vec::with_capacity(*argc);
                    for _ in 0..*argc {
                        args.push(self.stack.pop().expect("Stack underflow for list method call"));
                    }
                    args.reverse();

                    let list_val = self.stack.pop().expect("Stack underflow for list method call");

                    let list = match list_val {
                        Value::List(l) => l,
                        _ => panic!("List method called on non-list"),
                    };

                    // Call the method (returns Value::List or item)
                    let result = call_list_method(list.clone(), method_name, args);

                    self.stack.push(result);
                }
            }
            if advance {
                self.ip += 1;
            }
        }

        if testing {
            return Ok(output);
        } else {
            return Ok(vec![]);
        }
    }

    pub fn stringify(object: &Value) -> String {
        return match object {
            Value::Num(n) => {
                let mut text = n.to_string();
                if text.ends_with(".0") {
                    text.truncate(text.len() - 2);
                }
                text
            }
            Value::Str(s) => String::from("\"".to_string() + s + "\""),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::List(l) => {
                let vec = l.borrow();
                let mut text = String::from("[");
                text.push_str(
                    &vec
                        .iter()
                        .map(|item| VM::stringify(item))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                text.push(']');
                text
            }
            _ => panic!("Unable to stringify value"),
        };
    }
}
