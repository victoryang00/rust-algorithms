use std::cell::RefCell;
use std::rc::Rc;
use rand::*;

type RealNode = Rc<RefCell<Node>>;
type Link = Option<Rc<RefCell<Node>>>;

#[derive(Debug, Clone)]
struct Node {
    data: String,
    next: Vec<Link>,
    offset: u64,
}

impl Node {
    fn new(next: Vec<Link>, offset: u64, data: String) -> RealNode {
        Rc::new(RefCell::new(Node { next, offset, data }))
    }
}

#[derive(Debug, Clone)]
pub struct SkipList {
    head: Link,
    tails: Vec<Link>,
    max_level: usize,
    length: u64,
}

impl SkipList {
    pub fn new(level: usize) -> Self {
        SkipList {
            head: None,
            tails: vec![None; level],
            max_level: level - 1,
            length: 0,
        }
    }

    fn random_level(&self) -> usize {
        let mut n = 0;
        while random::<bool>() && n < self.max_level {
            n += 1;
        }
        n
    }

    pub fn append(&mut self, offset: u64, data: String) {
        let level = 1 + if self.head.is_none() {
            self.max_level
        } else {
            self.random_level()
        };
            let node = Node::new(vec![None; level], offset, data);
            for i in 0..level {
                if let Some(old) = self.tails[i].take() {
                    let next = &mut old.borrow_mut().next;
                    next[i] = Some(node.clone());
                }
                self.tails[i] = Some(node.clone());
            }
            if self.head.is_none() {
                self.head = Some(node.clone());
            }
            self.length += 1;
        }
    
        fn max_level(&self) -> usize {
            self.max_level
        }
    
        fn size(&self) -> u64 {
            self.length
        }
    
        fn level_path(&self) {
            match self.head {
                Some(ref head) => {
                    let node = head.clone();
                    for level in (0..=self.max_level()).rev() {
                        let mut n = node.clone();
                        print!("level={:?}", level);
                        loop {
                            let next = n.clone();
                            print!(
                                "offset={:?}, data={:?}\t",
                                next.borrow().offset,
                                next.borrow().data
                            );
                            match next.borrow().next[level] {
                                Some(ref next) => {
                                    n = next.clone();
                                }
                                _ => break,
                            };
                        }
                    }
                }
                None => {}
            }
        }
    pub fn find(&self, offset: u64) -> Option<String> {
        match self.head {
            Some(ref head) => {
                let mut start_level = self.max_level - 1; // should be max_level-1
                let node = head.clone();
                let mut result = None;
                loop {
                    if node.borrow().next[start_level].is_some() {
                        break;
                    }
                    start_level -= 1;
                }
                let mut n = node;
                for level in (0..=start_level).rev() {
                    loop {
                        let next = n.clone();
                        match next.borrow().next[level] {
                            Some(ref tmp) => {
                                if tmp.borrow().offset <= offset {
                                    n = tmp.clone();
                                } else {
                                    break;
                                }
                            }
                            _ => break,
                        };
                    }
                    if n.borrow().offset == offset {
                        let tmp = n.borrow();
                        result = Some(tmp.data.clone());
                        break;
                    }
                }
                result
            }
            None => None,
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_skip_list() {
        let mut skl = SkipList::new(5);
        for i in 1..1000 {
            skl.append(i, format!("data-{}", i));
        }
        for i in 1..1000 {
            assert_eq!(skl.find(i), Some(format!("data-{}", i)));
        }
    }
}