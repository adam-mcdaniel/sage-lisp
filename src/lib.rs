use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
};

use nom::{
    error::{convert_error, VerboseError}, Err
};

mod parser;
pub use parser::*;

use lazy_static::lazy_static;
use serde::Serialize;

lazy_static! {
    static ref SYMBOLS: RwLock<HashMap<String, Symbol>> = RwLock::new(HashMap::new());
}

/// A symbol that uses string interning
#[derive(Clone, Hash, Eq, Ord)]
pub struct Symbol(Arc<String>);

impl Symbol {
    /// Create a new symbol
    pub fn new(name: &str) -> Self {
        let mut symbols = SYMBOLS.write().unwrap();
        if let Some(symbol) = symbols.get(name) {
            return symbol.clone();
        }

        let symbol = Symbol(Arc::new(name.to_string()));
        symbols.insert(name.to_string(), symbol.clone());
        symbol
    }

    /// Get the name of the symbol
    pub fn name(&self) -> &str {
        &self.0
    }

    /// Get an iterator over all symbols
    pub fn all_symbols() -> Vec<Symbol> {
        SYMBOLS.read().unwrap().values().cloned().collect()
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Symbol::new(s)
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Symbol::new(&s)
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }
        self.0 == other.0
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if Arc::ptr_eq(&self.0, &other.0) {
            return Some(std::cmp::Ordering::Equal);
        }
        self.0.partial_cmp(&other.0)
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

/// An environment for evaluating lisp expressions
#[derive(Debug, Default, Clone)]
pub struct Env {
    bindings: Arc<HashMap<Expr, Arc<Expr>>>,
}

impl Env {
    /// Create a new environment
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_symbol(&mut self, symbol: &str, value: Expr) {
        self.bind(Expr::Symbol(Symbol::new(symbol)), value);
    }

    pub fn bind_builtin(&mut self, symbol: &'static str, f: fn(&mut Env, Vec<Expr>) -> Expr) {
        self.bind_symbol(symbol, Expr::Builtin(Builtin::new(f, symbol)));
    }

    pub fn bind_lazy_builtin(&mut self, symbol: &'static str, f: fn(&mut Env, Vec<Expr>) -> Expr) {
        self.bind_symbol(
            symbol,
            Expr::Builtin(Builtin::new(f, symbol).with_lazy_eval(true)),
        );
    }

    pub fn merge(&mut self, other: &Env) {
        for (k, v) in other.bindings.iter() {
            self.bind(k.clone(), (**v).clone());
        }
    }

    /// Duplicate a binding for one variable to another
    pub fn alias(&mut self, from: impl Into<Symbol>, to: impl Into<Symbol>) {
        let from = from.into();
        let to = to.into();
        if let Some(value) = self.get(&Expr::Symbol(from)) {
            self.bind(Expr::Symbol(to), value.clone());
        }
    }

    pub fn get_bindings(&self) -> HashMap<Expr, Expr> {
        self.bindings
            .iter()
            .map(|(k, v)| (k.clone(), (**v).clone()))
            .collect()
    }

    /// Bind a symbol to a value
    pub fn bind(&mut self, symbol: Expr, value: Expr) {
        if self.get(&symbol) == Some(&value) {
            return;
        }
        let bindings = Arc::make_mut(&mut self.bindings);
        bindings.insert(symbol, Arc::new(value));
        // self.bindings = Arc::new(bindings.clone());
    }

    /// Get the value of a symbol
    pub fn get(&self, symbol: &Expr) -> Option<&Expr> {
        self.bindings.get(symbol).map(|v| v.as_ref())
    }

    /// Remove a binding
    pub fn unbind(&mut self, symbol: &Expr) {
        let bindings = Arc::make_mut(&mut self.bindings);
        bindings.remove(symbol);
        self.bindings = Arc::new(bindings.clone());
    }

    /// Evaluate an expression in this environment
    pub fn eval(&mut self, mut expr: Expr) -> Expr {
        use Expr::*;
        let saved_bindings = self.bindings.clone();
        let mut is_in_new_env = false;
        loop {
            if let Some(value) = self.get(&expr) {
                return value.clone();
            }

            match &expr {
                List(l) => {
                    if l.is_empty() {
                        return expr;
                    }

                    let mut args = l.clone();
                    let func = args.remove(0);
                    let func = self.eval(func);

                    match func {
                        Function(env, params, body) => {
                            // saved_bindings = self.bindings.clone();
                            is_in_new_env = true;
                            if let Some(new_env) = env {
                                self.merge(&new_env);
                            }

                            if params.len() != args.len() {
                                return Expr::Err(Box::new(Expr::String(format!(
                                    "Expected {} arguments, got {}",
                                    params.len(),
                                    args.len()
                                ))));
                            }

                            let args = args
                                .into_iter()
                                .map(|arg| self.eval(arg))
                                .collect::<Vec<_>>();

                            for (param, arg) in params.into_iter().zip(args.into_iter()) {
                                self.bind(param, arg);
                            }

                            expr = *body;
                        }
                        Builtin(f) => {
                            expr = (f.f)(self, args);
                            if !f.lazy_eval {
                                break;
                            }
                            // saved_bindings = self.bindings.clone();
                        }
                        Tree(t) => {
                            // Get the element of the tree
                            let key = self.eval(args.get(0).unwrap().clone());

                            expr = t.get(&key).cloned().unwrap_or(Expr::None);
                            break;
                        }
                        Map(m) => {
                            // Get the element of the map
                            let key = self.eval(args.get(0).unwrap().clone());

                            // expr = if let Some(value) = m.get(&key) {
                            //     value.clone();
                            // } else {
                            //     Expr::None;
                            // }
                            expr = m.get(&key).cloned().unwrap_or(Expr::None);
                            break;
                        }
                        Symbol(s) => {
                            if let Some(value) = self.get(&expr) {
                                expr = value.clone();
                            } else {
                                expr = Expr::Err(Box::new(Expr::String(format!(
                                    "Symbol {} not found",
                                    s.name()
                                ))));
                            }
                        }

                        _result => {
                            // expr = Expr::Err(Box::new(Expr::String(format!("Cannot call {}", func))));
                            break;
                        }
                    }
                }
                Many(d) => {
                    if d.is_empty() {
                        return Expr::None;
                    }

                    // Eval the first expression
                    for e in d.iter().take(d.len() - 1) {
                        self.eval(e.clone());
                    }
                    expr = d.last().unwrap().clone();
                }
                Map(m) => {
                    let mut new_map = HashMap::new();
                    for (k, v) in m.iter() {
                        new_map.insert(k.clone(), self.eval(v.clone()));
                    }
                    expr = Expr::Map(new_map);
                    break;
                }
                Tree(t) => {
                    let mut new_tree = BTreeMap::new();
                    for (k, v) in t.iter() {
                        new_tree.insert(k.clone(), self.eval(v.clone()));
                    }
                    expr = Expr::Tree(new_tree);
                    break;
                }
                Quote(e) => {
                    expr = *e.clone();
                    break;
                }
                Function(Option::None, args, body) => {
                    // Replace the environment with a new one
                    let mut new_env = self.clone();
                    for arg in args.iter() {
                        new_env.unbind(arg);
                    }
                    expr = Function(Some(Box::new(new_env)), args.clone(), body.clone());
                    break;
                }
                _ => return expr,
            }
        }
        if is_in_new_env {
            self.bindings = saved_bindings;
        }
        expr
    }
}

#[derive(Debug, Clone)]
pub struct Builtin {
    pub f: fn(&mut Env, Vec<Expr>) -> Expr,
    pub name: &'static str,
    pub(crate) lazy_eval: bool,
}

impl Builtin {
    pub fn new(f: fn(&mut Env, Vec<Expr>) -> Expr, name: &'static str) -> Self {
        Self {
            f,
            name,
            lazy_eval: false,
        }
    }

    pub fn with_lazy_eval(self, lazy_eval: bool) -> Self {
        Self { lazy_eval, ..self }
    }

    pub fn apply(&self, env: &mut Env, args: Vec<Expr>) -> Expr {
        (self.f)(env, args)
    }
}

impl Display for Builtin {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "<builtin {}>", self.name)
    }
}

/// A lisp expression to be evaluated
#[derive(Debug, Default, Clone)]
pub enum Expr {
    #[default]
    None,

    /// A floating point number
    Float(f64),
    /// An integer
    Int(i64),
    /// A string
    String(String),
    /// A symbol
    Symbol(Symbol),
    /// A boolean
    Bool(bool),

    /// A list of expressions
    List(Vec<Expr>),
    /// A tree of expressions
    Tree(BTreeMap<Expr, Expr>),
    /// A map of expressions
    Map(HashMap<Expr, Expr>),

    /// A block of expressions to be evaluated in order
    Many(Arc<Vec<Expr>>),

    /// A quoted expression
    Quote(Box<Expr>),
    /// An error
    Err(Box<Self>),

    /// A function
    Function(Option<Box<Env>>, Vec<Expr>, Box<Expr>),
    /// A builtin function
    Builtin(Builtin),
}

impl From<String> for Expr {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for Expr {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for Expr {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<f64> for Expr {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl<T> From<Vec<T>> for Expr
where
    T: Into<Expr>,
{
    fn from(v: Vec<T>) -> Self {
        Self::List(v.into_iter().map(|e| e.into()).collect())
    }
}

impl From<serde_json::Value> for Expr {
    fn from(value: serde_json::Value) -> Self {
        use serde_json::Value::*;
        match value.clone() {
            Null => Expr::None,
            Bool(b) => Expr::Bool(b),
            Number(n) => {
                if n.is_f64() {
                    Expr::Float(n.as_f64().unwrap())
                } else {
                    Expr::Int(n.as_i64().unwrap())
                }
            }
            String(s) => Expr::String(s),
            Array(a) => Expr::List(a.into_iter().map(|e| e.into()).collect()),
            Object(o) => Expr::Tree(
                o.into_iter()
                    .map(|(k, v)| (Expr::String(k), v.into()))
                    .collect(),
            ),
        }
    }
}

impl From<Expr> for serde_json::Value {
    fn from(expr: Expr) -> Self {
        use serde_json::Value::*;
        match expr.clone() {
            Expr::None => Null,
            Expr::Bool(b) => Bool(b),
            Expr::Float(f) => Number(serde_json::Number::from_f64(f).unwrap()),
            Expr::Int(i) => Number(serde_json::Number::from(i)),
            Expr::String(s) => String(s),
            Expr::List(l) => Array(l.into_iter().map(|e| e.into()).collect()),
            Expr::Tree(m) => Object(
                m.into_iter()
                    .map(|(k, v)| match (k.into(), v.into()) {
                        (String(k), v) => (k, v),
                        (k, v) => (k.to_string(), v),
                    })
                    .collect(),
            ),
            Expr::Map(m) => Object(
                m.into_iter()
                    .map(|(k, v)| match (k.into(), v.into()) {
                        (String(k), v) => (k, v),
                        (k, v) => (k.to_string(), v),
                    })
                    .collect(),
            ),
            _ => Null,
        }
    }
}
use serde::de::DeserializeOwned;
impl Expr {
    pub fn serialize<T: Serialize>(x: T) -> Self {
        serde_json::to_value(&x).unwrap().into()
    }

    pub fn deserialize<T: DeserializeOwned>(x: &Self) -> Result<T, serde_json::Error> {
        serde_json::from_value::<T>(x.clone().into())
    }

    pub fn symbol(name: impl ToString) -> Self {
        Self::Symbol(Symbol::new(&name.to_string()))
    }

    pub fn error(message: impl Into<Self>) -> Self {
        Self::Err(Box::new(message.into()))
    }

    pub fn quote(&self) -> Self {
        Self::Quote(Box::new(self.clone()))
    }

    pub fn apply(&self, args: &[Self]) -> Self {
        let mut result = vec![self.clone()];
        result.extend(args.to_vec());
        Self::List(result)
    }

    pub fn parse(input: &str) -> Result<Expr, String> {
        let input = Self::remove_comments(input);
        let result = parser::parse_program::<VerboseError<&str>>(&input)
            .map(|(_, expr)| expr)
            .map_err(|e| match e {
                Err::Error(e) | Err::Failure(e) => convert_error::<&str>(&input, e),
                Err::Incomplete(e) => unreachable!("Incomplete: {:?}", e),
            });
        result
    }

    fn remove_comments(input: &str) -> String {
        let mut input = input;
        let mut output = String::new();
        while !input.is_empty() {
            // Ignore comments in quoted strings

            if input.starts_with('"') {
                let mut last_was_escape = false;
                let mut len = 0;
                for c in input[1..].chars() {
                    len += 1;
                    if c == '\\' && !last_was_escape {
                        last_was_escape = true;
                        continue;
                    }
                    if last_was_escape {
                        last_was_escape = false;
                        continue;
                    }

                    if c == '"' {
                        break;
                    }
                }

                output.push_str(&input[..len + 1]);
                input = &input[len + 1..];
            }

            if input.starts_with(';') {
                let end = input.find('\n').unwrap_or(input.len());
                input = &input[end..];
            } else if !input.is_empty() {
                output.push_str(&input[..1]);
                input = &input[1..];
                // let end = input.find(';').unwrap_or(input.len());
                // output.push_str(&input[..end]);
                // input = &input[end..];
            }
        }
        output
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        use Expr::*;
        match (self, other) {
            (None, None) => true,
            (Builtin(f1), Builtin(f2)) => f1 as *const _ == f2 as *const _,
            (Float(f1), Float(f2)) => f1.to_bits() == f2.to_bits(),
            (Int(i1), Int(i2)) => i1 == i2,
            (Int(i), Float(f)) | (Float(f), Int(i)) => *f == *i as f64,
            (String(s1), String(s2)) => s1 == s2,
            (Symbol(s1), Symbol(s2)) => s1 == s2,
            (List(l1), List(l2)) => l1 == l2,
            (Tree(t1), Tree(t2)) => t1 == t2,
            (Map(m1), Map(m2)) => m1 == m2,
            (Function(_, args1, body1), Function(_, args2, body2)) => {
                args1 == args2 && body1 == body2
            }
            (Quote(e1), Quote(e2)) => e1 == e2,
            (Err(e1), Err(e2)) => e1 == e2,
            (Bool(b1), Bool(b2)) => b1 == b2,
            (Many(d1), Many(d2)) => d1 == d2,
            _ => false,
        }
    }
}

impl PartialOrd for Expr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Expr::*;
        match (self, other) {
            (None, None) => Some(std::cmp::Ordering::Equal),
            (Float(f1), Float(f2)) => f1.partial_cmp(f2),
            (Int(i1), Int(i2)) => i1.partial_cmp(i2),
            (Int(i), Float(f)) => (*i as f64).partial_cmp(f),
            (Float(f), Int(i)) => f.partial_cmp(&(*i as f64)),
            (String(s1), String(s2)) => s1.partial_cmp(s2),
            (Symbol(s1), Symbol(s2)) => s1.partial_cmp(s2),
            (List(l1), List(l2)) => l1.partial_cmp(l2),
            (Tree(t1), Tree(t2)) => t1.partial_cmp(t2),
            (Map(m1), Map(m2)) => {
                let m1 = BTreeMap::from_iter(m1.iter());
                let m2 = BTreeMap::from_iter(m2.iter());
                m1.partial_cmp(&m2)
            }
            (Quote(e1), Quote(e2)) => e1.partial_cmp(e2),
            (Function(_, args1, body1), Function(_, args2, body2)) => {
                if args1 == args2 {
                    body1.partial_cmp(body2)
                } else {
                    args1.partial_cmp(args2)
                }
            }
            (Err(e1), Err(e2)) => e1.partial_cmp(e2),
            (Builtin(f1), Builtin(f2)) => {
                (f1 as *const _ as usize).partial_cmp(&(f2 as *const _ as usize))
            }
            (Bool(b1), Bool(b2)) => b1.partial_cmp(b2),
            (Many(d1), Many(d2)) => d1.partial_cmp(d2),
            _ => Option::None,
        }
    }
}

impl Eq for Expr {}

impl Ord for Expr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Less)
    }
}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Expr::*;
        // Write the tag as an integer to the hasher
        state.write_u8(match self {
            None => 0,
            Float(_) => 1,
            Int(_) => 2,
            Bool(_) => 3,
            String(_) => 4,
            Symbol(_) => 5,
            List(_) => 6,
            Tree(_) => 7,
            Map(_) => 8,
            Many(_) => 9,
            Quote(_) => 10,
            Err(_) => 11,
            Function(_, _, _) => 12,
            Builtin(_) => 13,
        });

        match self {
            None => 0.hash(state),
            Float(f) => f.to_bits().hash(state),
            Int(i) => i.hash(state),
            Bool(b) => b.hash(state),
            String(s) => s.hash(state),
            Symbol(s) => s.hash(state),
            List(l) => l.hash(state),
            Tree(t) => t.hash(state),
            Map(m) => BTreeMap::from_iter(m.iter()).hash(state),
            Many(d) => d.hash(state),
            Quote(e) => e.hash(state),
            Err(e) => e.hash(state),
            Function(_, args, body) => {
                args.hash(state);
                body.hash(state);
            }
            Builtin(f) => (f as *const _ as usize).hash(state),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use Expr::*;
        match self {
            None => write!(f, "nil"),
            Float(n) => write!(f, "{}", n),
            Int(n) => write!(f, "{}", n),
            Bool(b) => write!(f, "{}", b),
            String(s) => write!(f, "\"{}\"", s),
            Symbol(s) => write!(f, "{}", s.name()),
            Quote(e) => write!(f, "'{}", e),
            Err(e) => write!(f, "<error: {}>", e),
            Many(d) => {
                write!(f, "{{ ")?;
                for (i, e) in d.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, " }}")
            }
            List(e) => {
                write!(f, "(")?;
                for (i, e) in e.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            Tree(t) => {
                write!(f, "[")?;
                for (i, (k, v)) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{} {}", k, v)?;
                }
                write!(f, "]")
            }
            Map(m) => {
                write!(f, "#[")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{} {}", k, v)?;
                }
                write!(f, "]")
            }
            Function(_, args, body) => {
                write!(f, "(lambda (")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ") {})", body)
            }
            Builtin(b) => write!(f, "<builtin {}>", b.name),
        }
    }
}
