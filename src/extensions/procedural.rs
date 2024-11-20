use crate::{Env, Expr};

pub fn add_bindings(env: &mut Env) {
    env.bind_builtin("while", while_loop);
    env.bind_builtin("for", for_loop);
    env.bind_lazy_builtin("if", if_statement);
}

/// Perform a loop while a condition is true.
/// 
/// # Example
/// 
/// ```lisp
/// (while (< i 10) {
///     (define i (+ i 1))
/// 
///     (print i)
/// }
/// ```
/// 
/// # Syntax
/// 
/// ```lisp
/// (while <condition> <body>)
/// ```
/// 
/// # Remarks
/// 
/// The condition expression is evaluated before each iteration of the loop.
/// The body expression is evaluated only if the condition is true.
fn while_loop(env: &mut Env, args: Vec<Expr>) -> Expr {
    if args.len() != 2 {
        return Expr::error("while: expected 2 arguments".to_string());
    }

    let condition = args.get(0).unwrap();
    let body = args.get(1).unwrap();

    let mut result = Expr::None;
    while env.eval(condition.clone()) == Expr::Bool(true) {
        result = env.eval(body.clone());
    }

    result
}

/// Perform a loop iterating over a list of elements.
/// 
/// # Example
/// 
/// ```lisp
/// (for i in (range 10) {
///    (print i)
/// })
/// ```
///
/// # Output
/// 
/// ```
/// 0
/// 1
/// 2
/// 3
/// 4
/// 5
/// 6
/// 7
/// 8
/// 9
/// ```
fn for_loop(env: &mut Env, args: Vec<Expr>) -> Expr {
    if args.len() != 4 {
        return Expr::error("for: expected 2 arguments".to_string());
    }

    let variable = args.get(0).unwrap();
    let iterable = env.eval(args.get(2).cloned().unwrap_or(Expr::None));
    let body = args.get(3).unwrap();

    let mut result = Expr::None;
    if let Expr::List(elements) = iterable {
        for element in elements {
            env.bind(variable.clone(), element);
            // Evaluate the body of the loop
            result = env.eval(body.clone());
        }
    } else {
        return Expr::error(format!("for: expected a list, found {iterable:?}"));
    }

    result
}


/// Conditional if-else expression.
/// 
/// # Example
/// 
/// ```lisp
/// (define x 5)
/// (if (< x 10) {
///     (println "x is less than 10")
/// } {
///     (println "x is greater than or equal to 10")
/// })
/// ```
/// 
/// ## Output
/// 
/// ```
/// x is less than 10
/// ```
fn if_statement(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let cond = env.eval(exprs[0].clone());
    let then = exprs[1].clone();
    if exprs.len() < 3 {
        if cond == Expr::Bool(true) {
            then
        } else {
            Expr::None
        }
    } else {
        let else_ = exprs[2].clone();
        if cond == Expr::Bool(true) {
            then
        } else {
            else_
        }
    }
}