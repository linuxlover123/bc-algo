//! ## Skip List
//!
//! #### 算法说明
//! - 读写效率与AVL、红黑树等相当；
//! - 易于理解和实现；
//! - 结构固定，可提供merkle proof。
//!
//! #### 应用场景
//! - 数据检索。
//!
//! #### 实现属性
//! - <font color=Red>×</font> 多线程安全
//! - <font color=Red>×</font> 无 unsafe 代码
//!
//! #### Example
//!```
//!```

use error::XErr;
use std::collections::LinkedList;
use std::rc::{Rc, Weak};

pub trait SL: Eq + PartialEq + Ord + PartialOrd {}

//- @unit_siz: 单元内元素数量超过此值将进行分裂
//- @headers: 每一层的第一个元素，headers[0]即为root节点
pub struct SkipList<T: SL> {
    unit_siz: usize,
    headers: Vec<Rc<Node<T>>>,
}

//- @data: 存储的数据类型
//- @lower: 自身对应的下一层节点(除底层节点外，一定存在)
//- @upper: 自身对应的上一层节点(不一定存在)
pub struct Node<T: SL> {
    data: Rc<T>,
    lower: Option<Rc<Node<T>>>,
    upper: Weak<Node<T>>,
}

impl<T: SL> SkipList<T> {
    ///#### 创建跳表
    pub fn new(unit_siz: usize) -> SkipList<T> {
        SkipList {
            unit_siz,
            headers: vec![],
        }
    }

    ///#### 查询数据
    pub fn get(data: Rc<T>) -> Result<Rc<Node<T>>, XErr<T>> {
        unimplemented!();
    }

    ///#### 插入数据
    pub fn put(data: T) -> Result<(), XErr<T>> {
        unimplemented!();
    }

    ///#### 删除数据
    pub fn remove(data: Rc<T>) -> Result<(), XErr<T>> {
        unimplemented!();
    }

    ///#### 销毁跳表
    pub fn destroy(data: Rc<T>) -> Result<(), XErr<T>> {
        unimplemented!();
    }
}

impl<T: SL> Node<T> {
    //#### 创建节点
    fn new(data: T) -> Node<T> {
        Node {
            data: Rc::new(data),
            lower: None,
            upper: Weak::new(),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn sl() {
        assert_eq!(1, 1);
    }
}

mod error {
    use super::{Node, SL};
    use std::{error::Error, fmt, rc::Rc};

    pub enum XErr<T: SL> {
        NotExists(Rc<Node<T>>),
        Unknown,
    }

    impl<T: SL> fmt::Display for XErr<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                XErr::NotExists(_) => write!(f, "Not exists!"),
                XErr::Unknown => write!(f, "Unknown error!"),
            }
        }
    }

    impl<T: SL> fmt::Debug for XErr<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Display::fmt(self, f)
        }
    }

    impl<T: SL> Error for XErr<T> {
        fn description(&self) -> &str {
            match self {
                XErr::NotExists(_) => "Not exists!",
                XErr::Unknown => "Unknown error!",
            }
        }

        fn source(&self) -> Option<&(dyn Error + 'static)> {
            None
        }
    }
}
