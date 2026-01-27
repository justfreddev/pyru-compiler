use std::{ collections::HashMap, fmt, rc::Rc };

use crate::{ codegen::Bytecode };

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    Null,
    List(Vec<Value>),
    Function {
        params: Vec<String>,
        arity: usize,
        body: Rc<Vec<Bytecode>>,
    },
}

#[derive(Debug)]
struct CallFrame {
    ip: isize,
    bytecode: Rc<Vec<Bytecode>>,
    stack_base: usize,
    locals: HashMap<String, usize>,
}

pub struct VM {
    bytecode: Rc<Vec<Bytecode>>,
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    rte: HashMap<String, Value>,
    ip: isize,
}

impl VM {
    pub fn new(bytecode: Vec<Bytecode>) -> Self {
        return Self {
            bytecode: Rc::new(bytecode),
            stack: vec![],
            frames: vec![],
            rte: HashMap::new(),
            ip: 0,
        };
    }

    pub fn execute(&mut self) {
        while self.ip < (self.bytecode.len() as isize) {
            let mut advance = true;
            // println!("\n\n\nStack: {:#?}", self.stack);
            // println!("RTE: {:#?}", self.rte);
            // if self.frames.len() > 0 {
            //     println!("Frame: {:#?}", self.frames.last().unwrap());
            // }
            // println!("Current instruction: {:#?}", self.bytecode[self.ip]);
            match &self.bytecode[self.ip as usize] {
                Bytecode::PushNum(n) => self.stack.push(Value::Num(*n)),
                Bytecode::PushStr(s) => self.stack.push(Value::Str(s.clone())),
                Bytecode::PushBool(b) => self.stack.push(Value::Bool(*b)),
                Bytecode::PushNull => self.stack.push(Value::Null),
                Bytecode::Pop => {
                    self.stack.pop().expect("Stack underflow when popping from it");
                }

                Bytecode::LoadVar(name) => {
                    if let Some(frame) = self.frames.last() {
                        if let Some(&offset) = frame.locals.get(name) {
                            let value = self.stack[frame.stack_base + offset].clone();
                            self.stack.push(value);
                        } else {
                            let value = self.rte
                                .get(name)
                                .unwrap_or_else(|| panic!("Undefined variable {}", name));
                            self.stack.push(value.clone());
                        }
                    } else {
                        let value = self.rte
                            .get(name)
                            .unwrap_or_else(|| panic!("Undefined variable {}", name));
                        self.stack.push(value.clone());
                    }
                }
                Bytecode::StoreVar(name) => {
                    let value = self.stack
                        .pop()
                        .expect("Stack underflow when storing variable value");

                    if let Some(frame) = self.frames.last() {
                        if let Some(&offset) = frame.locals.get(name) {
                            self.stack[frame.stack_base + offset] = value;
                            return;
                        }
                    }

                    self.rte.insert(name.clone(), value);
                }

                Bytecode::Add => {
                    let b = self.stack.pop().expect("Stack underflow when adding");
                    let a = self.stack.pop().expect("Stack underflow when adding");

                    match (a, b) {
                        (Value::Num(x), Value::Num(y)) => self.stack.push(Value::Num(x + y)),
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
                    self.ip += *n;
                }
                Bytecode::JumpIfFalse(n) => {
                    if
                        let Value::Bool(x) = self.stack
                            .pop()
                            .expect("Stack underflow when jumping if false")
                    {
                        if !x {
                            self.ip += n;
                        }
                    } else {
                        panic!("Jump if false value isn't a boolean");
                    }
                }

                Bytecode::Function(_, params, body) => {
                    let func = Value::Function {
                        params: params.clone(),
                        arity: params.len(),
                        body: Rc::new(body.clone()),
                    };
                    self.stack.push(func);
                }
                Bytecode::Call(argc) => {
                    let func_index = self.stack.len() - argc - 1;
                    let func = self.stack.remove(func_index);

                    let Value::Function { params, arity, body } = func else {
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
                    println!("{}", val);
                }

                Bytecode::MakeList(n) => {
                    let mut list = vec![];
                    for _ in 0..*n {
                        list.push(self.stack.pop().expect("Stack underflow when making a list"));
                    }
                    self.stack.push(Value::List(list));
                }
                Bytecode::Index => {
                    let index = self.stack.pop().expect("Stack underflow when indexing list");
                    let list = self.stack.pop().expect("Stack underflow when indexing list");

                    if let Value::Num(i) = index {
                        if let Value::List(v) = list {
                            self.stack.push(v[i as usize].clone());
                        } else {
                            panic!("List being indexed isn't a list");
                        }
                    } else {
                        panic!("Num used to index isn't a num");
                    }
                }
                Bytecode::In => {
                    let list = self.stack.pop().expect("Stack underflow for In");
                    let item = self.stack.pop().expect("Stack underflow for In");

                    let result = match (item, list) {
                        (Value::Num(n), Value::List(l)) => l.iter().any(|v| *v == Value::Num(n)),
                        (Value::Str(s), Value::List(l)) =>
                            l.iter().any(|v| *v == Value::Str(s.clone())),
                        (Value::Bool(b), Value::List(l)) => l.iter().any(|v| *v == Value::Bool(b)),
                        (e, Value::List(l)) => l.iter().any(|v| *v == e),
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
                            let start_index = match start {
                                Some(Value::Num(n)) => n as usize,
                                None => 0,
                                Some(_) => panic!("Slice start must be a number"),
                            };

                            let end_index = match end {
                                Some(Value::Num(n)) => n as usize,
                                None => l.len(),
                                Some(_) => panic!("Slice end must be a number"),
                            };

                            let sliced = l[start_index..end_index.min(l.len())].to_vec();
                            Value::List(sliced)
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

                    let list = self.stack.pop().expect("Stack underflow for list method call");

                    let result = match list {
                        Value::List(mut l) =>
                            match method_name.as_str() {
                                "append" => {
                                    if *argc != 1 {
                                        panic!("append expects 1 argument");
                                    }
                                    l.push(args.remove(0));
                                    Value::List(l)
                                }
                                "pop" => {
                                    if *argc == 0 {
                                        l.pop().unwrap_or(Value::Null)
                                    } else if *argc == 1 {
                                        let idx = match &args[0] {
                                            Value::Num(n) => *n as usize,
                                            _ => panic!("pop index must be a number"),
                                        };
                                        if idx >= l.len() {
                                            Value::Null
                                        } else {
                                            l.remove(idx)
                                        }
                                    } else {
                                        panic!("pop expects 0 or 1 arguments");
                                    }
                                }
                                "len" => {
                                    if *argc != 0 {
                                        panic!("len expects 0 arguments");
                                    }
                                    Value::Num(l.len() as f64)
                                }
                                _ => panic!("Unknown list method: {}", method_name),
                            }
                        _ => panic!("List method called on non-list"),
                    };

                    self.stack.push(result);
                }
            }
            if advance {
                self.ip += 1;
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Value::Num(n) => write!(f, "{n}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Bool(x) => write!(f, "{x}"),
            Value::Null => write!(f, "null"),
            Value::List(v) => write!(f, "{v:?}"),
            Value::Function { .. } => todo!(),
        };
    }
}
