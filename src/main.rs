use lisp::*;
use clap::Parser;

// const PROGRAM: &str = r#"
// (do
//     (defun fact (n) 
//         (if (<= n 0) 1 
//             (do 
//                 (* n (fact (- n 1))))))
//     (defun print-fact (n) (println n "! = " (fact n)))

//     (print-fact 5))
// "#;

#[derive(Parser)]
#[command(version, about)]
pub struct Program {
    // The program name, if any. A positional argument.
    #[arg(name = "program_name")]
    program_name: Option<String>,
    // A string to evaluate.
    #[arg(short='c', long)]
    program: Option<String>,
}

fn main() {
    // let mut program = PROGRAM.to_owned();

    let args = Program::parse();
    // Either open the file or use the program string.
    let mut program = match args.program {
        Some(ref program) => program.clone(),
        None => {
            match args.program_name {
                Some(ref program_name) => {
                    std::fs::read_to_string(program_name).unwrap()
                },
                None => {
                    panic!("No program provided");
                }
            }
        }
    };

    let e = Expr::parse(&mut program).unwrap();

    let mut env = Env::new();
    env.bind_builtin("+", |env, exprs| {
        let mut sum = Expr::default();
        for e in exprs {
            let e = env.eval(e.clone());
            // sum += env.eval(e);
            match (sum, env.eval(e)) {
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
                },
                (Expr::List(a), b) => {
                    let mut list = a.clone();
                    list.push(b);
                    sum = Expr::List(list);
                },
                (a, b) => panic!("Invalid expr {:?} + {:?}", a, b)
            }
        }
        sum
    });

    env.bind_builtin("-", 
        |env, exprs| {
            let mut diff = Expr::default();
            for e in exprs {
                let e = env.eval(e.clone());
                match (diff, e) {
                    (Expr::None, b) => diff = b,
                    (Expr::Int(a), Expr::Int(b)) => diff = Expr::Int(a - b),
                    (Expr::Float(a), Expr::Float(b)) => diff = Expr::Float(a - b),
                    (Expr::Int(a), Expr::Float(b)) => diff = Expr::Float(a as f64 - b),
                    (Expr::Float(a), Expr::Int(b)) => diff = Expr::Float(a - b as f64),
                    (a, b) => panic!("Invalid expr {:?} - {:?}", a, b)
                }
            }
            diff
        }
    );

    env.bind_builtin("*", |env, exprs| {
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
                },
                (a, b) => panic!("Invalid expr {:?} * {:?}", a, b)
            }
        }
        product
    });

    env.bind_builtin("/", |env, exprs| {
        let mut quotient = Expr::default();
        for e in exprs {
            let e = env.eval(e.clone());
            match (quotient, e) {
                (Expr::None, b) => quotient = b,
                (Expr::Int(a), Expr::Int(b)) => quotient = Expr::Int(a / b),
                (Expr::Float(a), Expr::Float(b)) => quotient = Expr::Float(a / b),
                (Expr::Int(a), Expr::Float(b)) => quotient = Expr::Float(a as f64 / b),
                (Expr::Float(a), Expr::Int(b)) => quotient = Expr::Float(a / b as f64),
                (a, b) => panic!("Invalid expr {:?} / {:?}", a, b)
            }
        }
        quotient
    });

    env.bind_builtin("%", |env, exprs| {
        let mut quotient = Expr::default();
        for e in exprs {
            let e = env.eval(e.clone());
            match (quotient, e) {
                (Expr::None, b) => quotient = b,
                (Expr::Int(a), Expr::Int(b)) => quotient = Expr::Int(a % b),
                (Expr::Float(a), Expr::Float(b)) => quotient = Expr::Float(a % b),
                (Expr::Int(a), Expr::Float(b)) => quotient = Expr::Float(a as f64 % b),
                (Expr::Float(a), Expr::Int(b)) => quotient = Expr::Float(a % b as f64),
                (a, b) => panic!("Invalid expr {:?} % {:?}", a, b)
            }
        }
        quotient
    });
    
    env.bind_builtin("=", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());
        
        Expr::Bool(a == b)
    });

    env.bind_builtin("!=", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());
        
        Expr::Bool(a != b)
    });

    env.bind_builtin("<=", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());
        
        Expr::Bool(a <= b)
    });

    env.bind_builtin(">=", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());
        
        Expr::Bool(a >= b)
    });

    env.bind_builtin("<", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());
        
        Expr::Bool(a < b)
    });

    env.bind_builtin(">", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());
        
        Expr::Bool(a > b)
    });

    env.bind_builtin("if", |env, exprs| {
        let cond = env.eval(exprs[0].clone());
        let then = exprs[1].clone();
        let else_ = exprs[2].clone();
        if cond == Expr::Bool(true) {
            env.eval(then)
        } else {
            env.eval(else_)
        }
    });

    env.bind_builtin("define", 
        |env, exprs| {
            let name = exprs[0].clone();
            let value = env.eval(exprs[1].clone());
            env
                .bind(name, value);
            Expr::None
        }
    );

    env.bind_builtin("defun", |env, args| {
        let name = args[0].clone();
        let params = args[1].clone();
        let body = args[2].clone();
        if let Expr::List(params) = params {
            let f = env.eval(Expr::Function(None, params, Box::new(body)));
            env.bind(name, f);
            Expr::None
        } else {
            panic!("Invalid params {:?}", params);
        }
    });

    env.bind_builtin("println", |env, exprs| {
        for e in exprs {
            let e = env.eval(e.clone());

            match e {
                Expr::String(s) => print!("{}", s),
                Expr::Symbol(s) => print!("{}", s.name()),
                _ => print!("{}", e)
            }
        }
        println!();
        Expr::None
    });

    env.bind_builtin("do", |env, exprs| {
        let mut result = Expr::default();
        for e in exprs {
            result = env.eval(e.clone());
        }
        result
    });

    env.bind_builtin("sqrt", |env, expr| {
        let e = env.eval(expr[0].clone());
        match e {
            Expr::Int(i) => Expr::Float((i as f64).sqrt()),
            Expr::Float(f) => Expr::Float(f.sqrt()),
            e => panic!("Invalid expr {:?}", e)
        }
    });

    env.bind_builtin("^", |env, expr| {
        let a = env.eval(expr[0].clone());
        let b = env.eval(expr[1].clone());
        match (a, b) {
            (Expr::Int(a), Expr::Int(b)) => Expr::Float((a as f64).powf(b as f64)),
            (Expr::Float(a), Expr::Float(b)) => Expr::Float(a.powf(b)),
            (Expr::Int(a), Expr::Float(b)) => Expr::Float((a as f64).powf(b)),
            (Expr::Float(a), Expr::Int(b)) => Expr::Float(a.powf(b as f64)),
            (a, b) => panic!("Invalid expr {:?} ^ {:?}", a, b)
        }
    });

    let lambda = |env: &mut Env, expr: &[Expr]| {
        let params = expr[0].clone();
        let body = expr[1].clone();
        if let Expr::List(params) = params {
            Expr::Function(Some(Box::new(env.clone())), params, Box::new(body))
        } else {
            panic!("Invalid params {:?}", params);
        }
    };
    env.bind_builtin("lambda", lambda);
    env.bind_builtin("\\", lambda);

    env.bind_builtin("apply", |env, expr| {
        let f = env.eval(expr[0].clone());
        let args = env.eval(expr[1].clone());
        if let Expr::List(args) = args {
            match f {
                Expr::Function(Some(mut env), params, body) => {
                    let mut new_env = env.clone();
                    for (param, arg) in params.iter().zip(args.iter()) {
                        new_env.bind(param.clone(), env.eval(arg.clone()));
                    }
                    new_env.eval(*body.clone())
                },
                Expr::Function(None, params, body) => {
                    let mut new_env = env.clone();
                    for (param, arg) in params.iter().zip(args.iter()) {
                        new_env.bind(param.clone(), env.eval(arg.clone()));
                    }
                    new_env.eval(*body.clone())
                }
                Expr::Builtin(f) => {
                    f.apply(&mut env.clone(), &args)
                },
                f => panic!("Invalid function {:?}", f)
            }
        } else {
            panic!("Invalid args {:?}", args);
        }
    });

    env.bind_builtin("cons", |env, expr| {
        let a = env.eval(expr[0].clone());
        let b = env.eval(expr[1].clone());
        // Create a new list with a as the head and b as the tail.
        if let Expr::List(b) = b {
            let mut list = vec![a];
            list.extend(b);
            Expr::List(list)
        } else if b == Expr::None {
            Expr::List(vec![a])
        } else {
            Expr::List(vec![a, b])
        }
    });
    let head = |env: &mut Env, expr: &[Expr]| {
        let a = env.eval(expr[0].clone());
        if let Expr::List(a) = a {
            a[0].clone()
        } else {
            panic!("Invalid expr {:?}", a);
        }
    };
    let tail = |env: &mut Env, expr: &[Expr]| {
        let a = env.eval(expr[0].clone());
        if let Expr::List(a) = a {
            Expr::List(a[1..].to_vec())
        } else {
            panic!("Invalid expr {:?}", a);
        }
    };

    env.bind_builtin("car", head);
    env.bind_builtin("head", head);
    env.bind_builtin("cdr", tail);
    env.bind_builtin("tail", tail);

    let format = |env: &mut Env, expr: &[Expr]| {
        let format = env.eval(expr[0].clone());
        // Collect the args
        let args = expr[1..].to_vec();
        
        let mut format = match format {
            Expr::String(s) => s,
            e => panic!("Invalid format {:?}", e)
        };
        
        // Find all of the format specifiers.
        let mut specifiers = vec![];
        for (i, c) in format.chars().enumerate() {
            if c == '{' {
                let mut j = i + 1;
                while j < format.len() {
                    if format.chars().nth(j).unwrap() == '}' {
                        break;
                    }
                    j += 1;
                }
                specifiers.push(format[i+1..j].to_owned());
            }
        }

        // Replace the named specifiers with variables in the scope.
        for name in &specifiers {
            if name.is_empty() {
                continue;
            }
            let name = Expr::Symbol(Symbol::new(name));
            
            let value = env.eval(name.clone());
            let specifier = format!("{{{name}}}");
            match value {
                Expr::String(s) => {
                    format = format.replacen(&specifier, &s, 1);
                },
                other => {
                    format = format.replacen(&specifier, &other.to_string(), 1);
                }
            }
        }

        // Replace the empty specifiers with the args.
        let mut i = 0;        
        for name in &specifiers {
            if !name.is_empty() {
                continue;
            }
            if i >= args.len() {
                return Expr::Err(Box::new(Expr::String("Not enough arguments".to_owned())));
            }
            let specifier = format!("{{}}");
            let value = env.eval(args[i].clone());
            match value {
                Expr::String(s) => {
                    format = format.replacen(&specifier, &s, 1);
                },
                other => {
                    format = format.replacen(&specifier, &other.to_string(), 1);
                }
            }
            // format = format.replacen("{}", &args[i].to_string(), 1);
            i += 1;
        }

        if i < args.len() {
            return Expr::Err(Box::new(Expr::String("Too many arguments".to_owned())));
        }
        
        Expr::String(format)
    };

    env.bind_builtin("format", format);

    env.bind_builtin("list", |env, expr| {
        let mut list = vec![];
        for e in expr {
            list.push(env.eval(e.clone()));
        }
        Expr::List(list)
    });

    env.bind_builtin("append", |env, expr| {
        let mut list = vec![];
        for e in expr {
            let e = env.eval(e.clone());
            if let Expr::List(l) = e {
                list.extend(l);
            } else {
                panic!("Invalid append {:?}", e);
            }
        }
        Expr::List(list)
    });

    env.bind_builtin("eval", |env, expr| {
        let e = env.eval(expr[0].clone());
        env.eval(e)
    });

    env.bind_builtin("quote", |_env, expr| {
        expr[0].clone()
    });

    env.bind_builtin("or", |env, expr| {
        for e in expr {
            let e = env.eval(e.clone());
            if e == Expr::Bool(true) {
                return Expr::Bool(true);
            }
        }
        Expr::Bool(false)
    });

    env.bind_builtin("and", |env, expr| {
        for e in expr {
            let e = env.eval(e.clone());
            if e == Expr::Bool(false) {
                return Expr::Bool(false);
            }
        }
        Expr::Bool(true)
    });

    env.bind_builtin("not", |env, expr| {
        let e = env.eval(expr[0].clone());
        match e {
            Expr::Bool(b) => Expr::Bool(!b),
            e => panic!("Invalid expr {:?}", e)
        }
    });

    env.bind_builtin("len", |env, expr| {
        let e = env.eval(expr[0].clone());
        match e {
            Expr::String(s) => Expr::Int(s.len() as i64),
            Expr::List(l) => Expr::Int(l.len() as i64),
            Expr::Map(m) => Expr::Int(m.len() as i64),
            Expr::Tree(t) => Expr::Int(t.len() as i64),
            e => panic!("Invalid expr {:?}", e)
        }
    });

    env.bind_builtin("let", |env, expr| {
        let mut new_env = env.clone();
        let bindings = expr[0].clone();
        let body = expr[1].clone();
        if let Expr::List(bindings) = bindings {
            for binding in bindings {
                if let Expr::List(binding) = binding {
                    let name = binding[0].clone();
                    let value = env.eval(binding[1].clone());
                    new_env.bind(name, value);
                } else {
                    panic!("Invalid binding {:?}", binding);
                }
            }
        } else {
            panic!("Invalid bindings {:?}", bindings);
        }
        new_env.eval(body)
    });

    env.bind_builtin("get", |env, expr| {
        let a = env.eval(expr[0].clone());
        let b = env.eval(expr[1].clone());

        match (a, b) {
            (Expr::String(a), Expr::Int(b)) => Expr::String(a.chars().nth(b as usize).unwrap_or('\0').to_string()),
            (Expr::List(a), Expr::Int(b)) => a.get(b as usize).cloned().unwrap_or(Expr::None),
            (Expr::Map(a), b) => a.get(&b).cloned().unwrap_or(Expr::None),
            (Expr::Tree(a), b) => a.get(&b).cloned().unwrap_or(Expr::None),
            (a, b) => panic!("Invalid expr {:?} get {:?}", a, b)
        }
    });

    env.bind_builtin("set", |env, expr| {
        let a = env.eval(expr[0].clone());
        let b = env.eval(expr[1].clone());
        let c = env.eval(expr[2].clone());

        match (a, b) {
            (Expr::String(mut a), Expr::Int(b)) => {
                if b == a.len() as i64 {
                    a.push_str(&c.to_string());
                } else {
                    a = a.chars().enumerate().map(|(i, c)| if i == b as usize { c } else { '\0' }).collect();
                }
                Expr::String(a)
            },
            (Expr::List(mut a), Expr::Int(b)) => {
                if b as usize >= a.len() {
                    a.resize(b as usize + 1, Expr::None);
                }
                a[b as usize] = c;
                Expr::List(a)
            },
            (Expr::Map(mut a), b) => {
                a.insert(b, c);
                Expr::Map(a)
            },
            (Expr::Tree(mut a), b) => {
                a.insert(b, c);
                Expr::Tree(a)
            },
            (a, b) => panic!("Invalid expr {:?} set {:?} {:?}", a, b, c)
        }
    });

    env.bind_builtin("map", |env, expr| {
        let f = env.eval(expr[0].clone());
        let a = env.eval(expr[1].clone());
        match a {
            Expr::List(a) => {
                let mut list = vec![];
                for e in a {
                    list.push(env.eval(Expr::List(vec![f.clone(), e])));
                }
                Expr::List(list)
            },
            Expr::Map(a) => {
                let mut map = std::collections::HashMap::new();
                for (k, v) in a {
                    // map.insert(k.clone(), env.eval(Expr::List(vec![f.clone(), k, v])));
                    let pair = env.eval(Expr::List(vec![f.clone(), Expr::Quote(Box::new(k)), v]));
                    if let Expr::List(pair) = pair {
                        map.insert(pair[0].clone(), pair[1].clone());
                    } else {
                        panic!("Invalid pair {:?}", pair);
                    }
                }
                Expr::Map(map)
            },
            Expr::Tree(a) => {
                let mut tree = std::collections::BTreeMap::new();
                for (k, v) in a {
                    // tree.insert(k.clone(), env.eval(Expr::List(vec![f.clone(), k, v])));
                    let pair = env.eval(Expr::List(vec![f.clone(), Expr::Quote(Box::new(k)), v]));
                    if let Expr::List(pair) = pair {
                        tree.insert(pair[0].clone(), pair[1].clone());
                    } else {
                        panic!("Invalid pair {:?}", pair);
                    }
                }
                Expr::Tree(tree)
            },
            a => panic!("Invalid expr {:?}", a)
        }
    });

    env.bind_builtin("filter", |env, expr| {
        let f = env.eval(expr[0].clone());
        let a = env.eval(expr[1].clone());

        match a {
            Expr::List(a) => {
                let mut list = vec![];
                for e in a {
                    if env.eval(Expr::List(vec![f.clone(), e.clone()])) == Expr::Bool(true) {
                        list.push(e);
                    }
                }
                Expr::List(list)
            },
            Expr::Map(a) => {
                let mut map = std::collections::HashMap::new();
                for (k, v) in a {
                    let x = env.eval(Expr::List(vec![f.clone(), Expr::Quote(Box::new(k.clone())), v.clone()]));
                    if x == Expr::Bool(true) {
                        map.insert(k, v);
                    }
                }
                Expr::Map(map)
            },
            Expr::Tree(a) => {
                let mut tree = std::collections::BTreeMap::new();
                for (k, v) in a {
                    let x = env.eval(Expr::List(vec![f.clone(), Expr::Quote(Box::new(k.clone())), v.clone()]));
                    if x == Expr::Bool(true) {
                        tree.insert(k, v);
                    }
                }
                Expr::Tree(tree)
            },
            a => panic!("Invalid expr {:?}", a)
        }
    });

    env.bind_builtin("reduce", |env, expr| {
        let f = env.eval(expr[0].clone());
        let a = env.eval(expr[1].clone());
        let b = env.eval(expr[2].clone());

        match a {
            Expr::List(a) => {
                let mut acc = b;
                for e in a {
                    acc = env.eval(Expr::List(vec![f.clone(), acc, e]));
                }
                acc
            },
            Expr::Map(a) => {
                let mut acc = b;
                for (k, v) in a {
                    acc = env.eval(Expr::List(vec![f.clone(), acc, k, v]));
                }
                acc
            },
            Expr::Tree(a) => {
                let mut acc = b;
                for (k, v) in a {
                    acc = env.eval(Expr::List(vec![f.clone(), acc, k, v]));
                }
                acc
            },
            a => panic!("Invalid expr {:?}", a)
        }
    });

    env.bind_builtin("range", |env, expr| {
        let a = env.eval(expr[0].clone());
        let b = env.eval(expr[1].clone());
        match (a, b) {
            (Expr::Int(a), Expr::Int(b)) => {
                let mut list = vec![];
                for i in a..b {
                    list.push(Expr::Int(i));
                }
                Expr::List(list)
            },
            (Expr::Int(a), Expr::Float(b)) => {
                let mut list = vec![];
                for i in a..(b as i64) {
                    list.push(Expr::Int(i));
                }
                Expr::List(list)
            },
            (Expr::Float(a), Expr::Int(b)) => {
                let mut list = vec![];
                for i in (a as i64)..b {
                    list.push(Expr::Int(i));
                }
                Expr::List(list)
            },
            (Expr::Float(a), Expr::Float(b)) => {
                let mut list = vec![];
                for i in (a as i64)..(b as i64) {
                    list.push(Expr::Int(i));
                }
                Expr::List(list)
            },
            (a, b) => panic!("Invalid expr {:?} range {:?}", a, b)
        }
    });
    
    let result = env.eval(e);
    if args.program.is_some() && result != Expr::None {
        println!("{}", result);
    }
}