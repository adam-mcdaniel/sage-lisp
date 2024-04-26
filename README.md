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
    
    (println "Stirling's approx. for 5! = " (stirlings 5))
    
    (defun quicksort (lst)
        (if (<= (len lst) 1) lst {
            (def pivot (get lst (/ (len lst) 2)))
            (def less (filter (\(x) (< x pivot)) lst))
            (def equal (filter (\(x) (= x pivot)) lst))
            (def greater (filter (\(x) (> x pivot)) lst))
            (+ (quicksort less) equal (quicksort greater))}))
            
    (def test-list (list 5 3 7 2 8 1 9 4 6))
    (println "Unsorted list: " test-list)
    (println "Sorted list: " (quicksort test-list)))
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
$ ./sagel -c "(do (println hi!) (+ 1 2 3))"
hi!
6
```