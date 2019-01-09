//! ## 单向链表
//! 多数区块链项目采用单向链表作为其核心数据结构。

use std::{fmt::Display, rc::Rc};

type SizType = u64;

/// 链结构。
pub struct OneWayLinkedList<T: Clone + Display> {
    len: SizType,
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
        OneWayLinkedList { len: 0, head: None }
    }

    /// 向链表中添加一个节点。
    pub fn add(&mut self, data: T) {
        let new = Some(Rc::new(Node {
            data: data.clone(),
            back: self.head.as_ref().map(|h| Rc::clone(h)),
        }));

        self.head = new;
        self.len += 1;
    }

    /// 删除最新的节点，并返回其值；若链表为空，则返回 None。
    pub fn pop(&mut self) -> Option<T> {
        let res;
        if 0 == self.len {
            res = None;
        } else {
            res = Some(self.head.as_ref().unwrap().data.clone());

            if 1 == self.len {
                self.head = None;
            } else {
                self.head.as_mut().map(|h| {
                    *h = Rc::clone(h.back.as_ref().unwrap());
                });
            }

            self.len -= 1;
        }
        res
    }

    /// 返回链表中所有节点的个数。
    pub fn len(&self) -> SizType {
        self.len
    }

    /// 按 **从新到旧** 的顺序依次打印每个节点的值。
    pub fn stringify(&self) -> String {
        let mut res = String::new();

        let mut p = self.head.as_ref();
        while let Some(n) = p {
            let Node {
                data: ref d,
                back: ref b,
            } = **n;
            res.push_str(&format!("{}==>", d));
            p = b.as_ref();
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

        for x in 0..=9 {
            list.add(x);
        }
        println!("{}", list.stringify());
    }
}
