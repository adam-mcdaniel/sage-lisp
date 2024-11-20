
{
    (mod.use math)
    (mod.use io)
    (mod.use fn)
    (mod.use env)
    (mod.use proc)
    (mod.use list)
    (mod.use fmt)

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
        (if (<= (length lst) 1) lst {
            (define pivot (get lst (/ (length lst) 2)))
            (define less (filter (\(x) (< x pivot)) lst))
            (define equal (filter (\(x) (= x pivot)) lst))
            (define greater (filter (\(x) (> x pivot)) lst))
            (+ (quicksort less) equal (quicksort greater))}))
            
    (define test-list (list 5 3 7 2 8 1 9 4 6))
    (println "Unsorted list: " test-list)
    (println "Sorted list: " (quicksort test-list))
}