use super::{Key, Node};
use std::{error::Error, fmt, rc::Rc};

//NotExists(None): 跳表为空
//NotExists(Some(_)): 目标节点的左/右兄弟
//UnitSizIsZero:初始化跳表时,指定的单元大小是0
//UnitSizNotEven:初始化跳表时,指定的单元大小不是偶数
pub enum XErr<K: Key, V: Clone> {
    NotExists(Option<Rc<Node<K, V>>>),
    AlreadyExists(Rc<Node<K, V>>),
    UnitSizNotEven,
    UnitSizIsZero,
    Unknown,
}

impl<K: Key, V: Clone> fmt::Display for XErr<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XErr::NotExists(_) => write!(f, "Not Exists!"),
            XErr::AlreadyExists(_) => write!(f, "Key/Value Already Exists!"),
            XErr::UnitSizNotEven => write!(f, "Unit Size Must Be Even!"),
            XErr::UnitSizIsZero => write!(f, "Unit Size Can Not Be Zero!"),
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
            XErr::AlreadyExists(_) => "Key/Value Already Exists!",
            XErr::UnitSizNotEven => "Unit Size Must Be Even!",
            XErr::UnitSizIsZero => "Unit Size Can Not Be Zero!",
            XErr::Unknown => "Unknown Error!",
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
