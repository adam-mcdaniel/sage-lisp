# sage-lisp

This crate implements a standalone Lisp implementation, intended for use in the [Sage](https://github.com/adam-mcdaniel/sage) preprocessor.

```lisp
(do
    (defun fact (n) 
        (if (<= n 0) 1 
            (* n (fact (- n 1)))))
    (defun print-fact (n) (println n "! = " (fact n)))

    (defun stirlings (n)
        (if (<= n 0) 1
            (* (sqrt (* 2 3.14159 n))
               (pow (/ n 2.718281) n))))

    (print-fact 5)
    
    (println "Stirling's approx. for 5! = " (stirlings 5)))
```

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
$ ./sagel -c "(do (println hi!) (+ 1 2 3))
hi!
6
```