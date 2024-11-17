# Sage-Lisp

![Logo](assets/logo.png)

- **Author**: Adam McDaniel
- **GitHub**: [adam-mcdaniel/sage-lisp](https://github.com/adam-mcdaniel/sage-lisp)
- **License**: MIT

This crate implements a standalone Lisp implementation, intended for use in the [Sage](https://github.com/adam-mcdaniel/sage) preprocessor.

## Table of Contents

- [About](#about)
- [Features](#features)
- [Usage](#usage)
- [Example Lisp Syntax](#example-lisp-syntax)
- [Example Embedding](#example-embedding)
- [About the Author](#about-the-author)
- [Documentation](#documentation)
- [License](#license)

## About

Sage-Lisp is a simple core lisp interpreter written in Rust,
that can be expanded to include new functionality with built-in functions.

It's designed such that you can provide your own standard library of functions,
implemented in Rust or in Lisp, and evaluate lisp expressions with your own
standard conventions.

It works well as an embedded language for data transformation, where you can
import data into the interpreter (using Serde), perform transformations on it, and then
export the data back out into a format that you can use in your application.

## Features

- **Simple Lisp Interpreter**: A simple core lisp interpreter that can evaluate lisp expressions.
- **Built-in Functions**: Extend the language with new functionality using built-in functions.
- **Symbol Interning**: Symbols are interned to ensure that they are unique and fast to compare.
- **Tail Recursion**: Uses tail recursion to evaluate deeply nested function calls without stack overflow.
- **Lazy Evaluation**: Supports lazy evaluation of expressions, for defining special forms.
- **Serde Integration**: Serialize and deserialize lisp expressions using Serde.
- **Error Handling**: Provides helpful error messages for parsing and evaluation errors.
- **Expanded Syntax**: Introduces infix operators, code block syntax, syntax for hashmaps and ordered maps, and more.
- **Customizable**: Define your own standard library of functions and variables, and override the default behavior.


## Usage

To run Sage lisp on a program, build it like so:
```bash
$ cargo build --features build-binary --release
```

Then, use it to run a file as a program.

```bash
$ cp ./target/release/sagel .
$ ./sagel example.lisp
```

Or provide a program as a command line argument.

```bash
$ ./sagel -c "(do (println hi!) (+ 1 2 3))"
hi!
6
```


## Example Lisp Syntax

The example below shows some of the syntax that is supported by the interpreter,
along with some of the built-in functions that are provided when the crate is
used as an executable.

```lisp
;; Define a function that calculates the factorial of a number
(defun fact (n) 
    (if n <= 0
        1
        n * (fact n - 1)))
;; Stirling's approximation for the factorial
(defun stirlings (n)
    (if n <= 0 1
        (* (sqrt 2 * 3.14159265358979323846 * n)
           ((n / 2.71828182845904523536) ^ n))))
;; Perform a quicksort on a list of numbers
(defun quicksort (lst)
    (if (<= (len lst) 1) lst {
        (define pivot (get lst (/ (len lst) 2)))
        (define less (filter (\(x) (< x pivot)) lst))
        (define equal (filter (\(x) (= x pivot)) lst))
        (define greater (filter (\(x) (> x pivot)) lst))
        (+ (quicksort less) equal (quicksort greater))}))
```

## Example Embedding

Below is an example of how you can embed the interpreter into your application,
with your own standard library of functions and variables.

```rust
// Import the necessary types and traits for the interpreter.
use sage_lisp::{Expr, Env};

// Create a new environment
fn make_core_env() -> Env {
    // Create a new environment
    let mut env = Env::new();

    // Create a function that adds two numbers
    env.bind_builtin("+", |env, exprs| {
        let mut sum = Expr::default();
        for e in exprs {
            // Evaluate the supplied argument in the environment
            let e = env.eval(e.clone());
            // Match the expression to the sum
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
                // Return an error if the expression is invalid
                (a, b) => return Expr::error(format!("Invalid expr {} + {}", a, b)),
            }
        }
        sum
    });
    // Create a function that prints a string
    env.bind_builtin("println", |env, exprs| {
        for e in exprs {
            // Evaluate the supplied argument in the environment
            let e = env.eval(e.clone());
            // Print the expression
            match e {
                Expr::String(s) => print!("{}", s),
                Expr::Symbol(s) => print!("{}", s.name()),
                _ => print!("{}", e),
            }
        }
        println!();
        Expr::None
    });
    env
}

fn main() {
    // Create a new environment with our custom standard library
    let mut env = make_core_env();

    // Evaluate a lisp expression in the environment
    env.eval_str("{ (println 1 + 2) (println (+ 2 2)) }").unwrap();
}
```

## About the Author

[I'm a computer science PhD student](https://adam-mcdaniel.net) at the [University of Tennessee, Knoxville🍊](https://www.youtube.com/watch?v=-8MlEo02u54). Rust is my favorite language, and [I've](https://github.com/adam-mcdaniel/sage) [written](https://github.com/adam-mcdaniel/oakc) [many](https://github.com/adam-mcdaniel/harbor) [other](https://github.com/adam-mcdaniel/tsar) [programming](https://github.com/adam-mcdaniel/free) [languages](https://github.com/adam-mcdaniel/xasm).

I'm always looking for new projects to work on, so if you have an idea for a project or a collaboration, feel free to reach out to me or join me on the [Sage Discord server](https://discord.gg/rSGkM4bcdP)!

## Documentation

For more information on how to use the Sage Lisp interpreter, see the [documentation](https://adam-mcdaniel.github.io/sage-lisp/).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

