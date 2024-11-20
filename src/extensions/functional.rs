use crate::{Env, Expr};

pub fn add_bindings(env: &mut Env) {
    env.bind_builtin("eval", eval);
    env.bind_builtin("quote", quote);
    env.bind_builtin("map", map_container);
    env.bind_builtin("filter", filter_container);
    env.bind_builtin("reduce", reduce_container);
    env.bind_builtin("fn", anonymous_function);
    env.bind_builtin("\\", anonymous_function);
    env.bind_builtin("lambda", anonymous_function);
    env.bind_builtin("apply", apply_function);
}

fn eval(env: &mut Env, expr: Vec<Expr>) -> Expr {
    if expr.is_empty() {
        return Expr::error(format!("Eval requires an argument"));
    } else if expr.len() > 1 {
        return Expr::error(format!("Eval requires only one argument"));
    }
    let e = env.eval(expr[0].clone());
    env.eval(e)
}

fn quote(_env: &mut Env, expr: Vec<Expr>) -> Expr {
    if expr.is_empty() {
        return Expr::error(format!("Quote requires an argument"));
    } else if expr.len() > 1 {
        return Expr::error(format!("Quote requires only one argument"));
    }
    expr[0].clone()
}

pub(crate) fn anonymous_function(env: &mut Env, params_and_body: Vec<Expr>) -> Expr {
    if params_and_body.len() != 2 {
        return Expr::error(format!("Invalid function definition {:?}", params_and_body));
    }
    let params = params_and_body[0].clone();
    let body = params_and_body[1].clone();
    if let Expr::List(params) = params {
        Expr::Function(Some(Box::new(env.clone())), params, Box::new(body))
    } else {
        return Expr::error(format!("Invalid params {:?}", params));
    }
}

fn apply_function(env: &mut Env, function_and_args: Vec<Expr>) -> Expr {
    if function_and_args.len() != 2 {
        return Expr::error(format!("Invalid function apply {:?}", function_and_args));
    }
    let f = env.eval(function_and_args[0].clone());
    let args = env.eval(function_and_args[1].clone());
    if let Expr::List(args) = args {
        match f {
            Expr::Function(Some(mut env), params, body) => {
                let mut new_env = env.clone();
                for (param, arg) in params.iter().zip(args.iter()) {
                    new_env.bind(param.clone(), env.eval(arg.clone()));
                }
                new_env.eval(*body.clone())
            }
            Expr::Function(None, params, body) => {
                let mut new_env = env.clone();
                for (param, arg) in params.iter().zip(args.iter()) {
                    new_env.bind(param.clone(), env.eval(arg.clone()));
                }
                new_env.eval(*body.clone())
            }
            Expr::Builtin(f) => f.apply(&mut env.clone(), args),
            f => Expr::error(format!("Invalid function {f} apply {}", Expr::from(args))),
        }
    } else {
        Expr::error(format!("Invalid function {f} apply {}", Expr::from(args)))
    }
}

fn map_container(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    if exprs.len() != 2 {
        return Expr::error(format!("Invalid number of map arguments {:?}", exprs));
    }
    let f = env.eval(exprs[0].clone());
    let container = env.eval(exprs[1].clone());
    match container {
        Expr::List(container) => {
            let mut list = vec![];
            for e in container {
                list.push(env.eval(Expr::List(vec![f.clone(), e])));
            }
            Expr::List(list)
        }
        Expr::Map(container) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in container {
                let pair = env.eval(Expr::List(vec![f.clone(), k.quote(), v.quote()]));
                if let Expr::List(pair) = pair {
                    map.insert(pair[0].clone(), pair[1].clone());
                } else {
                    return Expr::error(format!("Invalid pair {}", pair));
                }
            }
            Expr::Map(map)
        }
        Expr::Tree(container) => {
            let mut tree = std::collections::BTreeMap::new();
            for (k, v) in container {
                let pair = env.eval(Expr::List(vec![f.clone(), k.quote(), v.quote()]));
                if let Expr::List(pair) = pair {
                    tree.insert(pair[0].clone(), pair[1].clone());
                } else {
                    return Expr::error(format!("Invalid pair {}", pair));
                }
            }
            Expr::Tree(tree)
        }
        container => return Expr::error(format!("Invalid container for map {container}")),
    }
}

fn filter_container(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    if exprs.len() != 2 {
        return Expr::error(format!("Invalid number of filter arguments {:?}", exprs));
    }
    let f = env.eval(exprs[0].clone());
    let container = env.eval(exprs[1].clone());

    match container {
        Expr::List(container) => {
            let mut list = vec![];
            for e in container {
                if env.eval(Expr::List(vec![f.clone(), e.clone()])) == Expr::Bool(true) {
                    list.push(e);
                }
            }
            Expr::List(list)
        }
        Expr::Map(container) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in container {
                let x = env.eval(Expr::List(vec![f.clone(), k.quote(), v.quote()]));
                if x == Expr::Bool(true) {
                    map.insert(k, v);
                }
            }
            Expr::Map(map)
        }
        Expr::Tree(container) => {
            let mut tree = std::collections::BTreeMap::new();
            for (k, v) in container {
                let x = env.eval(Expr::List(vec![f.clone(), k.quote(), v.quote()]));
                if x == Expr::Bool(true) {
                    tree.insert(k, v);
                }
            }
            Expr::Tree(tree)
        }
        container => return Expr::error(format!("Invalid container for filter {container}")),
    }
}

fn reduce_container(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    if exprs.len() < 2 {
        return Expr::error(format!("Invalid number of reduce arguments {:?}", exprs));
    }
    if exprs.len() > 3 {
        return Expr::error(format!("Invalid number of reduce arguments {:?}", exprs));
    }
    let f = env.eval(exprs[0].clone());
    let container = env.eval(exprs[1].clone());
    let accumulator_init = if exprs.len() > 2 {
        env.eval(exprs[2].clone())
    } else {
        Expr::None
    };

    match container {
        Expr::List(container) => {
            let mut acc = accumulator_init;
            for e in container {
                acc = env.eval(Expr::List(vec![f.clone(), acc, e]));
            }
            acc
        }
        Expr::Map(container) => {
            let mut acc = accumulator_init;
            for (k, v) in container {
                acc = env.eval(Expr::List(vec![f.clone(), acc, k, v]));
            }
            acc
        }
        Expr::Tree(container) => {
            let mut acc = accumulator_init;
            for (k, v) in container {
                acc = env.eval(Expr::List(vec![f.clone(), acc, k, v]));
            }
            acc
        }
        container => return Expr::error(format!("Invalid container for reduce {container}")),
    }
}