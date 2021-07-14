use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

struct Node{
    value:i32,
    next:HashMap<char, Rc<RefCell<Node>>>,
}