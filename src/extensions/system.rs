use crate::{Env, Expr};


pub fn add_bindings(env: &mut Env) {
    env.bind_builtin("exit", exit_program);
    env.bind_builtin("os", get_system_os_name);
    env.bind_builtin("arch", get_system_architecture);
    env.bind_builtin("family", get_system_family);
    env.bind_builtin("env", get_system_env);
    env.bind_builtin("args", get_system_args);
    env.bind_builtin("cwd", get_system_cwd);
    env.bind_builtin("home", get_system_home);
    env.bind_builtin("user", get_system_user);
    env.bind_builtin("username", get_system_username);
    env.bind_builtin("host", get_system_host);
    env.bind_builtin("shell", get_system_shell);
    env.bind_builtin("version", get_system_version);
    env.bind_builtin("path", get_system_path);
    env.bind_builtin("temp", get_system_temp);
    env.bind_builtin("shell", shell_out);
    env.bind_builtin("pid", get_system_pid);
}

fn exit_program(env: &mut Env, args: Vec<Expr>) -> Expr {
    if args.is_empty() {
        std::process::exit(0);
    }

    match env.eval(args[0].clone()) {
        Expr::Int(i) => std::process::exit(i as i32),
        Expr::String(s) => {
            eprintln!("{s}");
            std::process::exit(1);
        }
        e => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn get_system_os_name(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::String(std::env::consts::OS.to_string());
}

fn get_system_architecture(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::String(std::env::consts::ARCH.to_string());
}

fn get_system_family(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::String(std::env::consts::FAMILY.to_string());
}

fn get_system_env(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::Map(std::env::vars().map(|(k, v)| (Expr::String(k), Expr::String(v))).collect());
}

fn get_system_args(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    // Get the arguments supplied to the program
    let args: Vec<Expr> = std::env::args().map(Expr::String).collect();
    return Expr::List(args);
}

fn get_system_cwd(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::String(std::env::current_dir().unwrap().to_str().unwrap().to_string());
}

fn get_system_home(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::String(std::env::var("HOME").unwrap_or_else(|_| "".to_string()));
}

fn get_system_user(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::String(std::env::var("USER").unwrap_or_else(|_| "".to_string()));
}

fn get_system_username(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::String(std::env::var("USERNAME").unwrap_or_else(|_| "".to_string()));
}

fn get_system_host(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::String(std::env::var("HOSTNAME").unwrap_or_else(|_| "".to_string()));
}

fn get_system_shell(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::String(std::env::var("SHELL").unwrap_or_else(|_| "".to_string()));
}

fn get_system_version(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::String(std::env::var("VERSION").unwrap_or_else(|_| "".to_string()));
}

fn get_system_path(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::String(std::env::var("PATH").unwrap_or_else(|_| "".to_string()));
}

fn get_system_temp(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::String(std::env::temp_dir().to_str().unwrap().to_string());
}

// Run a shell command and return the output
fn shell_out(env: &mut Env, args: Vec<Expr>) -> Expr {
    let command = env.eval(args[0].clone());
    let command = match command {
        Expr::String(s) => s,
        e => return Expr::error(format!("Invalid command {:?}", e)),
    };

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // If non-zero exit code, return an error
    if !output.status.success() {
        return Expr::error(format!("{stderr}"));
    }

    return Expr::String(stdout);
}

fn get_system_pid(_env: &mut Env, _args: Vec<Expr>) -> Expr {
    return Expr::Int(std::process::id() as i64);
}