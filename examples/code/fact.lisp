{
    (defun fact (n) 
        (if n <= 0
            1
            n * (fact n - 1)))
    (defun stirlings (n)
        (if n <= 0 1
            (sqrt 2 * 3.14159265358979323846 * n)
               * (pow n / 2.71828182845904523536 n)))

    (define cbrt   (lambda (x) (^ x (/ 1 3.0))))
    (define qurt   (lambda (x) (^ x (/ 1 4.0))))
    (define square (lambda (x) (* x x)))
    (define cube   (lambda (x) (* x x x)))

    (define compose (lambda (f g) (lambda (x) (f (g x)))))

    (define inc (lambda (x) (+ x 1)))
    (define dec (lambda (x) (- x 1)))

    (define test (compose square inc))
    (println (test 5))

    (define test '(+ 1 2 3 4 5))

    (println (format "{testing} {} {}!" (fact 10) (eval test)))

    (defun is-even (n) (= (% n 2) 0))
    (defun is-odd (n) (= (% n 2) 1))

    (println (map (\ (k v) (list k (square v))) [x 5 y 10]))
    (println (filter (\ (k v) (is-even v)) [x 5 y 10]))

    (define l (range 1 10))
    (defun fastfact (n) (apply (eval '*) (range 1 n)))

    (define n 4)
    (println (format "Factorial of {n}: {}" (fastfact n)))

    (println "even: " (filter is-even l) " odd: " (filter is-odd l))

    (println "Stirling's approximation for 5! = " (stirlings 5))
}