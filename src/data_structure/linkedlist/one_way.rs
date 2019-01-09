//! ## 单向链表
//! 多数区块链项目采用单向链表作为其核心数据结构。

use std::{
    fmt::Display,
    rc::Rc,
};

type SizType=u64;

/// 链结构。
pub struct OneWayLinkedList<T: Clone + Display> {
    len: SizType ,
    head: Option<Rc<Node<T>>>,
}

/// 节点结构。
#[derive(Clone)]
struct Node<T: Clone + Display> {
    data: T,
    back: Option<Rc<Node<T>>>,
}

impl<T: Clone + Display> OneWayLinkedList<T> {
    /// 初始化一个新链表。
    pub fn new() -> OneWayLinkedList<T> {
        OneWayLinkedList {
            len: 0,
            head: None 
        }
    }

    /// 向链表中添加一个节点。
    pub fn add(&mut self, data: T) {
        let new = Some(Rc::new(
            Node{
                data: data.clone(),
                back: self.head.as_ref().map(|h|Rc::clone(h)), 
            }));

        self.head = new;
        self.len += 1;
    }

    /// 删除最新的节点，并返回其值；若链表为空，则返回 None。
    pub fn pop(&mut self) -> Option<T> {
        if 0 == self.len {
            return None;
        } else if 1 == self.len {
            let keep = Rc::clone(self.head.as_ref().unwrap());

            self.len -= 1;
            self.head = None;

            return Some((*keep).data.clone());
        } else {
            let keep = Rc::clone(self.head.as_ref().unwrap());

            self.len -= 1;
            self.head.as_mut().map(|h|Rc::clone(h.back.as_ref().unwrap()));
            return Some((*keep).data.clone());
        }
    }

    /// 返回链表中所有节点的个数。
    pub fn len(&self) -> SizType {
        self.len
    }

    /// 按 **从新到旧** 的顺序依次打印每个节点的值。
    pub fn stringify(&self) -> String {
        let mut res = String::new();

        let mut p = Rc::clone(self.head.as_ref().unwrap());
        for _ in 0..self.len {
            let Node{data: ref d, back: ref b} = *p;
            res.push_str(&format!("{}==>", d));
            p = Rc::clone(b.as_ref().unwrap());
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
        let mut list = OneWayLinkedList::new();
        for x in 0..=99 {
            list.add(x);
        }
        assert_eq!(list.len, 100);

        assert_eq!(Some(99), list.pop());
        assert_eq!(Some(98), list.pop());
        assert_eq!(Some(97), list.pop());
        assert_eq!(list.len, 97);

        for x in 0..97 {
            assert_eq!(Some(96 - x), list.pop());
        }
        assert_eq!(list.len, 0);
        assert_eq!(None, list.pop());

        println!("{}", list.stringify());
    }
}
