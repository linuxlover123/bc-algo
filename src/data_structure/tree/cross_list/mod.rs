//! ## Cross List
//!
//! #### 算法说明
//! - 跳表,多层交叉链表.
//!
//! #### 应用场景
//! - 增删改查.
//!
//! #### 实现属性
//! - <font color=Green>√</font> 多线程安全
//! - <font color=Red>×</font> 无 unsafe 代码

use std::rc::{Rc, Weak};

pub trait Key: Eq + PartialEq + Ord + PartialOrd {}

pub struct CrossList<K: Key, V: Clone> {
    unit_siz: usize, //必须是奇数
    lower_idx: usize, //直连子节点在自身children中的索引

    kv_cnt: usize,                //全局key-value总数量
    root: Option<Rc<Node<K, V>>>, //根节点,全局入口节点
}

//节点删除时,必须从下而上逐一操作,否则将造成内存泄漏
struct Node<K: Key, V: Clone> {
    key: K,
    value: V,

    left: Option<Rc<Node<K, V>>>,
    right: Option<Rc<Node<K, V>>>,
    upper: Option<Rc<Node<K, V>>>,
    children: Vec<Rc<Node<K, V>>>,
}

impl<K: Key, V: Clone> CrossList<K, V> {
    pub fn default() -> CrossList<K, V> {
        CrossList {
            unit_siz: 3,
            lower_idx: 1,
            kv_cnt: 0,
            root: None,
        }
    }

    pub fn new(siz: usize) -> CrossList<K, V> {
        CrossList {
            unit_siz: siz,
            lower_idx: siz / 2,
            kv_cnt: 0,
            root: None,
        }
    }
}

impl<K: Key, V: Clone> Node<K, V> {
    fn new(key: K, value: V) -> Node<K, V> {
        Node {
            key,
            value,
            left: None,
            right: None,
            upper: None,
            children: Vec::with_capacity(0),
        }
    }

    fn insert(&mut self, key: K, value: V) -> Result<Rc<Node<K, V>>, XErr<K, V>> {
        unimplemented!();
    }

    fn remove(&mut self) -> Result<Rc<Node<K, V>>, XErr<K, V>> {
        unimplemented!();
    }

    fn query(&self) -> Result<Rc<Node<K, V>>, XErr<K, V>> {
        unimplemented!();
    }

    fn update(&mut self) -> Result<(), XErr<K, V>> {
        unimplemented!();
    }
}
