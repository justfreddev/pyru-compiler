use std::fmt;

// #[derive(Clone, Debug, PartialEq, PartialOrd)]
// pub enum Value {
//     Literal(LiteralType),
// }

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum LiteralType {
    Str(String),
    Num(f64),
    True,
    False,
    Null,
}

// impl fmt::Display for Value {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         return match self {
//             Value::Literal(literal) => write!(f, "{literal}"),
//         };
//     }
// }

impl fmt::Display for LiteralType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            LiteralType::Str(s) => write!(f, "{s}"),
            LiteralType::Num(n) => write!(f, "{n}"),
            LiteralType::True => write!(f, "true"),
            LiteralType::False => write!(f, "false"),
            LiteralType::Null => write!(f, "null"),
        };
    }
}
