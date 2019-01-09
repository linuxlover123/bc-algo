//! ## 双向链表
//!
//! #### 属性
//! - <font color=Red>×</font> 多线程安全
//! - <font color=Green>√</font> 无 unsafe 代码
//!
//! #### 说明
//! - 具备双向索引能力的链表。
//!
//! #### 应用场景

use std::{rc::Rc, fmt::Display};

type SizType = u64;

/// 链结构。
pub struct TwoWayLinkedList<T: Clone + Display> {
    len: SizType,
    head: Option<Rc<Node<T>>>,
    tail: Option<Rc<Node<T>>>,
}

/// 节点结构。
#[derive(Clone)]
struct Node<T: Clone + Display> {
    data: T,
    prev: Option<Rc<Node<T>>>,
    back: Option<Rc<Node<T>>>,
}

impl<T: Clone + Display> TwoWayLinkedList<T> {
    /// 初始化一个新链表。
    pub fn new() -> TwoWayLinkedList<T> {
        TwoWayLinkedList {
            len: 0,
            head: None,
            tail: None,
        }
    }

    /// 前向追加节点。
    pub fn prevadd(&mut self, data: T) {
        let new = Rc::new(Node {
            data,
            prev: None,
            back: self.head.as_ref().map(|h|Rc::clone(h)),
        });

        if 0 == self.len {
            self.tail = Some(Rc::clone(&new));
        } else {
            self.head.as_mut().map(|h|{Rc::get_mut(h).unwrap().prev = Some(Rc::clone(&new));});
        }

        self.head = Some(new);
        self.len += 1;
    }

    /// 后向追加节点。
    pub fn backadd(&mut self, data: T) {
        let new = Some(Rc::new(Node {
            data,
            prev: self.tail.as_ref().map(|t|Rc::clone(t)),
            back: None,
        }));

        if 0 == self.len {
            self.head = Some(Rc::clone(new.as_ref().unwrap()));
        } else {
            self.tail.as_mut().map(|t|{Rc::get_mut(t).unwrap().back = Some(Rc::clone(new.as_ref().unwrap()))});
        }

        self.tail = new;
        //self.tail.as_mut().map(|t|{*t = Rc::clone(new.as_ref().unwrap());});
        self.len += 1;
    }

    /// 弹出最前面的节点。
    pub fn prevpop(&mut self) -> Option<T> {
        let res;

        if 0 == self.len {
            res = None;
        } else {
            res = Some(self.head.as_ref().unwrap().data.clone());

            if 1 == self.len {
                self.head = None;
                self.tail = None;
            } else {
                self.head.as_mut().map(|h|{
                    *h = Rc::clone(h.back.as_ref().unwrap());
                    Rc::get_mut(h).unwrap().prev = None;
                });
            }

            self.len -= 1;
        }

        res
    }

    /// 弹出最后面的节点。
    pub fn backpop(&mut self) -> Option<T> {
        let res;

        if 0 == self.len {
            res = None;
        } else {
            res = Some(self.tail.as_ref().unwrap().data.clone());

            if 1 == self.len {
                self.head = None;
                self.tail = None;
            } else {
                self.tail.as_mut().map(|t|{
                    *t = Rc::clone(t.prev.as_ref().unwrap());
                    Rc::get_mut(t).unwrap().back = None;
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

    /// 按 **prev ==> back** 的顺序依次打印每个节点的值。
    pub fn stringify(&self) -> String {
        let mut res = String::new();

        let mut ptr = self.tail.as_ref();
        while let Some(t) = ptr {
            let Node {
                data: ref d,
                prev: ref p,
                back: _,
            } = **t;
            res.push_str(&format!("{}==>", d));
            ptr = p.as_ref();
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
        let mut list = TwoWayLinkedList::new();
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
        assert_eq!(-98, list.backpop().unwrap());
        assert_eq!(list.len, 194);

        println!("{}", list.stringify());
    }
}
