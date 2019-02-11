use super::{AsBytes, Node};
use std::{error::Error, fmt, rc::Rc};

///- @XErr::HashLen: 哈希长度不一致
///- @XErr::HashCollision: 哈希碰撞
///- @XErr::NotExists: 目标不存在
pub enum XErr<V: AsBytes> {
    HashLen,
    NotExists(Rc<Node<V>>),
    HashCollision(Rc<Node<V>>),
    Unknown,
}

impl<V: AsBytes> fmt::Display for XErr<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XErr::HashLen => write!(f, "Invalid hashsig length!"),
            XErr::NotExists(_) => write!(f, "Not exists!"),
            XErr::HashCollision(_) => write!(f, "Hash collision!"),
            XErr::Unknown => write!(f, "Unknown error!"),
        }
    }
}

impl<V: AsBytes> fmt::Debug for XErr<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<V: AsBytes> Error for XErr<V> {
    fn description(&self) -> &str {
        match self {
            XErr::HashLen => "Invalid hashsig length!",
            XErr::NotExists(_) => "Not exists!",
            XErr::HashCollision(_) => "Hash collision!",
            XErr::Unknown => "Unknown error!",
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
