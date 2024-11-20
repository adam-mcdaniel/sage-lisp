use crate::{Env, Expr};

pub fn add_bindings(env: &mut Env) {
    env.bind_builtin("use", use_module);
}

fn use_module(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    // Get the module
    let module = match env.eval(exprs.get(0).cloned().unwrap_or_default()) {
        Expr::Map(m) => m,
        Expr::Tree(m) => m.into_iter().collect(),
        _ => return Expr::error("use: expected a module".to_string()),
    };

    // If there are specific symbols to import, import them
    for expr in exprs.iter().skip(1) {
        match expr {
            Expr::Symbol(name) => {
                if let Some(value) = module.get(&Expr::symbol(name)) {
                    env.bind(Expr::Symbol(name.clone()), value.clone());
                } else {
                    return Expr::error(format!("use: symbol {} not found", name));
                }
            },
            _ => return Expr::error("use: expected a symbol".to_string()),
        }
    }

    if exprs.len() == 1 {
        for (name, value) in module {
            env.bind(name, value);
        }
    }

    Expr::None
}