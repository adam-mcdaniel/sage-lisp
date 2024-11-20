use crate::{Env, Expr};

pub fn add_bindings(env: &mut Env) {
    env.bind_builtin("env", env_as_map);
    env.bind_builtin("define", define_binding);
    env.bind_builtin("undefine", undefine_binding);
    env.bind_builtin("defun", define_function);
    env.bind_builtin("let", let_bindings_in_new_environment);
}

fn env_as_map(env: &mut Env, _exprs: Vec<Expr>) -> Expr {
    return Expr::Map(env.get_bindings());
}

fn define_binding(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let name = exprs[0].clone();
    let value = env.eval(exprs[1].clone());
    env.bind(name, value.clone());
    value
}

fn undefine_binding(env: &mut Env, expr: Vec<Expr>) -> Expr {
    let name = expr[0].clone();
    env.unbind(&name);
    Expr::None
}

fn define_function(env: &mut Env, args: Vec<Expr>) -> Expr {
    if args.len() != 3 {
        return Expr::error(format!("Invalid function definition {:?}", args));
    }

    let name = args[0].clone();
    let params = args[1].clone();
    let body = args[2].clone();
    if let Expr::List(params) = params {
        let f = env.eval(Expr::Function(None, params, Box::new(body)));
        env.bind(name, f);
        Expr::None
    } else {
        return Expr::error(format!("Invalid params {:?}", params));
    }
}

fn let_bindings_in_new_environment(env: &mut Env, args: Vec<Expr>) -> Expr {
    let mut new_env = env.clone();
    let bindings = args[0].clone();
    let body = args[1].clone();
    match bindings {
        Expr::List(bindings) => {
            for binding in bindings {
                if let Expr::List(binding) = binding {
                    let name = binding[0].clone();
                    let value = env.eval(binding[1].clone());
                    new_env.bind(name, value);
                } else {
                    return Expr::error(format!("Invalid let-binding {binding}"));
                }
            }
        }
        Expr::Map(bindings) => {
            for (name, value) in bindings {
                new_env.bind(name, env.eval(value));
            }
        }
        Expr::Tree(bindings) => {
            for (name, value) in bindings {
                new_env.bind(name, env.eval(value));
            }
        }
        bindings => return Expr::error(format!("Invalid let-bindings {bindings}")),
    }
    new_env.eval(body)
}