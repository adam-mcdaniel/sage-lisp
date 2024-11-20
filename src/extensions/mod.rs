pub mod oop;
pub mod arithmetic;
pub mod functional;
pub mod environment;
pub mod system;
pub mod filesystem;
pub mod random;
pub mod list;
pub mod procedural;
pub mod io;
pub mod format;
pub mod modules;

pub fn env_to_map(env: &crate::Env) -> crate::Expr {
    crate::Expr::Map(env.get_bindings())
}

pub fn as_module(add_bindings: fn(&mut crate::Env)) -> crate::Expr {
    let mut env = crate::Env::new();
    add_bindings(&mut env);
    env_to_map(&mut env)
}

pub fn add_module(global_env: &mut crate::Env, module_name: impl ToString, add_bindings: fn(&mut crate::Env)) {
    let module = as_module(add_bindings);
    // Add the module to the global environment
    global_env.bind(crate::Expr::Symbol(module_name.to_string().into()), module);
}