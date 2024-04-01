use std::{
    collections::{
        BTreeMap,
        HashMap,
    },
    hash::{
        Hash,
        Hasher,
    },
    sync::{Arc, RwLock},
    fmt::{Display, Formatter, Result as FmtResult},
};

use lazy_static::lazy_static;

lazy_static! {
    static ref SYMBOLS: RwLock<HashMap<String, Symbol>> = RwLock::new(HashMap::new());
}

/// A symbol that uses string interning
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
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
}


/// An environment for evaluating lisp expressions
#[derive(Debug, Default, Clone)]
pub struct Env {
    bindings: Arc<HashMap<Expr, Expr>>
}

impl Env {
    /// Create a new environment
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_symbol(&mut self, symbol: &str, value: Expr) {
        self.bind(Expr::Symbol(Symbol::new(symbol)), value);
    }

    pub fn bind_builtin(&mut self, symbol: &'static str, f: fn(&mut Env, &[Expr]) -> Expr) {
        self.bind_symbol(symbol, Expr::Builtin(Builtin::new(f, symbol)));
    }

    pub fn merge(&mut self, other: &Env) {
        for (k, v) in other.bindings.iter() {
            self.bind(k.clone(), v.clone());
        }
    }

    /// Bind a symbol to a value
    pub fn bind(&mut self, symbol: Expr, value: Expr) {
        if self.get(&symbol) == Some(&value) {
            return;
        }
        let bindings = Arc::make_mut(&mut self.bindings);
        bindings.insert(symbol, value);
        self.bindings = Arc::new(bindings.clone());
    }

    /// Get the value of a symbol
    pub fn get(&self, symbol: &Expr) -> Option<&Expr> {
        self.bindings.get(symbol)
    }

    /// Remove a binding
    pub fn remove(&mut self, symbol: &Expr) {
        let bindings = Arc::make_mut(&mut self.bindings);
        bindings.remove(symbol);
        self.bindings = Arc::new(bindings.clone());
    }

    /// Evaluate an expression in this environment
    pub fn eval(&mut self, mut expr: Expr) -> Expr {
        // println!("Evaluating {expr} in {self:?}");
        use Expr::*;
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
                            // let mut new_env = env.map(|env| env.as_ref().clone()).unwrap_or_default();
                            if let Some(new_env) = env {
                                self.merge(&new_env);
                            }

                            for (param, arg) in params.iter().zip(args.iter()) {
                                let arg = self.eval(arg.clone());
                                self.bind(param.clone(), arg);
                            }

                            expr = *body;
                        },
                        Builtin(f) => {
                            return (f.f)(self, &args);
                        },
                        _ => return Expr::Err(Box::new(Expr::String(format!("Cannot call {}", func)))),
                    }
                },
                Quote(e) => {
                    return *e.clone();
                },
                Function(_, args, body) => {
                    // Replace the environment with a new one
                    let mut new_env = self.clone();
                    for arg in args.iter() {
                        new_env.remove(arg);
                    }
                    return Function(Some(Box::new(new_env)), args.clone(), body.clone());
                }
                _ => return expr,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Builtin {
    pub f: fn(&mut Env, &[Expr]) -> Expr,
    pub name: &'static str,
}

impl Builtin {
    pub fn new(f: fn(&mut Env, &[Expr]) -> Expr, name: &'static str) -> Self {
        Self {
            f,
            name,
        }
    }
}

impl Display for Builtin {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "<builtin {}>", self.name)
    }
}

fn is_valid_symbol_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '+' || c == '*' || c == '/' || c == '%' || c == '!' || c == '?' || c == '=' || c == '<' || c == '>'
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

    /// A quoted expression
    Quote(Box<Expr>),
    /// An error
    Err(Box<Self>),

    /// A function
    Function(Option<Box<Env>>, Vec<Expr>, Box<Expr>),
    /// A builtin function
    Builtin(Builtin),
}

impl Expr {
    pub fn parse(mut input: &mut str) -> Result<(&mut str, Expr), String> {
        while input.starts_with(' ') || input.starts_with('\n') || input.starts_with('\t') {
            input = &mut input[1..];
        }

        // If the input is empty, return None
        if input.is_empty() {
            return Ok((input, Expr::None));
        }
        
        // Try to parse as a number
        // Split by whitespace
        let mut first_token = input.split_whitespace().next().ok_or("Could not parse input")?.to_owned();
        first_token = first_token.chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect();
        
        if let Ok(f) = first_token.parse::<f64>() {
            // Move the input past the number
            let input = &mut input[first_token.len()..];
            return Ok((input, Expr::Float(f)));
        }

        if let Ok(i) = first_token.parse::<i64>() {
            // Move the input past the number
            let input = &mut input[first_token.len()..];
            return Ok((input, Expr::Int(i)));
        }

        // if let Ok(f) = input.parse::<f64>() {
        //     // Move the input past the number
        //     let input = &mut input[f.to_string().len()..];
        //     return Ok((input, Expr::Float(f)));
        // }

        // if let Ok(i) = input.parse::<i64>() {
        //     // Move the input past the number
        //     let input = &mut input[i.to_string().len()..];
        //     return Ok((input, Expr::Int(i)));
        // }

        if input.starts_with("nil") {
            // Move the input past the symbol
            let input = &mut input["nil".len()..];
            return Ok((input, Expr::None));
        }

        if input.starts_with("true") {
            // Move the input past the symbol
            let input = &mut input["true".len()..];
            return Ok((input, Expr::Bool(true)));
        }

        if input.starts_with("false") {
            // Move the input past the symbol
            let input = &mut input["false".len()..];
            return Ok((input, Expr::Bool(false)));
        }

        // Try to parse as a string
        if input.starts_with('"') {
            // Find the end of the string
            let end = input[1..].find('"').ok_or("Could not parse input")?;
            let string = input[1..end + 1].to_string();
            // Move the input past the string
            let input = &mut input[end + 2..];
            return Ok((input, Expr::String(string)));
        }

        // Parse a quoted expression
        if input.starts_with('\'') {
            // Parse the quoted expression
            let (input, expr) = Expr::parse(&mut input[1..])?;
            return Ok((input, Expr::Quote(Box::new(expr))));
        }

        // Try to parse as a list
        if input.starts_with('(') {
            let mut list = Vec::new();
            let mut input = &mut input[1..];
            loop {
                // Skip whitespace
                while input.starts_with(' ') {
                    input = &mut input[1..];
                }

                if input.starts_with(')') {
                    // Move the input past the closing parenthesis
                    input = &mut input[1..];
                    return Ok((input, Expr::List(list)));
                }

                let (new_input, expr) = Expr::parse(input)?;
                input = new_input;
                list.push(expr);
            }
        }

        // Try to parse as a tree
        if input.starts_with('[') {
            let mut tree = BTreeMap::new();
            let mut input = &mut input[1..];
            loop {
                // Skip whitespace
                while input.starts_with(' ') {
                    input = &mut input[1..];
                }

                if input.starts_with(']') {
                    // Move the input past the closing bracket
                    input = &mut input[1..];
                    return Ok((input, Expr::Tree(tree)));
                }

                let (new_input, key) = Expr::parse(input)?;
                input = new_input;

                // Skip whitespace
                while input.starts_with(' ') {
                    input = &mut input[1..];
                }

                let (new_input, value) = Expr::parse(input)?;
                input = new_input;

                tree.insert(key, value);
            }
        }

        // Try to parse as a map
        if input.starts_with('{') {
            let mut map = HashMap::new();
            let mut input = &mut input[1..];
            loop {
                // Skip whitespace
                while input.starts_with(' ') {
                    input = &mut input[1..];
                }

                if input.starts_with('}') {
                    // Move the input past the closing brace
                    input = &mut input[1..];
                    return Ok((input, Expr::Map(map)));
                }

                let (new_input, key) = Expr::parse(input)?;
                input = new_input;

                // Skip whitespace
                while input.starts_with(' ') {
                    input = &mut input[1..];
                }

                let (new_input, value) = Expr::parse(input)?;
                input = new_input;

                map.insert(key, value);
            }
        }

        // Try to parse as a function
        if input.starts_with("fn") {
            // Parse the arguments
            // println!("Parsing function");
            // Remove the fn keyword
            let mut input = &mut input["fn".len()..];

            let mut args_list = Vec::new();
            while !input.starts_with('|') {
                let (i, arg) = Expr::parse(input)?;
                input = i;
                args_list.push(arg);
                // Remove leading whitespace
                while input.starts_with(' ') {
                    input = &mut input[1..];
                }
            }

            // Remove the | character
            while input.starts_with(' ') || input.starts_with('|') {
                input = &mut input[1..];
            }

            // Make body mutable so we can update it
            let (input, body) = Expr::parse(input)?;
            // This parses functions of the form:
            // fn a b c | body
            return Ok((input, Expr::Function(None, args_list, Box::new(body))));
        }

        
        

        // Try to parse as a symbol
        let mut symbol = String::new();
        while is_valid_symbol_char(input.chars().next().ok_or("Could not parse input")?) {
            symbol.push(input.chars().next().ok_or("Could not parse input")?);
            input = &mut input[1..];
        }
        // println!("Symbol: {}", symbol);
        if !symbol.is_empty() {
            return Ok((input, Expr::Symbol(Symbol::new(&symbol))));
        }

        Err("Could not parse input".to_string())
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
            (String(s1), String(s2)) => s1 == s2,
            (Symbol(s1), Symbol(s2)) => s1 == s2,
            (List(l1), List(l2)) => l1 == l2,
            (Tree(t1), Tree(t2)) => t1 == t2,
            (Map(m1), Map(m2)) => m1 == m2,
            (Function(_, args1, body1), Function(_, args2, body2)) => args1 == args2 && body1 == body2,
            (Quote(e1), Quote(e2)) => e1 == e2,
            (Err(e1), Err(e2)) => e1 == e2,
            (Bool(b1), Bool(b2)) => b1 == b2,
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
            (String(s1), String(s2)) => s1.partial_cmp(s2),
            (Symbol(s1), Symbol(s2)) => s1.partial_cmp(s2),
            (List(l1), List(l2)) => l1.partial_cmp(l2),
            (Tree(t1), Tree(t2)) => t1.partial_cmp(t2),
            (Map(m1), Map(m2)) => {
                let m1 = BTreeMap::from_iter(m1.iter());
                let m2 = BTreeMap::from_iter(m2.iter());
                m1.partial_cmp(&m2)
            },
            (Quote(e1), Quote(e2)) => e1.partial_cmp(e2),
            (Function(_, args1, body1), Function(_, args2, body2)) => {
                if args1 == args2 {
                    body1.partial_cmp(body2)
                } else {
                    args1.partial_cmp(args2)
                }
            },
            (Err(e1), Err(e2)) => e1.partial_cmp(e2),
            (Builtin(f1), Builtin(f2)) => (f1 as *const _ as usize).partial_cmp(&(f2 as *const _ as usize)),
            (Bool(b1), Bool(b2)) => b1.partial_cmp(b2),
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
        match self {
            None => 0.hash(state),
            Float(f) => f.to_bits().hash(state),
            Int(i) => i.hash(state),
            Bool(b) => b.hash(state),
            String(s) => s.hash(state),
            Symbol(s) => s.hash(state),
            List(l) => l.hash(state),
            Tree(t) => t.hash(state),
            Map(m) => {
                BTreeMap::from_iter(m.iter()).hash(state)
            },
            Quote(e) => e.hash(state),
            Err(e) => e.hash(state),
            Function(_, args, body) => {
                args.hash(state);
                body.hash(state);
            },
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
            List(e) => {
                write!(f, "(")?;
                for (i, e) in e.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            },
            Tree(t) => {
                write!(f, "[")?;
                for (i, (k, v)) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}:{}", k, v)?;
                }
                write!(f, "]")
            },
            Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}:{}", k, v)?;
                }
                write!(f, "}}")
            },
            Function(_, args, body) => {
                write!(f, "fn ")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, " | {} ", body)
            },
            Builtin(b) => write!(f, "<builtin {}>", b.name),
        }
    }
}