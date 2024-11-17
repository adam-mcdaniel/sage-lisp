//! # Sage-Lisp
//! 
//! - **Author**: Adam McDaniel
//! - **GitHub**: [adam-mcdaniel/sage-lisp](https://github.com/adam-mcdaniel/sage-lisp)
//! - **License**: MIT
//! 
//! Sage-Lisp is a simple core lisp interpreter written in Rust,
//! that can be expanded to include new functionality with built-in functions.
//! 
//! It's designed such that you can provide your own standard library of functions,
//! implemented in Rust or in Lisp, and evaluate lisp expressions with your own
//! standard conventions.
//! 
//! It works well as an embedded language for data transformation, where you can
//! import data into the interpreter (using Serde), perform transformations on it, and then
//! export the data back out into a format that you can use in your application.
//! 
//! ## Features
//! 
//! - **Simple Lisp Interpreter**: A simple core lisp interpreter that can evaluate lisp expressions.
//! - **Built-in Functions**: Extend the language with new functionality using built-in functions.
//! - **Symbol Interning**: Symbols are interned to ensure that they are unique and fast to compare.
//! - **Tail Recursion**: Uses tail recursion to evaluate deeply nested function calls without stack overflow.
//! - **Lazy Evaluation**: Supports lazy evaluation of expressions, for defining special forms.
//! - **Serde Integration**: Serialize and deserialize lisp expressions using Serde.
//! - **Error Handling**: Provides helpful error messages for parsing and evaluation errors.
//! - **Expanded Syntax**: Introduces infix operators, code block syntax, syntax for hashmaps and ordered maps, and more.
//! - **Customizable**: Define your own standard library of functions and variables, and override the default behavior.
//! 
//! ## Example Lisp Syntax
//! 
//! The example below shows some of the syntax that is supported by the interpreter,
//! along with some of the built-in functions that are provided when the crate is
//! used as an executable.
//! 
//! ```lisp
//! ;; Define a function that calculates the factorial of a number
//! (defun fact (n) 
//!     (if n <= 0
//!         1
//!         n * (fact n - 1)))
//! ;; Stirling's approximation for the factorial
//! (defun stirlings (n)
//!     (if n <= 0 1
//!         (* (sqrt 2 * 3.14159265358979323846 * n)
//!            ((n / 2.71828182845904523536) ^ n))))
//! ;; Perform a quicksort on a list of numbers
//! (defun quicksort (lst)
//!     (if (<= (len lst) 1) lst {
//!         (define pivot (get lst (/ (len lst) 2)))
//!         (define less (filter (\(x) (< x pivot)) lst))
//!         (define equal (filter (\(x) (= x pivot)) lst))
//!         (define greater (filter (\(x) (> x pivot)) lst))
//!         (+ (quicksort less) equal (quicksort greater))}))
//! ```
//! 
//! ## Example Embedding
//! 
//! Below is an example of how you can embed the interpreter into your application,
//! with your own standard library of functions and variables.
//! 
//! ```rust
//! // Import the necessary types and traits for the interpreter.
//! use sage_lisp::{Expr, Env};
//! 
//! // Create a new environment
//! fn make_core_env() -> Env {
//!     // Create a new environment
//!     let mut env = Env::new();
//! 
//!     // Create a function that adds two numbers
//!     env.bind_builtin("+", |env, exprs| {
//!         let mut sum = Expr::default();
//!         for e in exprs {
//!             // Evaluate the supplied argument in the environment
//!             let e = env.eval(e.clone());
//!             // Match the expression to the sum
//!             match (sum, e) {
//!                 (Expr::None, b) => sum = b,
//!                 (Expr::Int(a), Expr::Int(b)) => sum = Expr::Int(a + b),
//!                 (Expr::Float(a), Expr::Float(b)) => sum = Expr::Float(a + b),
//!                 (Expr::Int(a), Expr::Float(b)) => sum = Expr::Float(a as f64 + b),
//!                 (Expr::Float(a), Expr::Int(b)) => sum = Expr::Float(a + b as f64),
//!                 (Expr::String(a), Expr::String(b)) => sum = Expr::String(format!("{}{}", a, b)),
//!                 (Expr::List(a), Expr::List(b)) => {
//!                     let mut list = a.clone();
//!                     list.extend(b);
//!                     sum = Expr::List(list);
//!                 }
//!                 (Expr::List(a), b) => {
//!                     let mut list = a.clone();
//!                     list.push(b);
//!                     sum = Expr::List(list);
//!                 }
//!                 // Return an error if the expression is invalid
//!                 (a, b) => return Expr::error(format!("Invalid expr {} + {}", a, b)),
//!             }
//!         }
//!         sum
//!     });
//!     // Create a function that prints a string
//!     env.bind_builtin("println", |env, exprs| {
//!         for e in exprs {
//!             // Evaluate the supplied argument in the environment
//!             let e = env.eval(e.clone());
//!             // Print the expression
//!             match e {
//!                 Expr::String(s) => print!("{}", s),
//!                 Expr::Symbol(s) => print!("{}", s.name()),
//!                 _ => print!("{}", e),
//!             }
//!         }
//!         println!();
//!         Expr::None
//!     });
//!     env
//! }
//! 
//! fn main() {
//!     // Create a new environment with our custom standard library
//!     let mut env = make_core_env();
//! 
//!     // Evaluate a lisp expression in the environment
//!     env.eval_str("{ (println 1 + 2) (println (+ 2 2)) }").unwrap();
//! }
//! ```

///////////////////////////////////////////////////////////////
// LIBRARIES AND MODULES
///////////////////////////////////////////////////////////////

/*
 * This is the main module for the lisp interpreter.
 * 
 * It contains everything for the core evaluation of lisp expressions,
 * and expanding the language with new functionality with built-in functions.
 *
 */

use std::{
    // Import BTreeMap and HashMap from the standard library.
    // BTreeMap is used for tree expressions, which are ordered maps.
    // HashMap is used for map expressions, which are unordered maps,
    // and for the symbol table plus environment bindings.
    collections::{BTreeMap, HashMap},
    // Import the necessary types and traits for formatting our output.
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    // Import hash types and traits for hashing our expressions,
    // to allow them to be used as keys in a hash map.
    hash::{Hash, Hasher},
    // Import atomic reference counting for shared ownership of symbols,
    // and read-write locks for the symbol table.
    sync::{Arc, RwLock},
};

// Import some nom functions and types for handling parsing errors,
// and displaying helpful error messages to the user.
use nom::{
    error::{convert_error, VerboseError}, Err
};

// Import serde for serializing and deserializing Lisp expressions.
// This allows anything to be serialized into a Lisp expression,
// which is convenient for importing data into the interpreter.
// From there, you can use this lisp as a simple embedded language
// that performs transformations on your data, and then deserializes
// it back out into a format that you can use in your application.
use serde::Serialize;
use serde::de::DeserializeOwned;

// Use lazy_static for setting up the symbol table as a global variable.
use lazy_static::lazy_static;

// Import the parser module for parsing lisp expressions.
mod parser;
pub use parser::*;


///////////////////////////////////////////////////////////////
// SYMBOLS AND SYMBOL TABLE
///////////////////////////////////////////////////////////////

/*
 * The symbol table is a hash map that maps strings to symbols.
 * It uses string interning to ensure that symbols are unique,
 * and to allow for fast comparison of symbols.
 */

lazy_static! {
    /// The symbol table that maps strings to symbols
    /// 
    /// This is a global variable that is shared between all environments.
    /// It is a read-write lock that allows for multiple environments to
    /// read from the symbol table at the same time, but only one environment
    /// to write to the symbol table at a time.
    static ref SYMBOLS: RwLock<HashMap<String, Symbol>> = RwLock::new(HashMap::new());
}

/// A symbol that uses string interning
#[derive(Clone, Hash, Eq, Ord)]
pub struct Symbol(Arc<String>);

impl Symbol {
    /// Create a new symbol from a string
    /// 
    /// If the symbol already exists in the symbol table, it will return the existing symbol.
    /// Otherwise, it will create a new symbol and add it to the symbol table.
    pub fn new(name: &str) -> Self {
        // Check if the symbol already exists
        let mut symbols = SYMBOLS.write().unwrap();
        // If the symbol already exists, return it
        if let Some(symbol) = symbols.get(name) {
            return symbol.clone();
        }

        // Otherwise, create a new symbol
        let symbol = Symbol(Arc::new(name.to_string()));
        // Add the symbol to the symbol table
        symbols.insert(name.to_string(), symbol.clone());
        symbol
    }

    /// Get the name of the symbol as a string
    /// 
    /// This is useful when you need the internal string representation of the symbol.
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Convert a &str to a symbol conveniently
/// 
/// This allows you to pass a string to a function that expects a symbol,
/// using the `into()` method.
impl From<&str> for Symbol {
    #[inline]
    fn from(s: &str) -> Self {
        Symbol::new(s)
    }
}

/// Convert a String to a symbol conveniently
/// 
/// This allows you to pass a string to a function that expects a symbol,
/// using the `into()` method.
impl From<String> for Symbol {
    #[inline]
    fn from(s: String) -> Self {
        Symbol::new(&s)
    }
}

/// Compare two symbols for equality
/// 
/// This allows you to compare two symbols using the `==` operator.
/// First, it checks if the two symbols are the same object in memory.
/// If they are not, it compares the internal strings of the symbols.
/// 
/// This is faster than comparing the strings directly, because a pointer comparison
/// is faster than a string comparison.
impl PartialEq for Symbol {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // Check if the two symbols are the same object in memory
        if Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }
        // Compare the internal strings of the symbols
        self.0 == other.0
    }
}

/// Compare two symbols for ordering.
/// 
/// If the two symbols are the same object in memory, they are equal.
/// Otherwise, it compares the internal strings of the symbols.
impl PartialOrd for Symbol {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if Arc::ptr_eq(&self.0, &other.0) {
            return Some(std::cmp::Ordering::Equal);
        }
        self.0.partial_cmp(&other.0)
    }
}

/// Print a symbol as debug output
/// 
/// Since a symbol is meant to be an identifier, it is printed as a normal string.
/// This is useful for debugging, because it allows you to distinguish symbols from strings.
impl Debug for Symbol {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.0)
    }
}


///////////////////////////////////////////////////////////////
// ENVIRONMENT AND EVALUATION
///////////////////////////////////////////////////////////////

/*
 * The environment is a data structure that stores the bindings of values to
 * other values. This is used to evaluate lisp expressions, by looking up the
 * value of atoms in the environment, and performing operations on them.
 * 
 * The environment is also used to store built-in functions, variable bindings,
 * or override the default behavior of other atoms. This allows you to extend
 * the language with new functionality, or to create your own standard library
 * of functions and variables.
 * 
 * Evaluation is performed with the `eval` or `eval_str` methods, which take
 * expressions and evaluates them in the current environment. This allows you
 * to evaluate lisp expressions in a controlled context.
 */


/// An environment for evaluating lisp expressions
#[derive(Debug, Default, Clone)]
pub struct Env {
    /// The bindings in the environment.
    /// 
    /// This can store variable bindings to values, but also bindings from
    /// other atoms. For example, the atom `5` can be bound to the atom `10`.
    bindings: Arc<HashMap<Expr, Arc<Expr>>>,
}

impl Env {
    /// Create a new environment with no bindings.
    /// 
    /// This is useful when you want to create a new environment from scratch,
    /// to populate your own standard library of functions and variables.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind a symbol to a value in the environment.
    #[inline]
    pub fn bind_symbol(&mut self, symbol: &str, value: Expr) {
        self.bind(Expr::Symbol(Symbol::new(symbol)), value);
    }

    /// Bind a builtin function to a symbol in the environment.
    #[inline]
    pub fn bind_builtin(&mut self, symbol: &'static str, f: fn(&mut Env, Vec<Expr>) -> Expr) {
        self.bind_symbol(symbol, Expr::Builtin(Builtin::new(f, symbol)));
    }

    /// Bind a lazy builtin function to a symbol in the environment.
    /// 
    /// A lazy builtin function does not evaluate the result of the call
    /// after the function is called. Instead, it returns the result without
    /// evaluating it. This is helpful for defining certain special forms.
    #[inline]
    pub fn bind_lazy_builtin(&mut self, symbol: &'static str, f: fn(&mut Env, Vec<Expr>) -> Expr) {
        self.bind_symbol(
            symbol,
            Expr::Builtin(Builtin::new(f, symbol).with_lazy_eval(true)),
        );
    }

    /// Merge the bindings of another environment into this one.
    /// 
    /// This will overwrite any existing bindings with the same key, preferring
    /// the bindings from the incoming `other` environment.
    #[inline]
    pub fn merge(&mut self, other: &Env) {
        for (k, v) in other.bindings.iter() {
            self.bind(k.clone(), (**v).clone());
        }
    }

    /// Duplicate a binding for one variable to another.
    /// 
    /// The value stored in the `from` variable will be copied to the `to` variable.
    #[inline]
    pub fn alias(&mut self, from: impl Into<Symbol>, to: impl Into<Symbol>) {
        let from = from.into();
        let to = to.into();
        if let Some(value) = self.get(&Expr::Symbol(from)) {
            self.bind(Expr::Symbol(to), value.clone());
        }
    }

    /// Get the bindings in the environment.
    /// 
    /// This returns a copy of the bindings in the environment, from expressions
    /// to their assigned values. This is useful for debugging and introspection.
    pub fn get_bindings(&self) -> HashMap<Expr, Expr> {
        self.bindings
            .iter()
            .map(|(k, v)| (k.clone(), (**v).clone()))
            .collect()
    }

    /// Bind a value to another value in the environment.
    /// Typically this is used to bind a symbol to a value.
    /// 
    /// This will overwrite any existing binding for the symbol.
    #[inline]
    pub fn bind(&mut self, symbol: Expr, value: Expr) {
        if self.get(&symbol) == Some(&value) {
            return;
        }
        let bindings = Arc::make_mut(&mut self.bindings);
        bindings.insert(symbol, Arc::new(value));
    }

    /// Get the value assigned to an expression in the environment.
    #[inline]
    pub fn get(&self, symbol: &Expr) -> Option<&Expr> {
        self.bindings.get(symbol).map(|v| v.as_ref())
    }

    /// Remove a binding from the environment. This will unbind the value assigned
    /// to the expression, if it exists, so that it is no longer accessible.
    pub fn unbind(&mut self, symbol: &Expr) {
        let bindings = Arc::make_mut(&mut self.bindings);
        bindings.remove(symbol);
        self.bindings = Arc::new(bindings.clone());
    }

    /// Evaluate a string as an expression. This parses the input string and evaluates
    /// the resulting expression in the environment.
    #[inline]
    pub fn eval_str(&mut self, input: impl ToString) -> Result<Expr, String> {
        let input = input.to_string();
        let expr = Expr::parse(&input)?;
        Ok(self.eval(expr))
    }

    /// Evaluate an expression in this environment.
    /// 
    /// This will evaluate the expression in the current environment, and return the result.
    /// It uses tail recursion to evaluate the expression, so that it can handle deeply nested
    /// function calls without overflowing the stack.
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


///////////////////////////////////////////////////////////////
// BUILTIN FUNCTIONS
///////////////////////////////////////////////////////////////

/*
 * Builtin functions are supplied to the interpreter as functions that are
 * defined in Rust. These functions can be called from the lisp environment,
 * and can be used to extend the language with new functionality.
 * 
 * All special forms, operators, and standard library functions are implemented
 * as built-in functions. This allows you to create your own standard library
 * of functions, and to override the default behavior of the interpreter.
 * 
 * Builtin functions can be defined with the `Builtin::new` constructor, which
 * takes a function pointer and a name for the function. You can also set the
 * `lazy_eval` flag to true, to make the function's return value lazy-evaluated.
 */

 /// A builtin function that can be called from the lisp environment.
 /// 
 /// This is a wrapped Rust function that can implement special forms,
 /// operators, or standard library functions.
#[derive(Debug, Clone)]
pub struct Builtin {
    /// The function pointer for the builtin function.
    /// 
    /// This is a Rust function that takes the calling environment and a list of arguments,
    /// and returns the result of the function. The function can perform any operation
    /// on the arguments, and can return any expression as a result.
    pub f: fn(&mut Env, Vec<Expr>) -> Expr,
    /// The name of the builtin function.
    pub name: &'static str,
    /// Whether the builtin function should be evaluated lazily, or immediately after calling.
    pub(crate) lazy_eval: bool,
}

impl Builtin {
    /// Create a new builtin function from a function pointer and a name.
    #[inline]
    pub fn new(f: fn(&mut Env, Vec<Expr>) -> Expr, name: &'static str) -> Self {
        Self {
            f,
            name,
            lazy_eval: false,
        }
    }

    /// Set the lazy evaluation flag for the builtin function.
    /// 
    /// This will make the function return the result of the call without evaluating it afterwards.
    /// Using this can help prevent infinite loops, or to define certain special forms.
    #[inline]
    pub fn with_lazy_eval(self, lazy_eval: bool) -> Self {
        Self { lazy_eval, ..self }
    }

    /// Apply the builtin function to a list of arguments in the given environment.
    #[inline]
    pub fn apply(&self, env: &mut Env, args: Vec<Expr>) -> Expr {
        (self.f)(env, args)
    }
}

/// Implement display for builtin functions.
/// 
/// This allows you to print a builtin function as a string, which is useful for debugging.
/// It will print the name of the function, so that you can see which function is being called.
impl Display for Builtin {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "<builtin {}>", self.name)
    }
}


///////////////////////////////////////////////////////////////
// LISP EXPRESSIONS
///////////////////////////////////////////////////////////////

/*
 * Lisp expressions are the core data structure of the interpreter.
 * 
 * They represent the syntax of the lisp language, and can be used to
 * represent any kind of data or operation that can be performed in lisp.
 * 
 * All expressions are stored as an enum, which allows for a flexible
 * representation of data.
 */

/// A lisp expression to be evaluated
#[derive(Debug, Default, Clone)]
pub enum Expr {
    /// The unit value (nil)
    #[default]
    None,

    /// A floating point number
    Float(f64),
    /// A signed 64-bit integer
    Int(i64),
    /// A string
    String(String),
    /// A symbol
    Symbol(Symbol),
    /// A boolean
    Bool(bool),

    /// A list of expressions.
    /// 
    /// When evaluated, this is used to represent function calls, where the first element
    /// is the function to call, and the rest of the elements are the arguments.
    ///
    /// When used as a data structure, this is used to represent a list of values.
    /// It can be indexed by position, and can be used to store a sequence of values.
    /// 
    /// This is stored as a vector, not a linked list, so that it can be indexed efficiently.
    List(Vec<Expr>),
    /// An ordered map of expressions.
    /// 
    /// This is helpful when the user desires the ordered properties of a BTreeMap,
    /// such as ordering by key, but with worse time complexities than a HashMap.
    Tree(BTreeMap<Expr, Expr>),
    /// A map of expressions.
    /// 
    /// This is helpful when the user wants the time complexities associated with
    /// a HashMap, such as O(1) for insertion, deletion, and lookup, but with no ordering.
    Map(HashMap<Expr, Expr>),

    /// A block of expressions to be evaluated in order.
    /// 
    /// This is used to group multiple expressions together, and to evaluate them in sequence.
    /// The result of the block is the result of the last expression in the block.
    /// 
    /// This is useful for defining functions, where you want to evaluate multiple expressions
    /// in order, and return the result of the last expression as the result of the function.
    Many(Arc<Vec<Expr>>),

    /// A quoted expression.
    /// 
    /// This allows for lazy evaluation of expressions: when a quoted expression is evaluated,
    /// it returns the expression itself, without evaluating it. This is useful for defining
    /// special forms, or for returning unevaluated expressions from functions, like symbols.
    Quote(Box<Expr>),
    /// An error.
    /// 
    /// When an error occurs during evaluation, this is used to wrap an error value
    /// that is propagated up the call stack. This allows for error handling in the interpreter.
    Err(Box<Self>),

    /// A function closure.
    /// 
    /// This is used to represent a function that takes arguments and returns a value.
    /// The function is defined by a list of arguments, and a body expression to evaluate.
    /// 
    /// Internally, the function also keeps track of the environment in which it was defined,
    /// which allows it to capture bindings to variables defined outside the function.
    Function(Option<Box<Env>>, Vec<Expr>, Box<Expr>),
    /// A builtin function.
    /// 
    /// This is used to represent a function that is defined in Rust, and can be called from lisp.
    Builtin(Builtin),
}

/// Convert a String to an Expr conveniently.
/// 
/// This will return a Lisp expression that represents the string, not a symbol.
impl From<String> for Expr {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}
/// Convert a &str to an Expr conveniently.
/// 
/// This will return a Lisp expression that represents the string, not a symbol.
impl From<&str> for Expr {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

/// Convert an i64 to an Expr conveniently.
/// 
/// This will return a Lisp expression that represents the integer.
impl From<i64> for Expr {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

/// Convert an f64 to an Expr conveniently.
/// 
/// This will return a Lisp expression that represents the floating point number.
impl From<f64> for Expr {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

/// Convert a list of expressions to an Expr conveniently.
/// 
/// This will return a Lisp expression that represents the list of expressions.
impl<T> From<Vec<T>> for Expr
where
    T: Into<Expr>,
{
    fn from(v: Vec<T>) -> Self {
        Self::List(v.into_iter().map(|e| e.into()).collect())
    }
}

/// Allow Serde to convert a serde_json::Value to an Expr.
/// 
/// This is a convenience method for converting JSON data into Lisp expressions,
/// which will allow us to connect the interpreter to other systems that can serialize
/// data into JSON, and then deserialize it into Lisp expressions.
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

/// Allow Serde to convert an Expr to a serde_json::Value.
/// 
/// This is a convenience method for converting Lisp expressions into JSON data,
/// which will allow us to connect the interpreter to other systems that can deserialize
/// JSON data into Lisp expressions, and then serialize it into JSON data.
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


impl Expr {
    /// Serialize a value into a Lisp expression.
    #[inline]
    pub fn serialize<T: Serialize>(x: T) -> Self {
        serde_json::to_value(&x).unwrap().into()
    }

    /// Deserialize a Lisp expression into a value.
    #[inline]
    pub fn deserialize<T: DeserializeOwned>(x: &Self) -> Result<T, serde_json::Error> {
        serde_json::from_value::<T>(x.clone().into())
    }

    /// Create a symbol Lisp expression from a string.
    #[inline]
    pub fn symbol(name: impl ToString) -> Self {
        Self::Symbol(Symbol::new(&name.to_string()))
    }

    /// Wrap another expression in an error value.
    /// 
    /// This is useful for propagating errors up the call stack, and for handling errors in the interpreter.
    #[inline]
    pub fn error(message: impl Into<Self>) -> Self {
        Self::Err(Box::new(message.into()))
    }

    /// Quote an expression to prevent it from being evaluated.
    #[inline]
    pub fn quote(&self) -> Self {
        Self::Quote(Box::new(self.clone()))
    }

    /// Apply a callable value to a list of arguments.
    #[inline]
    pub fn apply(&self, args: &[Self]) -> Self {
        let mut result = vec![self.clone()];
        result.extend(args.to_vec());
        Self::List(result)
    }

    /// Parse a string into a Lisp expression.
    /// 
    /// If the string is a valid Lisp expression, it will return the parsed expression.
    /// If the string is not a valid Lisp expression, it will return an error message.
    pub fn parse(input: &str) -> Result<Expr, String> {
        let input = Self::remove_comments(input);
        let result = parser::parse_program::<VerboseError<&str>>(input.trim())
            .map(|(_, expr)| expr)
            .map_err(|e| match e {
                Err::Error(e) | Err::Failure(e) => convert_error::<&str>(&input, e),
                Err::Incomplete(e) => unreachable!("Incomplete: {:?}", e),
            });
        result
    }

    /// Strip the comments from an input string.
    /// 
    /// This is used to remove comments from a string before parsing it.
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
            }
        }
        output
    }
}

/// Compare two expressions for equality.
/// 
/// This allows you to compare two expressions using the `==` operator.
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

/// Compare two expressions for ordering.
/// 
/// This allows you to compare two expressions using the `<`, `>`, `<=`, and `>=` operators,
/// as well as to sort expressions in a collection.
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

/// Compare two expressions for strong equality, where a == b and b == a.
impl Eq for Expr {}

/// Compare two expressions for strong ordering, where a < b and b > a.
impl Ord for Expr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Less)
    }
}

/// Hash an expression for use in a hash map or set.
/// 
/// This allows you to use expressions as keys in a hash map or set, which is useful
/// for storing data in a way that allows for fast lookups and comparisons.
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

/// Implement display for Lisp expressions.
/// 
/// This allows you to print a Lisp expression as a string, which is useful for debugging.
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
