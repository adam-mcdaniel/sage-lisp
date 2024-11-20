use crate::Env;

#[cfg(feature = "rand")]
use crate::Expr;
#[cfg(feature = "rand")]
use rand::Rng;


#[cfg(feature = "rand")]
pub fn add_bindings(env: &mut Env) {
    env.bind_builtin("integer", random_integer);
    env.bind_builtin("choice", random_choice);
    env.bind_builtin("shuffle", random_shuffle);
}

#[cfg(not(feature = "rand"))]
pub fn add_bindings(_env: &mut Env) {
    eprintln!("Warning: Random extension not available. Please enable the 'rand' feature.");
}

#[cfg(feature = "rand")]
fn random_integer(env: &mut Env, args: Vec<Expr>) -> Expr {
    let min = match env.eval(args.get(0).cloned().unwrap_or(Expr::None)) {
        Expr::Int(min) => min,
        _ => i64::MIN,
    };
    let max = match env.eval(args.get(1).cloned().unwrap_or(Expr::None)) {
        Expr::Int(max) => max,
        _ => i64::MAX,
    };
    Expr::Int(rand::thread_rng().gen_range(min..=max))
}

#[cfg(feature = "rand")]
fn random_choice(env: &mut Env, args: Vec<Expr>) -> Expr {
    let mut rng = rand::thread_rng();
    
    let collection = env.eval(args.get(0).cloned().unwrap_or(Expr::None));
    
    match collection {
        Expr::List(list) => {
            let index = rng.gen_range(0..list.len());
            list.get(index).cloned().unwrap_or(Expr::None)
        },
        _ => Expr::error(format!("Expected a list, found {:?}", collection)),
    }
}

#[cfg(feature = "rand")]
fn random_shuffle(env: &mut Env, args: Vec<Expr>) -> Expr {
    use rand::prelude::SliceRandom;
    let mut rng = rand::thread_rng();
    let mut args = args.clone();
    args.shuffle(&mut rng);
    Expr::List(args)
}