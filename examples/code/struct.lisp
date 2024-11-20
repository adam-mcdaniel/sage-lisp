{
    (oop.struct Point
        (new (self a b) {
            self.x <- a
            self.y <- b
        })
        
        (defun move (self dx dy) {
            self.x <- (math.sum $self.x dx)
            self.y <- (math.sum $self.y dy)
        })

        (defun print (self) {
            (io.println (self.show))
        })

        (defun debug (self) {
            (io.println (self.repr))
        })
        
        (defun show (self) {
            (fmt.format "Point({}, {})" $self.x $self.y)
        })
        
        (defun repr (self) {
            (fmt.show self)
        }))

    (env.define p (Point 10 20))

    (p.debug)
    (p.print)
    (p.move 5 5)
    (p.print)
}