use super::{Key, Node};
use std::{error::Error, fmt, rc::Rc};

//NotExists(None): 跳表为空
//NotExists(Some(_)): 目标节点的左/右兄弟
pub enum XErr<K: Key, V: Clone> {
    NotExists(Option<Rc<Node<K, V>>>),
    Unknown,
}

impl<K: Key, V: Clone> fmt::Display for XErr<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XErr::NotExists(_) => write!(f, "Not Exists!"),
            XErr::Unknown => write!(f, "Unknown Error!"),
        }
    }
}

impl<K: Key, V: Clone> fmt::Debug for XErr<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<K: Key, V: Clone> Error for XErr<K, V> {
    fn description(&self) -> &str {
        match self {
            XErr::NotExists(_) => "Not Exists!",
            XErr::Unknown => "Unknown Error!",
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
