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

    let program = Program::parse();
    // Either open the file or use the program string.
    let mut program = match program.program {
        Some(program) => program,
        None => {
            match program.program_name {
                Some(program_name) => {
                    std::fs::read_to_string(program_name).unwrap()
                },
                None => {
                    panic!("No program provided");
                }
            }
        }
    };

    let (_, e) = Expr::parse(&mut program).unwrap();

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
    
    env.bind_builtin("=", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());
        
        Expr::Bool(a == b)
    });

    env.bind_builtin("<=", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());
        
        Expr::Bool(a <= b)
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
            // print!("{}", e);
            match e {
                Expr::Int(i) => print!("{}", i),
                Expr::Float(f) => print!("{}", f),
                Expr::String(s) => print!("{}", s),
                Expr::Bool(b) => print!("{}", b),
                Expr::List(l) => print!("{:?}", l),
                _ => print!("{:?}", e)
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

    env.bind_builtin("pow", |env, expr| {
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

    // println!("Expr: {}", e);
    // println!("Expr: {:?}", e);
    // println!("Result: {}", env.eval(e));
    env.eval(e);
}