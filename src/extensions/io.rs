use crate::{Env, Expr};

pub fn add_bindings(env: &mut Env) {
    env.bind_builtin("println", println);
    env.bind_builtin("print", print);
}

fn println(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    print(env, exprs);
    println!();
    Expr::None
}

fn print(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    for e in exprs {
        let e = env.eval(e.clone());

        match e {
            Expr::String(s) => print!("{}", s),
            Expr::Symbol(s) => print!("{}", s.name()),
            _ => print!("{}", e),
        }
    }
    Expr::None
}