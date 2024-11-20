
use crate::{Env, Expr, Symbol, extensions::functional::anonymous_function};
use std::{collections::HashMap, sync::{Arc, RwLock}};

pub fn add_bindings(env: &mut Env) {
    env.bind_builtin("struct", struct_definition);
    env.bind_builtin("new", new_obj);
    env.bind_builtin("get", get_attr);
    env.bind_builtin("<-", write_obj);
    env.bind_builtin("$", read_obj);
}

fn member_to_function(defun: Vec<Expr>) -> (Symbol, Vec<Expr>, Expr) {
    let name = if let Expr::Symbol(name) = defun.get(1).unwrap_or(&Expr::None) {
        name.clone()
    } else {
        Symbol::from("init")
    };
    let mut params = vec![];
    
    // Get the params from the first list
    if let Expr::List(params_expr) = defun.get(if name == &"init" {1} else {2}).unwrap_or(&Expr::None) {
        // println!("Params: {:?}", params_expr);
        for param in params_expr {
            if let Expr::Symbol(_name) = param {
                params.push(param.clone());
            } else {
                return (name, vec![], Expr::error(format!("Invalid param {:?}", param)));
            }
        }
    }

    // Get the body from the second list
    let body_expr = defun.get(if name == &"init" {2} else {3}).unwrap_or(&Expr::None);
    // println!("Body: {:?}", body_expr);

    (name, params.clone(), body_expr.clone())
}

fn function_to_lambda(member: (Symbol, Vec<Expr>, Expr)) -> Expr {
    let (_name, params, body) = member;
    Expr::Function(None, params, Box::new(body))
}

fn add_members_to_obj(mut obj: Expr, members: Vec<(Symbol, Expr)>) -> Expr {
    for member in members {
        let (name, value) = member;
        // obj = set_attr(env, vec![obj, Expr::Symbol(name), value]);
        obj = Expr::apply(
            &Expr::builtin("set", set_attr),
            &vec![
                obj,
                Expr::Symbol(name),
                value
            ]
        );
    }
    obj
}

fn struct_definition(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    // Define a new struct
    let name = exprs.get(0).unwrap_or(&Expr::None);

    // A struct is a function that returns a new object.
    // The object is populated with a series of `defun` calls.

    // Get the list of `defun` calls
    let mut members = vec![];
    let mut constructor = None;
    for expr in exprs.iter().skip(1) {
        if let Expr::List(exprs) = expr {
            if let Some(Expr::Symbol(name)) = exprs.get(0) {
                if name == &"defun" {
                    members.push(exprs.clone());
                } else if name == &"new" {
                    constructor = Some(exprs.clone());
                } else {
                    return Expr::error(format!("Invalid struct member {:?}", exprs));
                }
            }
        }
    }

    // Create a new function that returns a new object, and calls the init 
    // function if it exists.

    let mut object_members = vec![];
    for member in members {
        let (name, params, body) = member_to_function(member);
        // object.insert(params[0].clone(), Expr::Function(None, params, Box::new(body)));
        object_members.push((
            name,
            params,
            body
        ));
    }

    let mut constructor_params = vec![];
    let mut constructor_body = Expr::None;

    if let Some(constructor) = constructor {
        let (name, params, body) = member_to_function(constructor);
        constructor_params = params;
        constructor_body = body;
    }

    let object = add_members_to_obj(
        Expr::apply(
            &Expr::builtin("new", new_obj),
            &vec![
                Expr::Map(Default::default())
            ]
        ),
        object_members.into_iter().map(|(name, params, body)| {
            (name.clone(), function_to_lambda((name, params, body)))
        }).collect()
    );

    // Call init
    let constructor = Expr::Function(None, constructor_params[1..].to_vec(), {
        // Call the constructor
        Expr::apply(
            &anonymous_function(env, vec![
                Expr::List(vec![constructor_params[0].clone()]),
                Expr::many(vec![
                    constructor_body,
                    constructor_params[0].clone()
                ]).into()
            ]),
            &vec![
                object.clone(),
            ]
        ).into()
    });

    // constructor

    // Bind the struct to the environment
    env.bind(name.clone(), constructor);

    Expr::None
}

fn new_obj(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    let value = env.eval(exprs.get(0).cloned().unwrap_or_default());
    Expr::Object(Arc::new(RwLock::new(value)))
}

pub fn read_obj(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    // Get the value of an object
    let object = env.eval(exprs[0].clone());
    if let Expr::Object(object) = object {
        object.read().unwrap().clone()
    } else {
        Expr::error(format!("Invalid object {object} get"))
    }
}

pub fn write_obj(env: &mut Env, exprs: Vec<Expr>) -> Expr {
    // Set the value of an object
    let object = env.eval(exprs[0].clone());
    let value = env.eval(exprs[1].clone());
    if let Expr::Object(object) = object {
        *object.write().unwrap() = value.clone();
        value
    } else {
        Expr::error(format!("Invalid object {object} set {value}"))
    }
}

pub fn get_attr(env: &mut Env, expr: Vec<Expr>) -> Expr {
    let a = env.eval(expr[0].clone());
    let b = expr[1].clone();

    match (a, b) {
        (Expr::Object(obj), member) => {
            let val = obj.read().unwrap();
            match get_attr(env, vec![val.clone(), member.clone()]) {
                Expr::None => {
                    // Create a new object with the member as the value
                    let new_obj = Expr::Object(Arc::new(RwLock::new(Expr::None)));
                    let set_obj = set_attr(env, vec![val.clone(), member.clone(), new_obj.clone()]);
                    drop(val);
                    // Add the member to the object
                    // Write the new object to the object
                    *obj.write().unwrap() = set_obj;
                    new_obj
                }
                other => {
                    match other.strip_object() {
                        Expr::Function(_, params, _) => {

                            // Apply a method with a self parameter
                            let obj = env.eval(expr[0].clone());

                            // Get the param count
                            let method = other;
                            let param_count = params.len();
                            // Return a function that takes the rest of the arguments,
                            // and applies the method with the object as the first argument.

                            let mut args = vec![obj.clone()];
                            let mut params = vec![];
                            for i in 1..param_count {
                                let name = Symbol::from(format!("arg{}", i));
                                args.push(Expr::Symbol(name.clone()));
                                params.push(Expr::Symbol(name));
                            }

                            let body = method.apply(&args);

                            let f = Expr::Function(None, params, Box::new(body));
                            f
                        }
                        _ => other
                    }
                }
            }
        }
        (Expr::Map(a), Expr::Symbol(b)) => {
            a.get(&Expr::Symbol(b.clone())).cloned().unwrap_or_else(|| {
                a.get(&Expr::String(b.name().to_owned()))
                    .cloned()
                    .unwrap_or(Expr::None)
            })
        }
        (Expr::Map(a), b) => a.get(&b).cloned().unwrap_or(Expr::None),
        (Expr::Tree(a), Expr::Symbol(b)) => {
            a.get(&Expr::Symbol(b.clone())).cloned().unwrap_or_else(|| {
                a.get(&Expr::String(b.name().to_owned()))
                    .cloned()
                    .unwrap_or(Expr::None)
            })
        }
        (Expr::Tree(a), b) => a.get(&b).cloned().unwrap_or(Expr::None),

        (a, b) => return Expr::error(format!("Invalid expr get {} {}", a, b)),
    }
}



fn set_attr(env: &mut Env, expr: Vec<Expr>) -> Expr {
    let a = env.eval(expr[0].clone());
    let b = env.eval(expr[1].clone());
    let c = env.eval(expr[2].clone());

    let result = match (a, b) {
        (Expr::Object(obj), member) => {
            let val = obj.read().unwrap();
            let new_val = set_attr(env, vec![val.clone(), member.clone(), c.clone()]);
            drop(val);
            *obj.write().unwrap() = new_val.clone();
            Expr::Object(obj)
        }
        (Expr::Map(mut a), b) => {
            a.insert(b, c);
            Expr::Map(a)
        }
        (Expr::Tree(mut a), b) => {
            a.insert(b, c);
            Expr::Tree(a)
        }
        (a, b) => return Expr::error(format!("Invalid expr set {} {} {}", a, b, c)),
    };
    result
}