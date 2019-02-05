//! ## MPT(Merkle Patricia Trie)
//!
//! #### 算法说明
//! - 邻接节点之间具有哈希关系的压缩前缀搜索树;
//! - 由于每个节点的key都是具有相同长度的哈希值，故非叶节点上不会存在value；
//! - 当用于区块链等只增不删的场景时，可进行特化实现，实体数据统一存在顶层，下层各节点只存储对应的索引区间。
//!
//! #### 应用场景
//! - 存在性证明，数据检索。
//!
//! #### 实现属性
//! - <font color=Red>×</font> 多线程安全
//! - <font color=Red>×</font> 无 unsafe 代码

use std::error::Error;
use std::fmt::Display;
use std::rc::{Rc, Weak};

type HashSig = Vec<u8>;
type Value = Vec<u8>;

pub type HashFunc = Box<dyn Fn(&[&[u8]]) -> Vec<u8>>;

///- @glob_keyset: 全局所有的key统一存放于此，按首字节有序排列
///- @root: root节点的children的排列順序与glob_keyset是完全一致的
pub struct MPT {
    glob_keyset: Vec<Rc<HashSig>>,
    root: Rc<Node>,

    hash_len: usize,
    hash_func: HashFunc,
}

///- @keybase: 指向当前节点的key在全局KeySet中位置，root结点置为Rc::new(vec![])；
///- @keyidx: 当前节点的key，存储的是索引区间，取值规则是`前后均包含`；root结点置为[0, 0]；
///所有操作都是根结节开始的，其索引的对象是已知的，故无需存储指向索引对象的指针
///- @value: 被索引的最终数据，如某个区块中收录的交易集合等，所有非叶节点都是None
///- @hash: 叶节点对value取哈希，分支节点首先将所有children的哈希按序串连起来，然后取其哈希
///- @parent: 使用Weak结构，不需要在外面再套一层Option结构，第一层节点全部置为Weak::new()
///- @children: 下层节点的指针集合
#[derive(Debug)]
pub struct Node {
    keybase: Rc<HashSig>,
    keyidx: [usize; 2],

    value: Option<Value>,
    hash: HashSig,

    parent: Weak<Node>,
    children: Vec<Rc<Node>>,
}

///- @selfidx: 路径上的每个节点在所有兄弟节点中的索引
///- @hashs: 当前节点及其所有兄弟节点的哈希值的有序集合
pub struct ProofPath {
    selfidx: usize,
    hashs: Vec<HashSig>,
}

#[inline(always)]
fn sha1_hash(item: &[&[u8]]) -> Vec<u8> {
    use ring::digest::{Context, SHA256};

    let mut context = Context::new(&SHA256);
    for x in item {
        context.update(x);
    }
    context.finish().as_ref().to_vec()
}

impl MPT {
    #[inline(always)]
    fn check_hash_len(&self, h: &[u8]) -> bool {
        h.len() == self.hash_len
    }

    pub fn default() -> MPT {
        MPT {
            glob_keyset: vec![],
            root: Rc::new(Node::new()),
            hash_len: sha1_hash(&[&1i32.to_be_bytes()[..]]).len(),
            hash_func: Box::new(sha1_hash),
        }
    }

    pub fn new(hash_func: HashFunc) -> MPT {
        MPT {
            glob_keyset: vec![],
            root: Rc::new(Node::new()),
            hash_len: hash_func(&[&1i32.to_be_bytes()[..]]).len(),
            hash_func,
        }
    }

    pub fn hash(&self) -> &HashFunc {
        &self.hash_func
    }

    ///#### 查找是否存在某个key对应的value
    ///- #: 返回查找结果的引用
    ///- @key[in]: 查找对象
    #[inline(always)]
    pub fn get(&self, key: &[u8]) -> Result<Option<Value>, XErr> {
        let n = self.query(key)?;
        Ok(n.value.as_ref().cloned())
    }

    //#### 逐一检索key中的所有字节，直到检索成功或失败
    //- #: 检索成功，返回叶节点信息，
    //否则返回可在之后插入的节点信息，若之后插入该值，则本返回值即为其父节点
    //- @key[in]: 某个value的哈希值
    fn query(&self, key: &[u8]) -> Result<Rc<Node>, XErr> {
        if !self.check_hash_len(key) {
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
            n.keybase[n.keyidx[0]..=n.keyidx[1]].cmp(&key[n.keyidx[0]..=n.keyidx[1]])
        });
        match exists {
            Ok(idx) => {
                if me.children[idx].keyidx[1] + 1 == key.len() {
                    //查找成功
                    *res = Ok(Rc::clone(&me.children[idx]));
                } else if me.children[idx].keyidx[1] + 1 < key.len() {
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

    ///####获取merkle proof
    ///- #: 计算出的根哈希
    ///- @path[in]: 通过get_proof_path函数得到的merkle路径
    pub fn proof(&self, key: &[u8]) -> Result<bool, XErr> {
        let path = self.get_proof_path(key)?;
        for (i, _) in path.iter().enumerate().rev().skip(1).rev() {
            if (self.hash_func)(
                &path[i]
                    .hashs
                    .iter()
                    .map(|h| h.as_slice())
                    .collect::<Vec<&[u8]>>()
                    .as_slice(),
            ) != path[i + 1].hashs[path[i + 1].selfidx]
            {
                return Ok(false);
            }
        }

        let res = path
            .last()
            .map(|p| {
                (self.hash_func)(
                    p.hashs
                        .iter()
                        .map(|h| h.as_slice())
                        .collect::<Vec<&[u8]>>()
                        .as_slice(),
                )
            })
            .unwrap_or_else(|| vec![]);

        Ok(self.root.hash == res)
    }

    //#### 获取指定的key存在的merkle路径证明
    //- #: 按从叶到根的順序排列的哈希集合，使用proof函数验证
    //- @key[in]: 查找对象
    fn get_proof_path(&self, key: &[u8]) -> Result<Vec<ProofPath>, XErr> {
        let n = self.query(key)?;
        let mut path = vec![];
        n.get_proof_path(&mut path);
        Ok(path)
    }

    ///#### 同insert
    #[inline(always)]
    pub fn set(&mut self, value: Value) -> Result<Rc<Node>, XErr> {
        self.insert(value)
    }

    ///#### 插入新值
    ///- #: 插入成功(key已存在且value相同的情况也视为成功)返回新节点信息，
    ///失败则返回key重复的已有节点信息，**只有在出现哈希碰撞时才会出现**，此值永远无法原样插入！
    ///- @value: 要插入的新值，对应的key通过对其取哈希得到
    fn insert(&mut self, value: Value) -> Result<Rc<Node>, XErr> {
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
                    .zip(me.keybase.iter())
                    .skip(me.keyidx[0] + 1)
                    .position(|(k1, k2)| k1 != k2);
                if let Some(i) = diff_idx {
                    let mut branch = Rc::new(Node {
                        keybase: Rc::clone(&me.keybase),
                        keyidx: [me.keyidx[0], me.keyidx[0] + i],
                        value: None,
                        children: Vec::with_capacity(2),
                        parent: Rc::downgrade(&p),
                        hash: vec![], //此处暂时留空，后续操作会刷新此值
                    });

                    let h = (self.hash_func)(&[&value]);
                    let leaf_new = Rc::new(Node {
                        keybase: Rc::clone(&me.keybase),
                        keyidx: [me.keyidx[0] + i + 1, key.len() - 1],
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
                                c.keybase[c.keyidx[0]].cmp(&me.keybase[me.keyidx[0]])
                            })
                            .unwrap()],
                    );
                    unsafe {
                        let raw = Rc::into_raw(leaf_old) as *mut Node;
                        (*raw).keyidx = [me.keyidx[0] + i + 1, me.keyidx[1]];
                        (*raw).parent = Rc::downgrade(&branch);
                        leaf_old = Rc::from_raw(raw);
                    }

                    unsafe {
                        let raw = Rc::into_raw(branch) as *mut Node;
                        (*raw).children.push(leaf_new);
                        (*raw).children.push(leaf_old);
                        (*raw)
                            .children
                            .sort_by(|a, b| a.keybase[a.keyidx[0]].cmp(&b.keybase[b.keyidx[0]]));
                        branch = Rc::from_raw(raw);
                    }

                    let idx = p
                        .children
                        .binary_search_by(|n| {
                            n.keybase[n.keyidx[0]].cmp(&branch.keybase[branch.keyidx[0]])
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
                    keybase: Rc::clone(&self.glob_keyset[idx]),
                    keyidx: [0, self.hash_len - 1],
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

    //#### 插入新值后，递归向上刷新父节点的哈希
    fn refresh_hash(&self, leaf: &Node) {
        if let Some(mut p) = Weak::upgrade(&leaf.parent) {
            unsafe {
                let raw = Rc::into_raw(p) as *mut Node;
                (*raw).hash = (self.hash_func)(
                    &(*raw)
                        .children
                        .iter()
                        .map(|node| node.hash.as_slice())
                        .collect::<Vec<&[u8]>>(),
                );
                p = Rc::from_raw(raw);
            }
            self.refresh_hash(&p);
        } else {
            //不存在父节点，说明已递归到root节点
            return;
        }
    }
}

impl Node {
    fn new() -> Node {
        Node {
            keybase: Rc::new(vec![]),
            keyidx: [0; 2],
            value: None,
            hash: vec![],
            parent: Weak::new(),
            children: vec![],
        }
    }

    //#### should be a tail-recursion
    //- @me[in]: current node
    //- @path[out]: 从叶到根的順序写出结果
    fn get_proof_path(&self, path: &mut Vec<ProofPath>) {
        if let Some(p) = Weak::upgrade(&self.parent) {
            let cur = ProofPath {
                //传至此处的元素一定是存在的
                selfidx: p
                    .children
                    .binary_search_by(|n| n.keybase[n.keyidx[0]].cmp(&self.keybase[self.keyidx[0]]))
                    .unwrap(),
                hashs: p
                    .children
                    .iter()
                    .map(|n| n.hash.clone())
                    .collect::<Vec<HashSig>>(),
            };

            path.push(cur);
            p.get_proof_path(path);
        } else {
            return;
        }
    }
}

///- @HashCollision: 哈希长度不一致
///- @NotExists: 哈希碰撞
#[derive(Debug)]
pub enum XErr {
    HashLen,
    NotExists(Rc<Node>),
    HashCollision(Rc<Node>),
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

    const N: usize = 1117;

    #[test]
    fn mpt() {
        let mut sample = vec![];
        let mut mpt = MPT::default();

        (0..N).for_each(|_| sample.push(random::<u128>().to_be_bytes().to_vec()));
        sample.sort();
        sample.dedup();

        for v in sample.iter().cloned() {
            mpt.set(v).unwrap();
        }

        assert_eq!(sample.len(), mpt.glob_keyset.len());

        assert!(0 < mpt.root.children.len());
        assert!(mpt.root.children.len() <= mpt.glob_keyset.len());

        assert!(!mpt.root.hash.is_empty());
        let mut h;
        for v in sample.iter() {
            h = (mpt.hash())(&[v]);
            assert_eq!(v, &mpt.get(&h).unwrap().unwrap());
            assert!(mpt.proof(&h).unwrap());
        }
    }
}
