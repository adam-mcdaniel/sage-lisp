{
    (define x (new #[]))

    x.method <- (lambda (self test) (println "Hello, World! " self.a " " test))
    
    (x.method ())
    x.a <- 100
    (x.method ())
}