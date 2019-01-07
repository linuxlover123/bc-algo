use std::{
    rc::Rc,
    cell::RefCell
};

type SizeT=u64;
type DataT=i32;

struct List {
    len: SizeT ,
    header: Rc<RefCell<Node>>,
    tail: Rc<RefCell<Node>>,
}

#[derive(Clone)]
enum Node {
    Nil,
    Obj(DataT, Rc<RefCell<Node>>),
}

impl List {
    pub fn new() -> List {
        List {
            len: 0,
            header: Rc::new(RefCell::new(Node::Nil)),
            tail: Rc::new(RefCell::new(Node::Nil)),
        }
    }

    // 正向追加节点
    pub fn prevadd(&mut self, data: DataT) {
        self.len += 1;
        self.header = Rc::new(RefCell::new(Node::Obj(data, Rc::clone(&self.header))));

        // optimize to execute once lazily?
        if 1 == self.len {
            self.tail = Rc::clone(&self.header);
        }
    }

    // 反向追加节点
    pub fn backadd(&mut self, data: DataT) {
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
    pub fn prevpop(&mut self) -> DataT {
        unimplemented!();
    }

    // 弹出最后面的节点
    pub fn backpop(&mut self) -> DataT {
        unimplemented!();
    }

    pub fn len(&self) -> SizeT {
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

        //assert_eq!(99, list.prevpop());
        //assert_eq!(98, list.prevpop());
        //assert_eq!(97, list.prevpop());
        //assert_eq!(-100, list.backpop());
        //assert_eq!(-99, list.backpop());
        //assert_eq!(-99, list.backpop());

        println!("{}", list.stringify());
    }
}
