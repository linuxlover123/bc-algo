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
//!
//! #### Example
//!```
//!    use bc_algo::mpt::*;
//!    use rand::random;
//!
//!    fn main() {
//!        let mut sample = vec![];
//!        let mut hashsigs = vec![];
//!        let mut mpt = MPT::default();
//!
//!        (0..1117).for_each(|_| sample.push(random::<u128>()));
//!        sample.sort();
//!        sample.dedup();
//!
//!        for v in sample.iter().cloned() {
//!            hashsigs.push(mpt.put(v).unwrap());
//!        }
//!
//!        assert_eq!(sample.len(), mpt.glob_keyset_len());
//!
//!        assert!(0 < mpt.root_children_len());
//!        assert!(mpt.root_children_len() <= mpt.glob_keyset_len());
//!
//!        assert!(!mpt.root_hashsig().is_empty());
//!        for (v, h) in sample.iter().zip(hashsigs.iter()) {
//!            assert_eq!(v, &mpt.get(h).unwrap());
//!            assert!(mpt.proof(h).unwrap());
//!        }
//!    }
//!```

pub mod error;
pub mod traits;

use error::*;
use std::rc::{Rc, Weak};
use traits::*;

type HashSig = Box<[u8]>;
type HashFunc = Box<dyn Fn(&[&[u8]]) -> Box<[u8]>>;

//- @glob_keyset: 全局所有的key统一存放于此，按首字节有序排列
//- @root: root节点的children的排列順序与glob_keyset是完全一致的
//- @hashsig_len: 哈希值的字节长度
//- @hash: 哈希函数指针
pub struct MPT<V: AsBytes> {
    glob_keyset: Vec<Rc<HashSig>>,
    root: Rc<Node<V>>,

    hashsig_len: usize,
    hash: HashFunc,
}

//- @keybase: 指向当前节点的key在全局KeySet中位置，root结点置为Rc::new(vec![])；
//- @keyidx: 当前节点的key，存储的是索引区间，取值规则是`前后均包含`；root结点置为[0, 0]；
//所有操作都是根结节开始的，其索引的对象是已知的，故无需存储指向索引对象的指针
//- @value: 被索引的最终数据，如某个区块中收录的交易集合等，所有非叶节点都是None
//- @hash: 叶节点对value取哈希，分支节点首先将所有children的哈希按序串连起来，然后取其哈希
//- @parent: 使用Weak结构，不需要在外面再套一层Option结构，第一层节点全部置为Weak::new()
//- @children: 下层节点的指针集合
#[derive(Debug)]
pub struct Node<V: AsBytes> {
    keybase: Rc<HashSig>,
    keyidx: [usize; 2],

    value: Option<V>,
    hashsig: HashSig,

    parent: Weak<Node<V>>,
    children: Vec<Rc<Node<V>>>,
}

//- @selfidx: 路径上的每个节点在所有兄弟节点中的索引
//- @hashsigs: 当前节点及其所有兄弟节点的哈希值的有序集合
pub struct ProofPath {
    selfidx: usize,
    hashsigs: Vec<HashSig>,
}

#[inline(always)]
fn sha256(item: &[&[u8]]) -> Box<[u8]> {
    use ring::digest::{Context, SHA256};

    let mut context = Context::new(&SHA256);
    for x in item {
        context.update(x);
    }
    context
        .finish()
        .as_ref()
        .iter()
        .cloned()
        .collect::<Box<[u8]>>()
}

impl<V: AsBytes> MPT<V> {
    ///#### 使用预置哈希函数被始化一个MPT实例
    pub fn default() -> MPT<V> {
        MPT {
            glob_keyset: vec![],
            root: Rc::new(Node::new()),
            hashsig_len: sha256(&[&1i32.to_be_bytes()[..]]).len(),
            hash: Box::new(sha256),
        }
    }

    ///#### 使用自定义哈希函数被始化一个MPT实例
    pub fn new(hash: HashFunc) -> MPT<V> {
        MPT {
            glob_keyset: vec![],
            root: Rc::new(Node::new()),
            hashsig_len: hash(&[&1i32.to_be_bytes()[..]]).len(),
            hash,
        }
    }

    //#### 检查输入的hashsig长度是否合法
    #[inline(always)]
    fn check_hashsig_len(&self, h: &[u8]) -> bool {
        h.len() == self.hashsig_len
    }

    ///- #: 全局根哈希
    #[inline(always)]
    pub fn root_hashsig(&self) -> &[u8] {
        &self.root.hashsig
    }

    ///- #: root节点的children数量
    #[inline(always)]
    pub fn root_children_len(&self) -> usize {
        self.root.children.len()
    }

    ///- #: 返回全局所有key的数量
    #[inline(always)]
    pub fn glob_keyset_len(&self) -> usize {
        self.glob_keyset.len()
    }

    ///#### 查找是否存在某个key对应的value
    ///- #: 返回查找结果的引用
    ///- @key[in]: 查找对象
    #[inline(always)]
    pub fn get(&self, key: &[u8]) -> Option<V> {
        match self.query(key) {
            Ok(n) => n.value.clone(),
            Err(_) => None,
        }
    }

    //#### 逐一检索key中的所有字节，直到检索成功或失败
    //- #: 检索成功，返回叶节点信息，
    //否则返回可在之后插入的节点信息，若之后插入该值，则本返回值即为其父节点
    //- @key[in]: 某个value的哈希值
    fn query(&self, key: &[u8]) -> Result<Rc<Node<V>>, XErr<V>> {
        if !self.check_hashsig_len(key) {
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
    fn query_inner(me: Rc<Node<V>>, key: &[u8], res: &mut Result<Rc<Node<V>>, XErr<V>>) {
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
    ///- @key[in]: 查找对象
    pub fn proof(&self, key: &[u8]) -> Result<bool, XErr<V>> {
        let path = self.get_proof_path(key)?;
        for (i, _) in path.iter().enumerate().rev().skip(1).rev() {
            if (self.hash)(
                &path[i]
                    .hashsigs
                    .iter()
                    .map(|h| &h[..])
                    .collect::<Vec<&[u8]>>()
                    .as_slice(),
            ) != path[i + 1].hashsigs[path[i + 1].selfidx]
            {
                return Ok(false);
            }
        }

        let res = path
            .last()
            .map(|p| {
                (self.hash)(
                    p.hashsigs
                        .iter()
                        .map(|h| &h[..])
                        .collect::<Vec<&[u8]>>()
                        .as_slice(),
                )
            })
            .unwrap_or_else(|| Box::new([]));

        Ok(self.root.hashsig == res)
    }

    //#### 获取指定的key存在的merkle路径证明
    //- #: 按从叶到根的順序排列的哈希集合，使用proof函数验证
    //- @key[in]: 查找对象
    fn get_proof_path(&self, key: &[u8]) -> Result<Vec<ProofPath>, XErr<V>> {
        let n = self.query(key)?;
        let mut path = vec![];
        n.get_proof_path(&mut path);
        Ok(path)
    }

    ///#### 插入新值
    ///- #: 插入成功(key已存在且value相同的情况也视为成功)返回value的哈希值(即：key)，
    ///失败则返回key重复的已有节点信息，**只有在出现哈希碰撞时才会出现**，此值永远无法原样插入！
    ///- @value: 要插入的新值，对应的key通过对其取哈希得到
    #[inline(always)]
    pub fn put(&mut self, value: V) -> Result<HashSig, XErr<V>> {
        self.insert(value).map(|i| i.hashsig.clone())
    }

    //#### 插入新值
    //- #: 插入成功(key已存在且value相同的情况也视为成功)返回新节点信息，
    //失败则返回key重复的已有节点信息，**只有在出现哈希碰撞时才会出现**，此值永远无法原样插入！
    //- @value: 要插入的新值，对应的key通过对其取哈希得到
    fn insert(&mut self, value: V) -> Result<Rc<Node<V>>, XErr<V>> {
        let key = (self.hash)(&[&value.as_bytes()[..]]);
        let exists = self.query(&key);
        match exists {
            Ok(n) => {
                if n.hashsig == key {
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

                        let key = Rc::new(key);
                        self.glob_keyset.insert(i, Rc::clone(&key));

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
    //- @key[in]: 查找对象
    //- @value: 要插入的新值，对应的key通过对其取哈希得到
    fn insert_inner(&self, me: Rc<Node<V>>, key: &[u8], value: V) -> Rc<Node<V>> {
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
                        hashsig: Box::new([]), //此处暂时留空，后续操作会刷新此值
                    });

                    let h = (self.hash)(&[&value.as_bytes()[..]]);
                    let leaf_new = Rc::new(Node {
                        keybase: Rc::clone(&me.keybase),
                        keyidx: [me.keyidx[0] + i + 1, key.len() - 1],
                        value: Some(value),
                        children: Vec::with_capacity(0),
                        parent: Rc::downgrade(&branch),
                        hashsig: h,
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
                        let raw = Rc::into_raw(leaf_old) as *mut Node<V>;
                        (*raw).keyidx = [me.keyidx[0] + i + 1, me.keyidx[1]];
                        (*raw).parent = Rc::downgrade(&branch);
                        leaf_old = Rc::from_raw(raw);
                    }

                    unsafe {
                        let raw = Rc::into_raw(branch) as *mut Node<V>;
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
                        let raw = Rc::into_raw(p) as *mut Node<V>;
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

                let h = (self.hash)(&[&value.as_bytes()[..]]);
                let leaf_new = Rc::new(Node {
                    keybase: Rc::clone(&self.glob_keyset[idx]),
                    keyidx: [0, self.hashsig_len - 1],
                    value: Some(value),
                    children: Vec::with_capacity(0),
                    parent: Rc::downgrade(&me),
                    hashsig: h,
                });
                let res = Rc::clone(&leaf_new);

                let raw = Rc::into_raw(me) as *mut Node<V>;
                unsafe {
                    (*raw).children.insert(idx, leaf_new);
                    Rc::from_raw(raw);
                }

                res
            }
        }
    }

    //#### 插入新值后，递归向上刷新父节点的哈希
    //- @leaf[in]: put()之后产生的新节点
    fn refresh_hash(&self, leaf: &Node<V>) {
        if let Some(mut p) = Weak::upgrade(&leaf.parent) {
            unsafe {
                let raw = Rc::into_raw(p) as *mut Node<V>;
                (*raw).hashsig = (self.hash)(
                    &(*raw)
                        .children
                        .iter()
                        .map(|node| &node.hashsig[..])
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

impl<V: AsBytes> Node<V> {
    fn new() -> Node<V> {
        Node {
            keybase: Rc::new(Box::new([])),
            keyidx: [0; 2],
            value: None,
            hashsig: Box::new([]),
            parent: Weak::new(),
            children: vec![],
        }
    }

    //#### should be a tail-recursion
    //- @path[out]: 从叶到根的順序写出结果
    fn get_proof_path(&self, path: &mut Vec<ProofPath>) {
        if let Some(p) = Weak::upgrade(&self.parent) {
            let cur = ProofPath {
                //传至此处的元素一定是存在的
                selfidx: p
                    .children
                    .binary_search_by(|n| n.keybase[n.keyidx[0]].cmp(&self.keybase[self.keyidx[0]]))
                    .unwrap(),
                hashsigs: p
                    .children
                    .iter()
                    .map(|n| n.hashsig.clone())
                    .collect::<Vec<HashSig>>(),
            };

            path.push(cur);
            p.get_proof_path(path);
        } else {
            return;
        }
    }
}

#[cfg(test)]
mod test {
    macro_rules! source_type_test {
        ($name: ident, $type: ty) => {
            mod $name {
                use super::super::*;
                use rand::random;

                pub fn rand() -> Vec<impl AsBytes> {
                    let mut sample = vec![];
                    (0..500).for_each(|_| sample.push(random::<$type>()));
                    sample.sort();
                    sample.dedup();
                    sample
                }

                pub fn rand_box() -> Vec<impl AsBytes> {
                    let mut sample = vec![];
                    (0..500).for_each(|_| {
                        sample.push(
                            (0..10)
                                .into_iter()
                                .map(|_| random::<$type>())
                                .collect::<Box<[$type]>>(),
                        )
                    });
                    sample.sort();
                    sample.dedup();
                    sample
                }

                pub fn rand_vec() -> Vec<impl AsBytes> {
                    let mut sample = vec![];
                    (0..500).for_each(|_| {
                        sample.push(
                            (0..10)
                                .into_iter()
                                .map(|_| random::<$type>())
                                .collect::<Vec<$type>>(),
                        )
                    });
                    sample.sort();
                    sample.dedup();
                    sample
                }

                pub fn $name<T: AsBytes>(sample: Vec<T>) {
                    let mut hashsigs = vec![];
                    let mut mpt = MPT::default();

                    for v in sample.iter().cloned() {
                        hashsigs.push(mpt.put(v).unwrap());
                    }

                    assert_eq!(sample.len(), mpt.glob_keyset_len());

                    assert!(0 < mpt.root_children_len());
                    assert!(mpt.root_children_len() <= mpt.glob_keyset_len());

                    assert!(!mpt.root_hashsig().is_empty());
                    for (v, h) in sample.iter().zip(hashsigs.iter()) {
                        assert_eq!(v, &mpt.get(h).unwrap());
                        assert!(mpt.proof(h).unwrap());
                    }
                }
            }

            #[test]
            fn $name() {
                let sample0 = $name::rand();
                let sample1 = $name::rand_box();
                let sample2 = $name::rand_vec();

                $name::$name(sample0);
                $name::$name(sample1);
                $name::$name(sample2);
            }
        };
    }

    source_type_test!(_char, char);
    source_type_test!(_u8, u8);
    source_type_test!(_u16, u16);
    source_type_test!(_u32, u32);
    source_type_test!(_u64, u64);
    source_type_test!(_u128, u128);
    source_type_test!(_usize, usize);
    source_type_test!(_i8, i8);
    source_type_test!(_i16, i16);
    source_type_test!(_i32, i32);
    source_type_test!(_i64, i64);
    source_type_test!(_i128, i128);
    source_type_test!(_isize, isize);
}
