//! ## (Merkle)Skip List
//!
//! #### 算法说明
//! - 读写效率与AVL、红黑树等相当；
//! - 易于理解和实现；
//! - 结构固定，可提供merkle proof。
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
//!```

pub mod error;
pub mod traits;

use error::XErr;
use traits::AsBytes;

use std::rc::{Rc, Weak};

type HashSig = Box<[u8]>;
type HashFunc = Box<dyn Fn(&[&[u8]]) -> HashSig>;

//- @unit_siz: 成员数量超过此值将进行分裂
//- @root: 根节点
pub struct SkipList<V: AsBytes> {
    root: Option<Rc<Node<V>>>,
    unit_siz: usize,

    merklesig_len: usize,
    hash: HashFunc,
}

//- @key: 用于索引的键，由HashFunc(V)得到
//- @value: 被索引的目标
//- @merklesig: 各节点的merkle路径哈希值
//- @lower: 下侧节点(一对一，只存储下层的第一个节点)
//- @upper: 上侧节点(多对一)
//- @left: 左侧节点(一对一)
//- @right: 右侧节点(一对一)
pub struct Node<V: AsBytes> {
    key: Rc<HashSig>,
    value: Rc<V>,
    merklesig: HashSig,
    lower: Option<Rc<Node<V>>>,
    upper: Weak<Node<V>>,
    left: Option<Rc<Node<V>>>,
    right: Weak<Node<V>>,
}

impl<V: AsBytes> SkipList<V> {
    pub fn destroy(self) {}

    ///#### 以默认配置初始化
    pub fn default() -> SkipList<V> {
        SkipList::init(8, Box::new(sha256))
    }

    ///#### 初始化
    ///- @unit_siz[in]: 元素数量超过此值将进行分裂
    pub fn init(unit_siz: usize, hash: HashFunc) -> SkipList<V> {
        SkipList {
            root: None,
            unit_siz,
            merklesig_len: hash(&[&0i32.to_be_bytes()[..]]).len(),
            hash,
        }
    }

    //#### 检查输入的merklesig长度是否合法
    #[inline(always)]
    fn check_merklesig_len(&self, h: &[u8]) -> bool {
        h.len() == self.merklesig_len
    }

    ///- #: 全局根哈希
    #[inline(always)]
    pub fn root_merklesig(&self) -> Option<HashSig> {
        self.root.as_ref().map(|r| r.merklesig.clone())
    }

    //#### 查询数据
    //- #: 成功返回目标节点指针，
    //失败返回错误原因(其中不存在的情况，返回可插入位置的左兄弟指针)
    fn get_inner(&self, key: &[u8]) -> Result<Rc<Node<V>>, XErr<V>> {
        unimplemented!();
    }

    ///#### 查询数据
    pub fn get(&self, key: &[u8]) -> Option<V> {
        self.get_inner(key).map(|n| (*n.value).clone()).ok()
    }

    ///#### 删除数据，并按需调整整体的数据结构
    pub fn remove(&self, key: &[u8]) -> Result<Rc<Node<V>>, XErr<V>> {
        let node = self.get_inner(key)?;
        unimplemented!();
    }

    //#### 创建新节点
    fn new_node(&self, key: HashSig, value: V, merklesig: HashSig) -> Node<V> {
        Node {
            key: Rc::new((self.hash)(&[&value.as_bytes()[..]])),
            value: Rc::new(value),
            merklesig,
            lower: None,
            upper: Weak::new(),
            left: None,
            right: Weak::new(),
        }
    }

    ///#### 插入数据，并按需调整整体的数据结构
    pub fn put(&self, value: V) -> Result<HashSig, XErr<V>> {
        if self.root.is_none() {

        } else {
            let sig = self.hash(&[&value.as_bytes()[..]]);
            match self.get_inner(&sig[..]) {
                Ok(n) => {
                    if n.value == &value {
                        Ok(sig)
                    } else {
                        Err(XErr::HashCollision(n))
                    }
                }
                Err(n) => {
                    //let r = Rc::
                }
            }
        }
    }

    ///####获取merkle proof
    ///- #: 若根哈希值与计算出的根哈希相等，返回true
    ///- @key[in]: 查找对象
    pub fn proof(&self, key: &[u8]) -> Result<bool, XErr<V>> {
        if self.root.is_none() {
            return Ok(false);
        }

        let path = self.get_proof_path(key)?;
        for (i, _) in path.iter().enumerate().rev().skip(1).rev() {
            if (self.hash)(
                &path[i]
                    .merklesigs
                    .iter()
                    .map(|h| &h[..])
                    .collect::<Vec<&[u8]>>()
                    .as_slice(),
            ) != path[i + 1].merklesigs[path[i + 1].selfidx]
            {
                return Ok(false);
            }
        }

        let res = path
            .last()
            .map(|p| {
                (self.hash)(
                    p.merklesigs
                        .iter()
                        .map(|h| &h[..])
                        .collect::<Vec<&[u8]>>()
                        .as_slice(),
                )
            })
            .unwrap_or_else(|| Box::new([]));

        Ok(self.root.as_ref().unwrap().merklesig == res)
    }

    //#### 获取指定的key存在的merkle路径证明
    //- #: 按从叶到根的順序排列的哈希集合，使用proof函数验证
    //- @key[in]: 查找对象
    fn get_proof_path(&self, key: &[u8]) -> Result<Vec<ProofPath>, XErr<V>> {
        let n = self.get_inner(key)?;
        let mut path = vec![];
        self.get_proof_path_r(n, &mut path);
        Ok(path)
    }

    //#### should be a tail-recursion
    //- @cur[in]: 当前节点
    //- @path[out]: 从叶到根的順序写出结果
    #[allow(clippy::while_let_loop)]
    fn get_proof_path_r(&self, cur: Rc<Node<V>>, path: &mut Vec<ProofPath>) {
        if let Some(u) = Weak::upgrade(&cur.upper) {
            let mut header = Rc::clone(&u.lower.as_ref().unwrap());
            let mut tmp;
            let mut sigs = vec![header.merklesig.clone()];

            loop {
                if let Some(n) = Weak::upgrade(&header.right) {
                    if !Rc::ptr_eq(&Weak::upgrade(&n.upper).unwrap(), &u) {
                        break;
                    }
                    tmp = n;
                } else {
                    break;
                }
                sigs.push(tmp.merklesig.clone());
                header = tmp;
            }

            path.push(ProofPath {
                selfidx: sigs.binary_search(&cur.merklesig).unwrap(),
                merklesigs: sigs,
            });

            self.get_proof_path_r(u, path);
        } else {
            return;
        }
    }
}

//- @selfidx: 路径上的每个节点在所有兄弟节点中的索引
//- @merklesigs: 当前节点及其所有兄弟节点的哈希值的有序集合
pub struct ProofPath {
    selfidx: usize,
    merklesigs: Vec<HashSig>,
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
                    let mut sl = SkipList::default();

                    for v in sample.iter().cloned() {
                        hashsigs.push(sl.put(v).unwrap());
                    }

                    assert_eq!(sample.len(), sl.glob_keyset_len());

                    assert!(0 < sl.root_children_len());
                    assert!(sl.root_children_len() <= sl.glob_keyset_len());

                    assert!(!sl.root_hashsig().is_esly());
                    for (v, h) in sample.iter().zip(hashsigs.iter()) {
                        assert_eq!(v, &sl.get(h).unwrap());
                        assert!(sl.proof(h).unwrap());
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
