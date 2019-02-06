use super::{AsBytes, Node};
use std::{error::Error, fmt, rc::Rc};

///- @XErr::HashCollision: 哈希长度不一致
///- @XErr::NotExists: 哈希碰撞
pub enum XErr<V: Clone + AsBytes> {
    HashLen,
    NotExists(Rc<Node<V>>),
    HashCollision(Rc<Node<V>>),
    Unknown,
}

impl<V: Clone + AsBytes> fmt::Display for XErr<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XErr::HashLen => write!(f, "Invalid hashsig length!"),
            XErr::NotExists(_) => write!(f, "Not exists!"),
            XErr::HashCollision(_) => write!(f, "Hash collision!"),
            XErr::Unknown => write!(f, "Unknown error!"),
        }
    }
}

impl<V: Clone + AsBytes> fmt::Debug for XErr<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<V: Clone + AsBytes> Error for XErr<V> {
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
