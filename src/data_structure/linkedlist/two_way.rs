//! ## 双向链表
//!
//! #### 属性
//! - <font color=Green>√</font> 多线程安全
//! - <font color=Green>√</font> 无 unsafe 代码
//!
//! #### 说明
//! - 具备双向索引能力的链表。
//!
//! #### 应用场景
//! - 大幅频繁变动的列表集。

use std::fmt::Display;
use std::sync::Arc;
use std::sync::RwLock;

type SizType = u64;

/// 链结构。
#[derive(Clone)]
pub struct TwoWayLinkedList<T: Clone + Display>(Arc<RwLock<List<T>>>);

struct List<T: Clone + Display> {
    len: SizType,
    head: Option<Arc<Node<T>>>,
    tail: Option<Arc<Node<T>>>,
}

/// 节点结构。
#[derive(Clone)]
struct Node<T: Clone + Display> {
    data: T,
    prev: Option<Arc<Node<T>>>,
    back: Option<Arc<Node<T>>>,
}

impl<T: Clone + Display> TwoWayLinkedList<T> {
    /// 初始化一个新链表。
    pub fn new() -> TwoWayLinkedList<T> {
        TwoWayLinkedList(Arc::new(RwLock::new(List {
            len: 0,
            head: None,
            tail: None,
        })))
    }

    /// 前向追加节点。
    pub fn prevadd(&self, data: T) {
        let mut me = self.0.write().unwrap();

        let new = Arc::new(Node {
            data,
            prev: None,
            back: me.head.as_ref().map(|h| Arc::clone(h)),
        });

        me.head = Some(Arc::clone(&new));

        if 0 == me.len {
            me.tail = Some(new);
        } else {
            Arc::get_mut(me.head.as_mut().unwrap()).map(|h| {
                Arc::get_mut(h.back.as_mut().unwrap()).map(|b| {
                    b.prev = Some(new);
                })
            });
        }

        me.len += 1;
    }

    /// 后向追加节点。
    pub fn backadd(&self, data: T) {
        let mut me = self.0.write().unwrap();

        let new = Arc::new(Node {
            data,
            prev: me.tail.as_ref().map(|t| Arc::clone(t)),
            back: None,
        });

        me.tail = Some(Arc::clone(&new));

        if 0 == me.len {
            me.head = Some(new);
        } else {
            Arc::get_mut(me.tail.as_mut().unwrap()).map(|t| {
                Arc::get_mut(t.prev.as_mut().unwrap()).map(|p| {
                    p.back = Some(new);
                })
            });
        }

        me.len += 1;
    }

    /// 弹出最前面的节点。
    pub fn prevpop(&self) -> Option<T> {
        let mut me = self.0.write().unwrap();
        let res;

        if 0 == me.len {
            res = None;
        } else {
            res = Some(me.head.as_ref().unwrap().data.clone());

            if 1 == me.len {
                me.head = None;
                me.tail = None;
            } else {
                me.head.as_mut().map(|h| {
                    *h = Arc::clone(h.back.as_ref().unwrap());
                    Arc::get_mut(h).unwrap().prev = None;
                });
            }

            me.len -= 1;
        }

        res
    }

    /// 弹出最后面的节点。
    pub fn backpop(&self) -> Option<T> {
        let mut me = self.0.write().unwrap();
        let res;

        if 0 == me.len {
            res = None;
        } else {
            res = Some(me.tail.as_ref().unwrap().data.clone());

            if 1 == me.len {
                me.head = None;
                me.tail = None;
            } else {
                me.tail.as_mut().map(|t| {
                    *t = Arc::clone(t.prev.as_ref().unwrap());
                    Arc::get_mut(t).unwrap().back = None;
                });
            }

            me.len -= 1;
        }

        res
    }

    /// 返回链表中所有节点的个数。
    pub fn len(&self) -> SizType {
        self.0.read().unwrap().len
    }

    /// 按 **prev ==> back** 的顺序依次打印每个节点的值。
    pub fn stringify(&self) -> String {
        let me = self.0.read().unwrap();
        let mut res = String::new();

        let mut ptr = me.tail.as_ref();
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
    use std::thread;

    #[test]
    fn test() {
        let l = TwoWayLinkedList::new();

        let list = l.clone();
        let tid = thread::spawn(move || {
            for x in 0..=99 {
                list.prevadd(x);
            }
            for x in 1..=100 {
                list.backadd(-x);
            }
        });

        tid.join().unwrap();
        assert_eq!(l.len(), 200);

        let list = l.clone();
        let tid = thread::spawn(move || {
            list.prevpop().unwrap();
            list.prevpop().unwrap();
            list.prevpop().unwrap();
            list.backpop().unwrap();
            list.backpop().unwrap();
            list.backpop().unwrap();
        });

        tid.join().unwrap();
        assert_eq!(l.len(), 194);

        assert_eq!(96, l.prevpop().unwrap());
        assert_eq!(-97, l.backpop().unwrap());

        println!("{}", l.stringify());
    }
}
