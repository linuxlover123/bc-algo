//! ## HashMap based on Patricia Trie
//!
//! #### 算法说明
//! - 同标准库HashMap。
//!
//! #### 应用场景
//! - 数据检索。
//!
//! #### 实现属性
//! - <font color=Red>×</font> 多线程安全
//! - <font color=Red>×</font> 无 unsafe 代码

use std::error::Error;
use std::fmt::Display;
use std::rc::{Rc, Weak};
use std::hash::{Hash, Hasher};

pub trait HashX {
    ///- ##### 哈希函数
    ///- @[in]: 一个或多个哈希对象的有序集合，
    ///效果相当于把其中的所有元素按序串连起来之后，统一取哈希
    fn hashx(source: &[&Self]) -> Vec<u8>;
    fn hashx_len() -> usize;
}

impl<T: Hash + Default> HashX for T {
    #[inline(always)]
    fn hashx(source: &[&Self]) -> Vec<u8> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for x in source {
            x.hash(&mut hasher);
        }
        hasher.finish().to_ne_bytes().to_vec()
    }

    #[inline(always)]
    fn hashx_len() -> usize {
        Self::hashx(&[&Self::default()]).len()
    }
}

///- @glob_keyset: 全局所有的key统一存放于此，按首字节有序排列
///- @root: root节点的children的排列順序与glob_keyset是完全一致的
pub struct HashMap<K: HashX + Eq + PartialEq + Ord + PartialOrd, V> {
    glob_keyset: Vec<Rc<HashSig>>,
    root: Rc<Node<K, V>>,

    keysiz: usize,
}

struct KV<K: HashX + Eq + PartialEq + Ord + PartialOrd, V> {
    k: K,
    v: V,
}

///- @hashsig_base: 指向当前节点的key在全局KeySet中位置，root结点置为Rc::new(vec![])；
///- @hashsig_idx: 当前节点的key，存储的是索引区间，取值规则是`前后均包含`；root结点置为[0, 0]；
///所有操作都是根结节开始的，其索引的对象是已知的，故无需存储指向索引对象的指针
///- @value: 被索引的最终数据，如某个区块中收录的交易集合等，所有非叶节点都是None
///- @parent: 使用Weak结构，不需要在外面再套一层Option结构，第一层节点全部置为Weak::new()
///- @children: 下层节点的指针集合
#[derive(Debug)]
pub struct Node<K: HashX + Eq + PartialEq + Ord + PartialOrd, V> {
    hashsig_base: Rc<HashSig>,
    hashsig_idx: [usize; 2],

    kv: KV<K, V>,

    parent: Weak<Node>,
    children: Vec<Rc<Node>>,
}

impl HashMap {
    #[inline(always)]
    fn check_keysiz(&self, h: &[u8]) -> bool {
        h.len() == self.keysiz
    }

    pub fn new() -> HashMap {
        HashMap {
            glob_keyset: vec![],
            root: Rc::new(Node::new()),
            keysiz: sha1_hash(&[&1i32.to_be_bytes()[..]]).len(),
        }
    }

    ///#### 查找是否存在某个key对应的value
    ///- #: 返回查找结果的引用
    ///- @key[in]: 查找对象
    pub fn query_value(&self, key: &[u8]) -> Result<Option<Value>, XErr> {
        let n = self.query(key)?;
        Ok(n.value.as_ref().cloned())
    }

    //#### 逐一检索key中的所有字节，直到检索成功或失败
    //- #: 检索成功，返回叶节点信息，
    //否则返回可在之后插入的节点信息，若之后插入该值，则本返回值即为其父节点
    //- @key[in]: 某个value的哈希值
    fn query(&self, key: &[u8]) -> Result<Rc<Node>, XErr> {
        if !self.check_keysiz(key) {
            return Err(XErr::HashLen);
        }

        let mut res = Err(XErr::Unknown);
        Self::query_inner(Rc::clone(&self.root), key, &mut res);
        res
    }

    //#### 逐一检索key中的所有字节，直到检索成功或失败
    //- #: 检索成功，返回叶节点信息，
    //否则返回可在之后插入的节点信息，若之后插入该值，则本返回值即为其父节点
    //- @me[in]: 父节点
    //- @key[in]: 索引对象，即某个value的哈希值
    //- @res[out]: 执行结果写出至此
    fn query_inner(me: Rc<Node>, key: &[u8], res: &mut Result<Rc<Node>, XErr>) {
        let exists = me.children.binary_search_by(|n| {
            n.hashsig_base[n.hashsig_idx[0]..=n.hashsig_idx[1]].cmp(&key[n.hashsig_idx[0]..=n.hashsig_idx[1]])
        });
        match exists {
            Ok(idx) => {
                if me.children[idx].hashsig_idx[1] + 1 == key.len() {
                    //查找成功
                    *res = Ok(Rc::clone(&me.children[idx]));
                } else if me.children[idx].hashsig_idx[1] + 1 < key.len() {
                    //children[idx]的key完全包含在key之中，进入下一层继续查找
                    Self::query_inner(Rc::clone(&me.children[idx]), key, res);
                } else {
                    //在key长度固定的前提下，不可能运行至此
                    unreachable!();
                }
            }
            Err(_) => {
                *res = Err(XErr::NotExists(Rc::clone(&me)));
            }
        }
    }

    ///#### 插入新值
    ///- #: 插入成功(key已存在且value相同的情况也视为成功)返回新节点信息，
    ///失败则返回key重复的已有节点信息，**只有在出现哈希碰撞时才会出现**，此值永远无法原样插入！
    ///- @value: 要插入的新值，对应的key通过对其取哈希得到
    pub fn insert(&mut self, value: Value) -> Result<Rc<Node>, XErr> {
        let key = (self.hash_func)(&[&value]);
        let exists = self.query(&key);
        match exists {
            Ok(n) => {
                if n.hash == key {
                    Ok(n)
                } else {
                    Err(XErr::HashCollision(n))
                }
            }
            Err(e) => {
                match e {
                    XErr::NotExists(n) => {
                        let i = self
                            .glob_keyset
                            .binary_search_by(|h| h[..].cmp(&key[..]))
                            .unwrap_err();

                        let mut newkey = key.to_vec();
                        newkey.shrink_to_fit();
                        self.glob_keyset.insert(i, Rc::new(newkey));

                        let res = self.insert_inner(Rc::clone(&n), &key, value);
                        self.refresh_hash(&res); //逆向重塑哈希
                        Ok(res)
                    }
                    _ => Err(e),
                }
            }
        }
    }

    //#### 插入新元素，self是待插入节点的父节点
    //- 若self自身无父节点，则self就是root节点，将新值直接插入到self的children中即可；
    //- 否则作为其它分支节点的children
    fn insert_inner(&self, me: Rc<Node>, key: &[u8], value: Value) -> Rc<Node> {
        let parent = Weak::upgrade(&me.parent);
        match parent {
            Some(p) => {
                let diff_idx = key
                    .iter()
                    .zip(me.hashsig_base.iter())
                    .skip(me.hashsig_idx[0] + 1)
                    .position(|(k1, k2)| k1 != k2);
                if let Some(i) = diff_idx {
                    let mut branch = Rc::new(Node {
                        hashsig_base: Rc::clone(&me.hashsig_base),
                        hashsig_idx: [me.hashsig_idx[0], me.hashsig_idx[0] + i],
                        value: None,
                        children: Vec::with_capacity(2),
                        parent: Rc::downgrade(&p),
                        hash: vec![], //此处暂时留空，后续操作会刷新此值
                    });

                    let h = (self.hash_func)(&[&value]);
                    let leaf_new = Rc::new(Node {
                        hashsig_base: Rc::clone(&me.hashsig_base),
                        hashsig_idx: [me.hashsig_idx[0] + i + 1, key.len() - 1],
                        value: Some(value),
                        children: Vec::with_capacity(0),
                        parent: Rc::downgrade(&branch),
                        hash: h,
                    });
                    let res = Rc::clone(&leaf_new);

                    let mut leaf_old = Rc::clone(
                        &p.children[p
                            .children
                            .binary_search_by(|c| {
                                c.hashsig_base[c.hashsig_idx[0]].cmp(&me.hashsig_base[me.hashsig_idx[0]])
                            })
                            .unwrap()],
                    );
                    unsafe {
                        let raw = Rc::into_raw(leaf_old) as *mut Node;
                        (*raw).hashsig_idx = [me.hashsig_idx[0] + i + 1, me.hashsig_idx[1]];
                        (*raw).parent = Rc::downgrade(&branch);
                        leaf_old = Rc::from_raw(raw);
                    }

                    unsafe {
                        let raw = Rc::into_raw(branch) as *mut Node;
                        (*raw).children.push(leaf_new);
                        (*raw).children.push(leaf_old);
                        (*raw)
                            .children
                            .sort_by(|a, b| a.hashsig_base[a.hashsig_idx[0]].cmp(&b.hashsig_base[b.hashsig_idx[0]]));
                        branch = Rc::from_raw(raw);
                    }

                    let idx = p
                        .children
                        .binary_search_by(|n| {
                            n.hashsig_base[n.hashsig_idx[0]].cmp(&branch.hashsig_base[branch.hashsig_idx[0]])
                        })
                        .unwrap();

                    unsafe {
                        let raw = Rc::into_raw(p) as *mut Node;
                        (*raw).children[idx] = branch;
                        Rc::from_raw(raw);
                    }

                    return res;
                } else {
                    //其它可能性，已事先通过query过滤掉
                    unreachable!();
                }
            }
            None => {
                let idx = self
                    .glob_keyset
                    .binary_search_by(|h| h[..].cmp(&key[..]))
                    .unwrap(); //调用insert之前，已在KeySet中插入key

                let h = (self.hash_func)(&[&value]);
                let leaf_new = Rc::new(Node {
                    hashsig_base: Rc::clone(&self.glob_keyset[idx]),
                    hashsig_idx: [0, self.keysiz - 1],
                    value: Some(value),
                    children: Vec::with_capacity(0),
                    parent: Rc::downgrade(&me),
                    hash: h,
                });
                let res = Rc::clone(&leaf_new);

                let raw = Rc::into_raw(me) as *mut Node;
                unsafe {
                    (*raw).children.insert(idx, leaf_new);
                    Rc::from_raw(raw);
                }

                res
            }
        }
    }
}

impl Node {
    fn new() -> Node {
        Node {
            hashsig_base: Rc::new(vec![]),
            hashsig_idx: [0; 2],
            value: None,
            hash: vec![],
            parent: Weak::new(),
            children: vec![],
        }
    }
}

///- @HashCollision: 哈希长度不一致
///- @NotExists: 哈希碰撞
#[derive(Debug)]
pub enum XErr {
    HashLen,
    NotExists(Rc<Node>),
    Unknown,
}

impl Display for XErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            XErr::HashLen => write!(f, "Invalid hash length!"),
            XErr::NotExists(_) => write!(f, "Not exists!"),
            XErr::HashCollision(_) => write!(f, "Hash collision!"),
            XErr::Unknown => write!(f, "Unknown error!"),
        }
    }
}

impl Error for XErr {
    fn description(&self) -> &str {
        match self {
            XErr::HashLen => "Invalid hash length!",
            XErr::NotExists(_) => "Not exists!",
            XErr::HashCollision(_) => "Hash collision!",
            XErr::Unknown => "Unknown error!",
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::random;

    #[test]
    fn hashmap() {
    }
}
