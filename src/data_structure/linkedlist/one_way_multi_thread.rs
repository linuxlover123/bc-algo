use std::{
    rc::Rc,
    cell::RefCell,
    fmt::Display,
};

type SizType=u64;

struct List<T: Clone + Display> {
    len: SizType ,
    header: Rc<RefCell<Node<T>>>,
    tail: Rc<RefCell<Node<T>>>,
}

#[derive(Clone)]
enum Node<T: Clone + Display> {
    Nil,
    Obj(T, Rc<RefCell<Node<T>>>),
}

impl<T: Clone + Display> List<T> {
    pub fn new() -> List<T> {
        List {
            len: 0,
            header: Rc::new(RefCell::new(Node::Nil)),
            tail: Rc::new(RefCell::new(Node::Nil)),
        }
    }

    // 前向追加节点
    pub fn prevadd(&mut self, data: T) {
        self.len += 1;
        self.header = Rc::new(RefCell::new(Node::Obj(data, Rc::clone(&self.header))));

        // optimize to execute once lazily?
        if 1 == self.len {
            self.tail = Rc::clone(&self.header);
        }
    }

    // 后向追加节点
    pub fn backadd(&mut self, data: T) {
        if 0 == self.len {
            self.prevadd(data);
        } else {
            self.len += 1;

            let keep = Rc::clone(&self.tail);
            if let Node::Obj(_, ref p) = *keep.borrow() {
                *p.borrow_mut() = Node::Obj(data, Rc::new(RefCell::new(Node::Nil)));
                self.tail = Rc::clone(p);
            } else {
                panic!("BUG!");
            }

            let _ = keep;
        }
    }

    // 弹出最前面的节点
    pub fn prevpop(&mut self) -> Option<T> {
        if 0 == self.len {
            return None;
        } else {
            self.len -= 1;

            let keep = Rc::clone(&self.header);
            let res;
            if let Node::Obj(data, ref p) = *keep.borrow() {
                res = data;
                self.header = Rc::clone(p);
            } else {
                panic!("BUG!");
            }

            let _ = keep;
            return res;
        }
    }

    // 弹出最后面的节点
    pub fn backpop(&mut self) -> Option<T> {
        if 0 == self.len {
            return None;
        } else {
            self.len -= 1;

            let keep = Rc::clone(&self.tail);
            let res;
            if let Node::Obj(data, ref p) = *keep.borrow() {
                res = data;
                //self.tail = Rc::clone(p);
            } else {
                panic!("BUG!");
            }

            let _ = keep;
            return res;
        }
    }

    pub fn len(&self) -> SizType {
        self.len
    }

    pub fn stringify(&self) -> String {
        let mut res = String::new();

        let mut p = Rc::clone(&self.header).borrow().clone();
        while let Node::Obj(node, next) = p {
            res.push_str(&format!("{}==>", node));
            p = Rc::clone(&next).borrow().clone();
        }

        res.push_str(&format!("Nil"));
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut list = List::new();
        for x in 0..=99 {
            list.prevadd(x);
        }
        assert_eq!(list.len, 100);

        for x in 1..=100 {
            list.backadd(-x);
        }
        assert_eq!(list.len, 200);

        assert_eq!(99, list.prevpop().unwrap());
        assert_eq!(98, list.prevpop().unwrap());
        assert_eq!(97, list.prevpop().unwrap());
        assert_eq!(-100, list.backpop().unwrap());
        assert_eq!(-99, list.backpop().unwrap());
        assert_eq!(-99, list.backpop().unwrap());

        println!("{}", list.stringify());
    }
}
