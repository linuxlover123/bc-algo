//! ## 双向链表
//!
//! #### 算法说明
//! - 具备双向索引能力的链表；
//! - 综合效率较低。
//!
//! #### 应用场景
//! - 算法演示；
//! - 实际应用中通常会选用顺序存储构，如 Vec 等。
//!
//! #### 实现属性
//! - <font color=Green>√</font> 多线程安全
//! - <font color=Green>√</font> 无 unsafe 代码

use std::sync::Arc;
use std::sync::RwLock;

type SizType = u64;

/// 链结构。
#[derive(Clone)]
pub struct TwoWayLinkedList<T: Clone>(Arc<RwLock<List<T>>>);

struct List<T: Clone> {
    len: SizType,
    head: Arc<RwLock<Option<Node<T>>>>,
    tail: Arc<RwLock<Option<Node<T>>>>,
    none: Arc<RwLock<Option<Node<T>>>>,
}

/// 节点结构。
#[derive(Clone)]
struct Node<T: Clone> {
    data: T,
    prev: Arc<RwLock<Option<Node<T>>>>,
    back: Arc<RwLock<Option<Node<T>>>>,
}

impl<T: Clone> TwoWayLinkedList<T> {
    /// 初始化一个新链表。
    pub fn new() -> TwoWayLinkedList<T> {
        TwoWayLinkedList(Arc::new(RwLock::new(List {
            len: 0,
            head: Arc::new(RwLock::new(None)),
            tail: Arc::new(RwLock::new(None)),
            none: Arc::new(RwLock::new(None)),
        })))
    }

    /// 前向追加节点。
    pub fn prevadd(&self, data: T) {
        let mut me = self.0.write().unwrap();

        let new = Arc::new(RwLock::new(Some(Node {
            data,
            prev: Arc::clone(&me.none),
            back: Arc::clone(&me.head),
        })));

        if 0 == me.len {
            me.tail = Arc::clone(&new);
        } else {
            me.head
                .write()
                .unwrap()
                .as_mut()
                .map(|h| h.prev = Arc::clone(&new));
        }

        me.head = new;
        me.len += 1;
    }

    /// 后向追加节点。
    pub fn backadd(&self, data: T) {
        let mut me = self.0.write().unwrap();

        let new = Arc::new(RwLock::new(Some(Node {
            data,
            prev: Arc::clone(&me.tail),
            back: Arc::clone(&me.none),
        })));

        if 0 == me.len {
            me.head = Arc::clone(&new);
        } else {
            me.tail
                .write()
                .unwrap()
                .as_mut()
                .map(|t| t.back = Arc::clone(&new));
        }

        me.tail = new;
        me.len += 1;
    }

    /// 弹出最前面的节点。
    pub fn prevpop(&self) -> Option<T> {
        let mut me = self.0.write().unwrap();
        let res;

        if 0 == me.len {
            res = None;
        } else {
            res = Some(me.head.read().unwrap().as_ref().unwrap().data.clone());

            if 1 == me.len {
                me.head = Arc::clone(&me.none);
                me.tail = Arc::clone(&me.none);
            } else {
                let keep = Arc::clone(&me.head.read().unwrap().as_ref().unwrap().back);
                me.head = keep;
                me.head.write().unwrap().as_mut().map(|h| {
                    h.prev = Arc::clone(&me.none);
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
            res = Some(me.tail.read().unwrap().as_ref().unwrap().data.clone());

            if 1 == me.len {
                me.head = Arc::clone(&me.none);
                me.tail = Arc::clone(&me.none);
            } else {
                let keep = Arc::clone(&me.tail.read().unwrap().as_ref().unwrap().prev);
                me.tail = keep;
                me.tail.write().unwrap().as_mut().map(|t| {
                    t.back = Arc::clone(&me.none);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test() {
        let list = TwoWayLinkedList::new();

        let l = list.clone();
        let ll = list.clone();

        let tid_l = thread::spawn(move || {
            for x in 0..=99 {
                l.prevadd(x);
            }
        });
        let tid_ll = thread::spawn(move || {
            for x in 1..=100 {
                ll.backadd(-x);
            }
        });

        tid_l.join().unwrap();
        tid_ll.join().unwrap();
        assert_eq!(list.len(), 200);

        let l = list.clone();
        let ll = list.clone();

        let tid_l = thread::spawn(move || {
            l.prevpop().unwrap();
            l.prevpop().unwrap();
            l.prevpop().unwrap();
        });
        let tid_ll = thread::spawn(move || {
            ll.backpop().unwrap();
            ll.backpop().unwrap();
            ll.backpop().unwrap();
        });

        tid_l.join().unwrap();
        tid_ll.join().unwrap();
        assert_eq!(list.len(), 194);
    }
}
