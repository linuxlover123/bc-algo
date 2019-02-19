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

mod error;
use error::XErr;
use std::rc::{Rc, Weak};

pub trait Key: Clone + Eq + PartialEq + Ord + PartialOrd {}

pub struct CrossList<K: Key, V: Clone> {
    unit_siz: usize,  //必须是奇数
    lower_idx: usize, //直连子节点在自身children中的索引,所有节点都是一致的

    kv_cnt: usize,                //全局key-value总数量
    root: Option<Rc<Node<K, V>>>, //根节点,全局入口节点
}

//节点删除时,必须从下而上逐一操作,否则将造成内存泄漏
struct Node<K: Key, V: Clone> {
    key: K,
    value: Rc<V>,

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

    //查询成功，返回目标节点指针；
    //查询失败，返回其左兄弟或右兄弟(若全局为空表，返回None)
    fn query(&self, key: K) -> Result<Rc<Node<K, V>>, XErr<K, V>> {
        let mut node = if let Some(r) = self.root.as_ref() {
            let root = Rc::clone(r);
            if key == root.key {
                return Ok(root);
            }
            root
        } else {
            return Err(XErr::NotExists(None));
        };

        loop {
            if key > node.key {
                if node.right.is_none() {
                    if node.children.is_empty() {
                        return Err(XErr::NotExists(Some(node)));
                    } else {
                        //进入下一层继续查找
                        node = Rc::clone(&node.children[self.lower_idx]);
                    }
                } else {
                    while let Some(right) = node.right.as_ref() {
                        if key > right.key {
                            node = Rc::clone(right);
                        } else if key == right.key {
                            return Ok(Rc::clone(right));
                        } else if right.children.is_empty() {
                            return Err(XErr::NotExists(Some(Rc::clone(right))));
                        } else {
                            //进入下一层继续查找
                            node = Rc::clone(&right.children[self.lower_idx]);
                            break;
                        }
                    }
                }
            } else {
                if node.left.is_none() {
                    if node.children.is_empty() {
                        return Err(XErr::NotExists(Some(node)));
                    } else {
                        //进入下一层继续查找
                        node = Rc::clone(&node.children[self.lower_idx]);
                    }
                } else {
                    while let Some(left) = node.left.as_ref() {
                        if key < left.key {
                            node = Rc::clone(left);
                        } else if key == left.key {
                            return Ok(Rc::clone(left));
                        } else if left.children.is_empty() {
                            return Err(XErr::NotExists(Some(Rc::clone(left))));
                        } else {
                            //进入下一层继续查找
                            node = Rc::clone(&left.children[self.lower_idx]);
                            break;
                        }
                    }
                }
            }
        }
    }

    //更新指定的key对应的value
    fn update(&mut self, key: K, value: V) -> Result<(), XErr<K, V>> {
        let node = self.query(key)?;
        let raw = Rc::into_raw(node) as *mut Node<K, V>;
        unsafe {
            (*raw).value = Rc::new(value);
            Rc::from_raw(raw);
        }
        Ok(())
    }

    fn insert(&mut self, key: K, value: V) -> Result<Rc<Node<K, V>>, XErr<K, V>> {
        unimplemented!();
    }

    fn remove(&mut self, key: K) -> Result<Rc<Node<K, V>>, XErr<K, V>> {
        let mut raw;
        let mut node = self.get_lowest_node(self.query(key)?);

        while let Some(upper) = node.upper.as_ref() {
            //TODO?
            //if node本身是散落节点 {
            //    直接删除之,将其左右邻直接串连在一起即可
            //} else if 左侧或右侧存在散落的节点 {
            //    1.删除node节点
            //    2.将左侧或右侧最近一个散落节点移入父节点的children中
            //    3.将node的父节点的新的children[self.lower_idx]的key/value复制给其父节点
            //    (若未变动,则无需复制,直接结束即可),若其父节点也是父父节点的直连节点,
            //    则以同样的逻辑循环向上处理,直至条件不再满足或到达顶层
            //} else if 左右都不存在散落的节点 {
            //    1.删除node节点
            //    2.将node的所有原兄弟节点的父节点指针置为None
            //    3.将node的所有原兄弟节点指针复制出来,互相之间及与左右邻之间重新串连
            //    3.删除node的直辖父节点(递归调用本函数处理)
            //}
        }

        //检测是否需要处理项层节点
        if node.upper.is_none() {
            //根节点
            if Rc::ptr_eq(&node, self.root.as_ref().unwrap()) {
                self.root = node.right.clone();
            }
            //首层节点,但非根节点
            else {
                raw = Rc::into_raw(Rc::clone(node.left.as_ref().unwrap())) as *mut Node<K, V>;
                unsafe {
                    (*raw).right = node.right.clone();
                    Rc::from_raw(raw);
                }
                if let Some(right) = node.right.as_ref() {
                    raw = Rc::into_raw(Rc::clone(right)) as *mut Node<K, V>;
                    unsafe {
                        (*raw).left = node.left.clone();
                        Rc::from_raw(raw);
                    }
                }
            }
        }

        Ok(node)
    }

    fn get_lowest_node(&self, node: Rc<Node<K, V>>) -> Rc<Node<K, V>> {
        let mut n = node;
        while !n.children.is_empty() {
            n = Rc::clone(&n.children[self.lower_idx]);
        }
        n
    }
}

impl<K: Key, V: Clone> Node<K, V> {
    fn new(k: K, v: V) -> Node<K, V> {
        Node {
            key: k,
            value: Rc::new(v),
            left: None,
            right: None,
            upper: None,
            children: Vec::with_capacity(0),
        }
    }
}
