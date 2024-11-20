use std::collections::HashMap;

use crate::{Env, Expr};

pub fn add_bindings(env: &mut Env) {
    env.bind_builtin("write", write_to_file);
    env.bind_builtin("read", read_from_file);
    env.bind_builtin("remove", remove_file);
    env.bind_builtin("create-dir", create_dir);
    env.bind_builtin("remove-dir", remove_dir);
    env.bind_builtin("rename", rename_file);
    env.bind_builtin("copy", copy_file);
    env.bind_builtin("metadata", metadata);
    env.bind_builtin("list-dir", list_dir);
    env.bind_builtin("canonicalize", canonicalize_path);
}

fn write_to_file(env: &mut Env, args: Vec<Expr>) -> Expr {
    let path = env.eval(args[0].clone());
    let data = env.eval(args[1].clone());
    let path = match path {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid path {:?}", path)),
    };

    let data = match data {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid data {:?}", data)),
    };

    match std::fs::write(path, data) {
        Ok(_) => Expr::None,
        Err(e) => Expr::error(format!("Error writing to file: {:?}", e)),
    }
}

fn read_from_file(env: &mut Env, args: Vec<Expr>) -> Expr {
    let path = env.eval(args[0].clone());
    let path = match path {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid path {:?}", path)),
    };

    match std::fs::read_to_string(path) {
        Ok(data) => Expr::String(data),
        Err(e) => Expr::error(format!("Error reading from file: {:?}", e)),
    }
}

fn remove_file(env: &mut Env, args: Vec<Expr>) -> Expr {
    let path = env.eval(args[0].clone());
    let path = match path {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid path {:?}", path)),
    };

    match std::fs::remove_file(path) {
        Ok(_) => Expr::None,
        Err(e) => Expr::error(format!("Error removing file: {:?}", e)),
    }
}

fn create_dir(env: &mut Env, args: Vec<Expr>) -> Expr {
    let path = env.eval(args[0].clone());
    let path = match path {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid path {:?}", path)),
    };

    match std::fs::create_dir(path) {
        Ok(_) => Expr::None,
        Err(e) => Expr::error(format!("Error creating directory: {:?}", e)),
    }
}

fn remove_dir(env: &mut Env, args: Vec<Expr>) -> Expr {
    let path = env.eval(args[0].clone());
    let path = match path {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid path {:?}", path)),
    };

    match std::fs::remove_dir(path) {
        Ok(_) => Expr::None,
        Err(e) => Expr::error(format!("Error removing directory: {:?}", e)),
    }
}

fn rename_file(env: &mut Env, args: Vec<Expr>) -> Expr {
    let old_path = env.eval(args[0].clone());
    let new_path = env.eval(args[1].clone());
    let old_path = match old_path {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid old path {:?}", old_path)),
    };

    let new_path = match new_path {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid new path {:?}", new_path)),
    };

    match std::fs::rename(old_path, new_path) {
        Ok(_) => Expr::None,
        Err(e) => Expr::error(format!("Error renaming file: {:?}", e)),
    }
}

fn copy_file(env: &mut Env, args: Vec<Expr>) -> Expr {
    let old_path = env.eval(args[0].clone());
    let new_path = env.eval(args[1].clone());
    let old_path = match old_path {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid old path {:?}", old_path)),
    };

    let new_path = match new_path {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid new path {:?}", new_path)),
    };

    match std::fs::copy(old_path, new_path) {
        Ok(_) => Expr::None,
        Err(e) => Expr::error(format!("Error copying file: {:?}", e)),
    }
}

fn metadata(env: &mut Env, args: Vec<Expr>) -> Expr {
    let path = env.eval(args[0].clone());
    let path = match path {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid path {:?}", path)),
    };

    match std::fs::metadata(path) {
        Ok(metadata) => {
            let mut map = HashMap::new();
            map.insert(Expr::Symbol("is_dir".into()), Expr::Bool(metadata.is_dir()));
            map.insert(Expr::Symbol("is_file".into()), Expr::Bool(metadata.is_file()));
            map.insert(Expr::Symbol("len".into()), Expr::Int(metadata.len() as i64));
            map.insert(Expr::Symbol("created".into()), Expr::Int(metadata.created().unwrap().elapsed().unwrap().as_secs() as i64));
            map.insert(Expr::Symbol("modified".into()), Expr::Int(metadata.modified().unwrap().elapsed().unwrap().as_secs() as i64));
            Expr::Map(map)
        }
        Err(e) => Expr::error(format!("Error getting metadata: {:?}", e)),
    }
}

fn list_dir(env: &mut Env, args: Vec<Expr>) -> Expr {
    let path = env.eval(args[0].clone());
    let path = match path {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid path {:?}", path)),
    };

    match std::fs::read_dir(path) {
        Ok(entries) => {
            let mut list = Vec::new();
            for entry in entries {
                let entry = entry.unwrap();
                let mut map = HashMap::new();
                map.insert(Expr::Symbol("path".into()), Expr::String(entry.path().to_str().unwrap().to_string()));
                map.insert(Expr::Symbol("file_name".into()), Expr::String(entry.file_name().to_str().unwrap().to_string()));
                list.push(Expr::Map(map));
            }
            Expr::List(list)
        }
        Err(e) => Expr::error(format!("Error listing directory: {:?}", e)),
    }
}

fn canonicalize_path(env: &mut Env, args: Vec<Expr>) -> Expr {
    let path = env.eval(args[0].clone());
    let path = match path {
        Expr::String(s) => s,
        _ => return Expr::error(format!("Invalid path {:?}", path)),
    };

    match std::fs::canonicalize(path) {
        Ok(path) => Expr::String(path.to_str().unwrap().to_string()),
        Err(e) => Expr::error(format!("Error canonicalizing path: {:?}", e)),
    }
}

