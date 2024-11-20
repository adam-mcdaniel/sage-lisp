use std::{io::Write, collections::{HashMap, BTreeMap}};

use lazy_static::lazy_static;
use prettytable::{cell, row, format::{TableFormat, LinePosition, LineSeparator}};

use crate::{Env, Expr, Symbol};

use super::env_to_map;

pub fn add_bindings(env: &mut Env) {
    env.bind_builtin("format", format);
    env.bind_builtin("show", show);
}


fn format(env: &mut Env, expr: Vec<Expr>) -> Expr {
    let format = env.eval(expr[0].clone());
    // Collect the args
    let args = expr[1..].to_vec();

    let mut format = match format {
        Expr::String(s) => s,
        e => return Expr::error(format!("Invalid format {e}")),
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
            specifiers.push(format[i + 1..j].to_owned());
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
            }
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
            return Expr::error("Too few arguments");
        }
        let specifier = format!("{{}}");
        let value = env.eval(args[i].clone());
        match value {
            Expr::String(s) => {
                format = format.replacen(&specifier, &s, 1);
            }
            other => {
                format = format.replacen(&specifier, &other.to_string(), 1);
            }
        }
        // format = format.replacen("{}", &args[i].to_string(), 1);
        i += 1;
    }

    if i < args.len() {
        return Expr::error("Too many arguments");
    }

    Expr::String(format)
}

lazy_static! {
    static ref TABLE_FORMAT: TableFormat = {
        let mut fmt = TableFormat::new();
        fmt.borders('│');
        fmt.column_separator('│');
        fmt.separator(LinePosition::Top, LineSeparator::new('═', '╤', '╒', '╕'));
        fmt.separator(LinePosition::Title, LineSeparator::new('═', '╪', '╞', '╡'));
        fmt.separator(LinePosition::Intern, LineSeparator::new('─', '┼', '├', '┤'));
        fmt.separator(LinePosition::Bottom, LineSeparator::new('─', '┴', '└', '┘'));

        fmt
    };

    static ref LIST_FORMAT: TableFormat = {
        let mut fmt = TableFormat::new();
        fmt.borders('┃');
        fmt.column_separator('┃');
        fmt.separator(LinePosition::Top, LineSeparator::new('━', '┳', '┏', '┓'));
        fmt.separator(LinePosition::Title, LineSeparator::new('━', '╋', '┣', '┫'));
        fmt.separator(LinePosition::Intern, LineSeparator::new('━', '╋', '┣', '┫'));
        fmt.separator(LinePosition::Bottom, LineSeparator::new('━', '┻', '┗', '┛'));
        
        fmt
    };

    static ref FUNCTION_FORMAT: TableFormat = {
        let mut fmt = TableFormat::new();
        fmt.borders('│');
        fmt.column_separator('│');
        fmt.separator(LinePosition::Top, LineSeparator::new('═', '╤', '╒', '╕'));
        fmt.separator(LinePosition::Title, LineSeparator::new('═', '╪', '╞', '╡'));
        fmt.separator(LinePosition::Intern, LineSeparator::new('─', '┼', '├', '┤'));
        fmt.separator(LinePosition::Bottom, LineSeparator::new('─', '┴', '└', '┘'));

        fmt
    };
}

fn test(env: &mut Env, _args: Vec<Expr>) -> Expr {
    // map_to_table(env.get_bindings(), 80).printstd();

    let table = map_to_table(env.get_bindings(), 80);
    // Set the column separator to a pipe
    table.printstd();

    Expr::None
}

fn show(env: &mut Env, expr: Vec<Expr>) -> Expr {
    if expr.len() != 1 {
        return Expr::error("show: expected 1 argument");
    }

    let expr = env.eval(expr[0].clone());
    let mut result = vec![];
    match expr {
        Expr::Map(map) => {
            map_to_table(map, 80).print(&mut result).expect("Error printing map");
        }
        Expr::Tree(tree) => {
            tree_to_table(tree, 80).print(&mut result).expect("Error printing tree");
        }
        Expr::List(list) => {
            list_to_table(list, 80).print(&mut result).expect("Error printing list");
        }
        expr => {
            result.write_all(expr_to_cell(expr, 80).to_string().as_bytes()).unwrap();
        }
    }
    Expr::String(String::from_utf8(result).unwrap())
}

fn fit_string(x: impl ToString, max_width: usize) -> String {
    let s = x.to_string();
    let mut result = String::new();
    let mut width = 0;
    for c in s.chars() {
        match c {
            '\n' => {
                result.push(c);
                width = 0;
            }
            _ => {
                if width >= max_width {
                    result.push('\n');
                    width = 0;
                }
                result.push(c);
                width += 1;
            }
        }
    }
    result
}

fn expr_to_cell(expr: Expr, max_width: usize) -> prettytable::Cell {
    match &expr {
        Expr::Object(o) => {
            let mut table = prettytable::Table::new();
            table.set_titles(row![bFWBC->"Object"]);

            table.add_row(row![expr_to_cell(o.read().unwrap().clone(), max_width - 4)]);
            table.set_format(*TABLE_FORMAT);

            cell!(table)
        },
        Expr::String(s) => cell!(fit_string(format!("{s:?}"), max_width)),
        Expr::Int(i) => cell!(i),
        Expr::Bool(b) => cell!(b),
        Expr::List(l) => cell!(list_to_table(l.clone(), max_width - 4)),
        Expr::Map(m) => cell!(map_to_table(m.clone(), max_width - 4)),
        Expr::Tree(m) => cell!(tree_to_table(m.clone(), max_width - 4)),
        Expr::Function(env, params, body) => cell!(function_to_table((env, params.clone(), *body.clone()), max_width - 4)),
        _ => cell!(fit_string(expr, max_width)),
    }
}

fn function_to_table(func: (&Option<Box<Env>>, Vec<Expr>, Expr), max_width: usize) -> prettytable::Table {
    let mut table = prettytable::Table::new();
    table.set_titles(row![bFWBR->"Function"]);

    let (env, args, body) = func;
    let env = env.as_ref().map(|env| env_to_map(env));
    // table.add_row(row![bFWBM->"Environment", expr_to_cell(env.unwrap_or(Expr::None), max_width-4)]);
    table.add_row(row![bFWBM->"Arguments", expr_to_cell(Expr::List(args), max_width-4)]);
    table.add_row(row![bFWBM->"Body", expr_to_cell(body, max_width-4)]);
    table.set_format(*FUNCTION_FORMAT);
    table
}

fn list_to_table(list: Vec<Expr>, max_width: usize) -> prettytable::Table {
    let mut table = prettytable::Table::new();
    table.set_titles(row![bFWBM->"List"]);

    for e in list {
        table.add_row(row![expr_to_cell(e, max_width-4)]);
    }

    table.set_format(*LIST_FORMAT);
    table
}

fn tree_to_table(map: BTreeMap<Expr, Expr>, max_width: usize) -> prettytable::Table {
    let mut table = prettytable::Table::new();
    table.set_titles(row![bBG->"TreeMap Key", bBG->"TreeMap Value"]);

    for (key, value) in map {
        table.add_row(row![expr_to_cell(key, max_width-4), expr_to_cell(value, max_width-4)]);
    }

    table.set_format(*TABLE_FORMAT);
    table
}

fn map_to_table(map: HashMap<Expr, Expr>, max_width: usize) -> prettytable::Table {
    let mut table = prettytable::Table::new();
    table.set_titles(row![bFWBM->"HashMap Key", bFWBM->"HashMap Value"]);

    for (key, value) in map {
        table.add_row(row![expr_to_cell(key, max_width-4), expr_to_cell(value, max_width-4)]);
    }

    table.set_format(*TABLE_FORMAT);
    table
}