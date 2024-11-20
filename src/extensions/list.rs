use crate::{Env, Expr};

pub fn add_bindings(env: &mut Env) {
    env.bind_builtin("list", list);
    env.bind_builtin("get", list_get);
    env.bind_builtin("set", list_set);
    env.bind_builtin("length", list_length);
    env.bind_builtin("push", list_push);
    env.bind_builtin("pop", list_pop);
    env.bind_builtin("insert", list_insert);
    env.bind_builtin("remove", list_remove);
    env.bind_builtin("reverse", list_reverse);
    env.bind_builtin("extend", list_extend);
    env.bind_builtin("split", list_split);
    env.bind_builtin("sort", list_sort);
    env.bind_builtin("sort-desc", list_sort_desc);
    env.bind_builtin("head", list_head);
    env.bind_builtin("tail", list_tail);
    env.bind_builtin("zip", list_zip);
}

/// Create a list from the given elements.
/// 
/// # Example
/// 
/// ```lisp
/// (list 1 2 3)
/// ```
/// 
/// ## Output
/// 
/// ```
/// (list 1 2 3)
/// ```
fn list(env: &mut Env, args: Vec<Expr>) -> Expr {
    let mut list = vec![];
    for e in args {
        list.push(env.eval(e.clone()));
    }
    Expr::List(list)
}

/// Get an element from a list at the specified index.
/// 
/// # Example
/// 
/// ```lisp
/// (get (list 1 2 3) 1)
/// ```
/// 
/// ## Output
/// 
/// ```
/// 2
/// ```
fn list_get(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Get the index
    let index = match env.eval(args.get(1).cloned().unwrap_or(Expr::None)) {
        Expr::Int(index) => index as usize,
        _ => return Expr::None,
    };

    // Check if the container is a list
    let list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Get the element at the specified index
    list.get(index).cloned().unwrap_or(Expr::None)
}

/// Set an element in a list at the specified index.
/// 
/// # Example
/// 
/// ```lisp
/// (set (list 1 2 3) 1 4)
/// ```
/// 
/// ## Output
/// 
/// ```
/// (list 1 4 3)
/// ```
fn list_set(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Get the index
    let index = match env.eval(args.get(1).cloned().unwrap_or(Expr::None)) {
        Expr::Int(index) => index as usize,
        _ => return Expr::None,
    };

    // Get the element to set
    let element = env.eval(args.get(2).cloned().unwrap_or(Expr::None));

    // Check if the container is a list
    let mut list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Set the element at the specified index
    list.get_mut(index).map(|e| *e = element);

    Expr::List(list)
}

/// Get the length of a list.
/// 
/// # Example
/// 
/// ```lisp
/// (length (list 1 2 3))
/// ```
/// 
/// ## Output
/// 
/// ```
/// 3
/// ```
fn list_length(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Check if the container is a list
    let list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Get the length of the list
    Expr::Int(list.len() as i64)
}

/// Pushes an element to the end of a list.
/// 
/// # Example
/// 
/// ```lisp
/// (push (list 1 2 3) 4)
/// ```
/// 
/// ## Output
/// 
/// ```lisp
/// (list 1 2 3 4)
/// ```
fn list_push(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Get the element to push
    let element = env.eval(args.get(1).cloned().unwrap_or(Expr::None));

    // Check if the container is a list
    let mut list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Push the element to the list
    list.push(element);

    Expr::List(list)
}

/// Removes the last element from a list.
/// 
/// # Example
/// 
/// ```lisp
/// (pop (list 1 2 3))
/// ```
/// 
/// ## Output
/// 
/// ```lisp
/// (list 1 2)
/// ```
fn list_pop(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Check if the container is a list
    let mut list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Pop the last element from the list
    list.pop();
    Expr::List(list)
}


/// Inserts an element at the specified index in a list.
/// 
/// # Example
/// 
/// ```lisp
/// (insert (list 1 2 3) 1 4)
/// ```
/// 
/// ## Output
/// 
/// ```lisp
/// (list 1 4 2 3)
/// ```
fn list_insert(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Get the index
    let index = match args.get(1) {
        Some(Expr::Int(index)) => *index as usize,
        _ => return Expr::None,
    };

    // Get the element to insert
    let element = env.eval(args.get(2).cloned().unwrap_or(Expr::None));

    // Check if the container is a list
    let mut list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Insert the element at the specified index
    list.insert(index, element);

    Expr::List(list)
}

/// Removes an element at the specified index in a list.
/// 
/// # Example
/// 
/// ```lisp
/// (remove (list 1 2 3) 1)
/// ```
/// 
/// ## Output
/// 
/// ```lisp
/// (list 1 3)
/// ```
fn list_remove(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Get the index
    let index = match args.get(1) {
        Some(Expr::Int(index)) => *index as usize,
        _ => return Expr::None,
    };

    // Check if the container is a list
    let mut list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Remove the element at the specified index
    list.remove(index);

    Expr::List(list)
}



/// Reverses the elements of a list.
/// 
/// # Example
/// 
/// ```lisp
/// (reverse (list 1 2 3))
/// ```
/// 
/// ## Output
/// 
/// ```lisp
/// (list 3 2 1)
/// ```
fn list_reverse(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Check if the container is a list
    let mut list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Reverse the list
    list.reverse();

    Expr::List(list)
}

/// Extend a list with the elements of another list.
/// 
/// # Example
/// 
/// ```lisp
/// (extend (list 1 2) (list 3 4))
/// ```
/// 
/// ## Output
/// 
/// ```lisp
/// (list 1 2 3 4)
/// ```
fn list_extend(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the first list
    let container1 = env.eval(args.get(0).cloned().unwrap_or(Expr::None));
    let container2 = env.eval(args.get(1).cloned().unwrap_or(Expr::None));

    // Check if the container is a list
    let mut list1 = match container1 {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    let mut list2 = match container2 {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Extend the first list with the elements of the second list
    list1.append(&mut list2);

    Expr::List(list1)
}

/// Splits a list into two lists at the specified index.
/// 
/// # Example
/// 
/// ```lisp
/// (split (list 1 2 3 4 5) 2)
/// ```
/// 
/// ## Output
/// 
/// ```lisp
/// ((list 1 2) (list 3 4 5))
/// ```
/// 
/// # Example
/// 
/// ```lisp
/// (split (list 1 2 3 4 5) 0)
/// ```
/// 
/// ## Output
/// 
/// ```lisp
/// ((list) (list 1 2 3 4 5))
/// ```
fn list_split(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Get the index
    let index = match args.get(1) {
        Some(Expr::Int(index)) => *index as usize,
        _ => return Expr::None,
    };

    // Check if the container is a list
    let list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Split the list at the specified index
    let (list1, list2) = list.split_at(index);

    Expr::List(vec![Expr::List(list1.to_vec()), Expr::List(list2.to_vec())])
}

/// Sorts a list in ascending order.
/// 
/// # Example
/// 
/// ```lisp
/// (sort (list 3 1 2))
/// ```
/// 
/// ## Output
/// 
/// ```lisp
/// (list 1 2 3)
/// ```
fn list_sort(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Check if the container is a list
    let mut list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Sort the list
    list.sort();

    Expr::List(list)
}

/// Sorts a list in descending order.
/// 
/// # Example
/// 
/// ```lisp
/// (sort-desc (list 3 1 2))
/// ```
/// 
/// ## Output
/// 
/// ```lisp
/// (list 3 2 1)
/// ```
fn list_sort_desc(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Check if the container is a list
    let mut list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Sort the list
    list.sort_by(|a, b| b.cmp(a));

    Expr::List(list)
}

/// Get the head element of a list
/// 
/// # Example
/// 
/// ```lisp
/// (head (list 1 2 3))
/// ```
/// 
/// ## Output
/// 
/// ```lisp
/// 1
/// ```
fn list_head(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Check if the container is a list
    let list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Get the head element of the list
    list.first().cloned().unwrap_or(Expr::None)
}

/// Get the tail of a list
/// 
/// # Example
/// 
/// ```lisp
/// (tail (list 1 2 3))
/// ```
/// 
/// ## Output
/// 
/// ```
/// (list 2 3)
/// ```
fn list_tail(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the list
    let container = env.eval(args.get(0).cloned().unwrap_or(Expr::None));

    // Check if the container is a list
    let list = match container {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Get the tail of the list
    let tail = list.iter().skip(1).cloned().collect::<Vec<Expr>>();

    Expr::List(tail)
}


/// Zip two lists into a list of pairs.
/// 
/// # Example
/// 
/// ```lisp
/// (zip (list 1 2 3) (list 4 5 6))
/// ```
/// 
/// ## Output
/// 
/// ```
/// (list (list 1 4) (list 2 5) (list 3 6))
/// ```
fn list_zip(env: &mut Env, args: Vec<Expr>) -> Expr {
    // Get the first list
    let container1 = env.eval(args.get(0).cloned().unwrap_or(Expr::None));
    let container2 = env.eval(args.get(1).cloned().unwrap_or(Expr::None));

    // Check if the container is a list
    let list1 = match container1 {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    let list2 = match container2 {
        Expr::List(list) => list.clone(),
        _ => return Expr::None,
    };

    // Zip the two lists
    let zipped = list1.iter().zip(list2.iter()).map(|(a, b)| Expr::List(vec![a.clone(), b.clone()])).collect::<Vec<Expr>>();
    Expr::List(zipped)
}
