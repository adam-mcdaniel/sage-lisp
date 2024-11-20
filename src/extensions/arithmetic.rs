use crate::{Env, Expr};

pub fn add_bindings(env: &mut Env) {
    env.bind_builtin("range", range);
    env.bind_builtin("sum", sum);
    env.bind_builtin("+", sum);
    env.bind_builtin("difference", difference);
    env.bind_builtin("-", difference);
    env.bind_builtin("product", product);
    env.bind_builtin("*", product);
    env.bind_builtin("quotient", quotient);
    env.bind_builtin("/", quotient);
    env.bind_builtin("remainder", remainder);
    env.bind_builtin("%", remainder);
    env.bind_builtin("=", equality);
    env.bind_builtin("!=", inequality);
    env.bind_builtin("<", less_than);
    env.bind_builtin("<=", less_than_or_equal);
    env.bind_builtin(">", greater_than);
    env.bind_builtin(">=", greater_than_or_equal);
    env.bind_builtin("pow", raise_to_power);
    env.bind_builtin("sqrt", square_root);
    env.bind_builtin("abs", absolute_value);
    env.bind_builtin("round", round);
    env.bind_builtin("floor", floor);
    env.bind_builtin("ceiling", ceiling);
    env.bind_builtin("truncate", truncate);
    env.bind_builtin("abs_diff", absolute_difference);
    env.bind_builtin("max", maximum);
    env.bind_builtin("min", minimum);

    env.bind_builtin("&&", boolean_and);
    env.bind_builtin("||", boolean_or);
    env.bind_builtin("!", boolean_not);
    env.bind_builtin("&", bitwise_and);
    env.bind_builtin("|", bitwise_or);
    env.bind_builtin("^", bitwise_xor);
    env.bind_builtin("~", bitwise_not);
    env.bind_builtin("<<", left_shift);
    env.bind_builtin(">>", arithmetic_right_shift);
    env.bind_builtin(">>>", logical_right_shift);
}

fn range_to_list(start: i64, end: i64) -> Expr {
    let mut list = vec![];
    for i in start..end {
        list.push(Expr::Int(i));
    }
    Expr::List(list)
}

fn range(env: &mut Env, low_and_high: Vec<Expr>) -> Expr {
    let low = env.eval(low_and_high.get(0).cloned().unwrap_or(Expr::Int(0)));
    let high = env.eval(low_and_high.get(1).cloned().unwrap_or(Expr::Int(0)));
    
    match (low, high) {
        (Expr::Int(low), Expr::Int(high)) => range_to_list(low, high),
        (low, high) => Expr::error(format!("Invalid range {}..{}", low, high)),
    }
}


fn boolean_and(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    match (a, b) {
        (Expr::Bool(a), Expr::Bool(b)) => Expr::Bool(a && b),
        (a, b) => Expr::error(format!("Invalid expr {} && {}", a, b)),
    }
}

fn boolean_or(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    match (a, b) {
        (Expr::Bool(a), Expr::Bool(b)) => Expr::Bool(a || b),
        (a, b) => Expr::error(format!("Invalid expr {} || {}", a, b)),
    }
}

fn boolean_not(env: &mut Env, expr: Vec<Expr>) -> Expr {
    let a = env.eval(expr[0].clone());
    match a {
        Expr::Bool(a) => Expr::Bool(!a),
        a => Expr::error(format!("Invalid expr !{}", a)),
    }
}

fn bitwise_and(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    match (a, b) {
        (Expr::Int(a), Expr::Int(b)) => Expr::Int(a & b),
        (a, b) => Expr::error(format!("Invalid expr {} & {}", a, b)),
    }
}

fn bitwise_or(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    match (a, b) {
        (Expr::Int(a), Expr::Int(b)) => Expr::Int(a | b),
        (a, b) => Expr::error(format!("Invalid expr {} | {}", a, b)),
    }
}

fn bitwise_xor(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    match (a, b) {
        (Expr::Int(a), Expr::Int(b)) => Expr::Int(a ^ b),
        (a, b) => Expr::error(format!("Invalid expr {} ^ {}", a, b)),
    }
}

fn bitwise_not(env: &mut Env, expr: Vec<Expr>) -> Expr {
    let a = env.eval(expr[0].clone());
    match a {
        Expr::Int(a) => Expr::Int(!a),
        a => Expr::error(format!("Invalid expr ~{}", a)),
    }
}

fn left_shift(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    match (a, b) {
        (Expr::Int(a), Expr::Int(b)) => Expr::Int(a << b),
        (a, b) => Expr::error(format!("Invalid expr {} << {}", a, b)),
    }
}

fn logical_right_shift(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    match (a, b) {
        (Expr::Int(a), Expr::Int(b)) => Expr::Int((a as u64 >> b) as i64),
        (a, b) => Expr::error(format!("Invalid expr {} >> {}", a, b)),
    }
}

fn arithmetic_right_shift(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    match (a, b) {
        (Expr::Int(a), Expr::Int(b)) => Expr::Int(a >> b),
        (a, b) => Expr::error(format!("Invalid expr {} >>> {}", a, b)),
    }
}

fn sum(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let mut sum = Expr::default();
    for e in exprs {
        let e = env.eval(e.clone());
        // sum += env.eval(e);
        match (sum, e) {
            (Expr::None, b) => sum = b,
            (Expr::Int(a), Expr::Int(b)) => sum = Expr::Int(a + b),
            (Expr::Float(a), Expr::Float(b)) => sum = Expr::Float(a + b),
            (Expr::Int(a), Expr::Float(b)) => sum = Expr::Float(a as f64 + b),
            (Expr::Float(a), Expr::Int(b)) => sum = Expr::Float(a + b as f64),
            (Expr::String(a), Expr::String(b)) => sum = Expr::String(format!("{}{}", a, b)),
            (Expr::List(a), Expr::List(b)) => {
                let mut list = a.clone();
                list.extend(b);
                sum = Expr::List(list);
            }
            (Expr::List(a), b) => {
                let mut list = a.clone();
                list.push(b);
                sum = Expr::List(list);
            }
            (a, b) => return Expr::error(format!("Invalid expr {} + {}", a, b)),
        }
    }
    sum
}

fn difference(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let mut diff = Expr::default();
    for e in exprs {
        let e = env.eval(e.clone());
        match (diff, e) {
            (Expr::None, b) => diff = b,
            (Expr::Int(a), Expr::Int(b)) => diff = Expr::Int(a - b),
            (Expr::Float(a), Expr::Float(b)) => diff = Expr::Float(a - b),
            (Expr::Int(a), Expr::Float(b)) => diff = Expr::Float(a as f64 - b),
            (Expr::Float(a), Expr::Int(b)) => diff = Expr::Float(a - b as f64),
            (a, b) => return Expr::error(format!("Invalid expr {} - {}", a, b)),
        }
    }
    diff
}

fn product(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let mut product = Expr::default();
    for e in exprs {
        let e = env.eval(e.clone());
        match (product, e) {
            (Expr::None, b) => product = b,
            (Expr::Int(a), Expr::Int(b)) => product = Expr::Int(a * b),
            (Expr::Float(a), Expr::Float(b)) => product = Expr::Float(a * b),
            (Expr::Int(a), Expr::Float(b)) => product = Expr::Float(a as f64 * b),
            (Expr::Float(a), Expr::Int(b)) => product = Expr::Float(a * b as f64),
            (Expr::List(a), Expr::Int(b)) => {
                let mut list = a.clone();
                for _ in 0..b {
                    list.extend(a.clone());
                }
                product = Expr::List(list);
            }
            (a, b) => return Expr::error(format!("Invalid expr {} * {}", a, b)),
        }
    }
    product
}

fn quotient(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let mut quotient = Expr::default();
    for e in exprs {
        let e = env.eval(e.clone());
        match (quotient, e) {
            (Expr::None, b) => quotient = b,
            (Expr::Int(a), Expr::Int(b)) => quotient = Expr::Int(a / b),
            (Expr::Float(a), Expr::Float(b)) => quotient = Expr::Float(a / b),
            (Expr::Int(a), Expr::Float(b)) => quotient = Expr::Float(a as f64 / b),
            (Expr::Float(a), Expr::Int(b)) => quotient = Expr::Float(a / b as f64),
            (a, b) => return Expr::error(format!("Invalid expr {} / {}", a, b)),
        }
    }
    quotient
}

fn remainder(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let mut quotient = Expr::default();
    for e in exprs {
        let e = env.eval(e.clone());
        match (quotient, e) {
            (Expr::None, b) => quotient = b,
            (Expr::Int(a), Expr::Int(b)) => quotient = Expr::Int(a % b),
            (Expr::Float(a), Expr::Float(b)) => quotient = Expr::Float(a % b),
            (Expr::Int(a), Expr::Float(b)) => quotient = Expr::Float(a as f64 % b),
            (Expr::Float(a), Expr::Int(b)) => quotient = Expr::Float(a % b as f64),
            (a, b) => return Expr::error(format!("Invalid expr {} % {}", a, b)),
        }
    }
    quotient
}

fn equality(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    Expr::Bool(a == b)
}

fn inequality(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    Expr::Bool(a != b)
}

fn less_than(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    Expr::Bool(a < b)
}

fn less_than_or_equal(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    Expr::Bool(a <= b)
}

fn greater_than(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    Expr::Bool(a > b)
}

fn greater_than_or_equal(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());

    Expr::Bool(a >= b)
}

fn raise_to_power(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let base = env.eval(exprs[0].clone());
    let exponent = env.eval(exprs[1].clone());
    match (base, exponent) {
        (Expr::Int(base), Expr::Int(exponent)) => Expr::Float((base as f64).powf(exponent as f64)),
        (Expr::Float(base), Expr::Float(exponent)) => Expr::Float(base.powf(exponent)),
        (Expr::Int(base), Expr::Float(exponent)) => Expr::Float((base as f64).powf(exponent)),
        (Expr::Float(base), Expr::Int(exponent)) => Expr::Float(base.powf(exponent as f64)),
        (base, exponent) => Expr::error(format!("Invalid base {base} and exponent {exponent}"))
    }
}

fn square_root(env: &mut Env, expr: Vec<Expr>) -> Expr {
    let square = env.eval(expr[0].clone());
    match square {
        Expr::Int(square) => Expr::Float((square as f64).sqrt()),
        Expr::Float(square) => Expr::Float(square.sqrt()),
        val => Expr::error(format!("Invalid square root of {val}"))
    }
}

fn absolute_value(env: &mut Env, expr: Vec<Expr>) -> Expr {
    let n = env.eval(expr[0].clone());
    match n {
        Expr::Int(n) => Expr::Int(n.abs()),
        Expr::Float(n) => Expr::Float(n.abs()),
        n => Expr::error(format!("Invalid absolute value of {n}"))
    }
}

fn round(env: &mut Env, expr: Vec<Expr>) -> Expr {
    let n = env.eval(expr[0].clone());
    match n {
        Expr::Int(n) => Expr::Int(n),
        Expr::Float(n) => Expr::Int(n.round() as i64),
        n => Expr::error(format!("Invalid value to round {n}"))
    }
}

fn floor(env: &mut Env, expr: Vec<Expr>) -> Expr {
    let n = env.eval(expr[0].clone());
    match n {
        Expr::Int(n) => Expr::Int(n),
        Expr::Float(n) => Expr::Int(n.floor() as i64),
        n => Expr::error(format!("Invalid value to floor {n}"))
    }
}

fn ceiling(env: &mut Env, expr: Vec<Expr>) -> Expr {
    let n = env.eval(expr[0].clone());
    match n {
        Expr::Int(n) => Expr::Int(n),
        Expr::Float(n) => Expr::Int(n.ceil() as i64),
        n => Expr::error(format!("Invalid value to ceiling {n}"))
    }
}

fn truncate(env: &mut Env, expr: Vec<Expr>) -> Expr {
    let n = env.eval(expr[0].clone());
    match n {
        Expr::Int(n) => Expr::Int(n),
        Expr::Float(n) => Expr::Int(n.trunc() as i64),
        n => Expr::error(format!("Invalid value to truncate {n}"))
    }
}

fn absolute_difference(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let a = env.eval(exprs[0].clone());
    let b = env.eval(exprs[1].clone());
    match (a, b) {
        (Expr::Int(a), Expr::Int(b)) => Expr::Int((a - b).abs()),
        (Expr::Float(a), Expr::Float(b)) => Expr::Float((a - b).abs()),
        (Expr::Int(a), Expr::Float(b)) => Expr::Float((a as f64 - b).abs()),
        (Expr::Float(a), Expr::Int(b)) => Expr::Float((a - b as f64).abs()),
        (a, b) => Expr::error(format!("Invalid expr {} - {}", a, b)),
    }
}

fn maximum(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let mut max = Expr::default();
    for e in exprs {
        let e = env.eval(e.clone());
        if max < e {
            max = e;
        }
    }
    max
}

fn minimum(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let mut min = Expr::default();
    for e in exprs {
        let e = env.eval(e.clone());
        if min > e {
            min = e;
        }
    }
    min
}

