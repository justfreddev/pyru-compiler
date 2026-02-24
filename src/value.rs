use std::{ cell::RefCell, collections::HashMap, fmt, rc::Rc };
use serde::Serialize;

use crate::{ codgen::Bytecode, vm::VM };

pub type Env = Rc<RefCell<HashMap<String, Value>>>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    Null,
    #[serde(serialize_with = "serialize_list")] List(Rc<RefCell<Vec<Value>>>),
    #[serde(skip_serializing)] Function {
        params: Vec<String>,
        arity: usize,
        body: Rc<Vec<Bytecode>>,
        env: Env,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Value::Num(n) => {
                let mut text = n.to_string();
                if text.ends_with(".0") {
                    text.truncate(text.len() - 2);
                }
                write!(f, "{text}")
            }
            Value::Str(s) => write!(f, "{s}"),
            Value::Bool(x) => write!(f, "{x}"),
            Value::Null => write!(f, "null"),
            Value::List(_) => write!(f, "{}", VM::stringify(self)),
            Value::Function { .. } => todo!(),
        };
    }
}

fn serialize_list<S>(list: &Rc<RefCell<Vec<Value>>>, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer
{
    let values = list.borrow();
    values.serialize(serializer)
}
