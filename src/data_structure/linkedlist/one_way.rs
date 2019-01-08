//! ## 单向链表
//! 多数区块链项目采用单向链表作为其核心数据结构。

use std::{
    fmt::Display,
    ptr,
};

type SizType=u64;

/// 链结构。
pub struct OneWayLinkedList<T: Clone + Display> {
    len: SizType ,
    head: *mut Node<T>,
}

/// 节点结构。
#[derive(Clone)]
struct Node<T: Clone + Display> {
    data: T,
    back: *mut Node<T>,
}

impl<T: Clone + Display> OneWayLinkedList<T> {
    /// 初始化一个新链表。
    pub fn new() -> OneWayLinkedList<T> {
        OneWayLinkedList {
            len: 0,
            head: ptr::null_mut(),
        }
    }

    /// 向链表中添加一个节点。
    pub fn add(&mut self, data: T) {
        let new = Box::into_raw(Box::new(
            Node{
                data,
                back: self.head, 
            }));

        self.head = new;
        self.len += 1;
    }

    /// 删除最新的节点，并返回其值；若链表为空，则返回 None。
    pub fn pop(&mut self) -> Option<T> {
        if 0 == self.len {
            return None;
        } else if 1 == self.len {
            let keep = self.head;

            self.len -= 1;
            self.head = ptr::null_mut();

            unsafe { return Some(Box::<Node<T>>::from_raw(keep).data); }
        } else {
            let keep = self.head;

            self.len -= 1;
            unsafe {
                self.head = (*keep).back;
                return Some(Box::<Node<T>>::from_raw(keep).data);
            };
        }
    }

    /// 返回链表中所有节点的个数。
    pub fn len(&self) -> SizType {
        self.len
    }

    /// 按**从新到旧**的顺序依次打印每个节点的值。
    pub fn stringify(&self) -> String {
        let mut res = String::new();

        let mut p = self.head;
        for _ in 0..self.len {
            unsafe {
                let Node{data, back} = *Box::<Node<T>>::from_raw(p);
                res.push_str(&format!("{}==>", data));
                p = back;
            }
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
