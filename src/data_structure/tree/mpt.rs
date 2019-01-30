//! ## Merkle Patricia Trie
//!
//! #### 算法说明
//! - 邻接节点之间具有哈希关系的压缩前缀搜索树;
//! - 由于每个节点的value长度相同(hash)，当用于区块链等只增不删的场景时，
//! 可进行特化实现，实体数据统一存入在顶层，下层各节点只存储对应的索引区间。
//!
//! #### 应用场景
//! - 存在性证明，数据检索。
//!
//! #### 实现属性
//! - <font color=Red>×</font> 多线程安全
//! - <font color=Red>×</font> 无 unsafe 代码

use std::rc::Rc;

pub trait TrieKey: Clone + Eq + Ord + PartialEq + PartialOrd {}

#[derive(Default)]
pub struct Trie<K, V>
where
    K: TrieKey,
    V: Clone,
{
    keys: Vec<Rc<Vec<K>>>,
    nodes: Vec<*mut Node<K, V>>,
}

pub struct Node<K, V>
where
    K: TrieKey,
    V: Clone,
{
    key: KeyIdx<K>,
    value: Option<V>,
    nodes: Vec<*mut Node<K, V>>,
}

struct KeyIdx<K>
where
    K: TrieKey,
{
    base: Rc<Vec<K>>,
    section: [usize; 2], //前后均包含
}

macro_rules! key {
    ($node: expr) => {
        $node.key.base[$node.key.section[0]..=$node.key.section[1]]
    };
}
macro_rules! gen_key {
    ($base: expr, $start: expr, $end: expr) => {
        KeyIdx {
            base: Rc::clone(&$base),
            section: [$start, $end],
        }
    };
}
macro_rules! gen_key_from {
    ($key: expr, $offset_start: expr, $offset_end: expr) => {
        KeyIdx {
            base: Rc::clone(&$key.base),
            section: [
                $key.section[0] + $offset_start,
                $key.section[1] + $offset_end,
            ],
        }
    };
}

impl<K, V> Trie<K, V>
where
    K: TrieKey,
    V: Clone,
{
    pub fn new() -> Trie<K, V> {
        Trie {
            keys: vec![],
            nodes: vec![],
        }
    }

    ///#### 插入元素
    ///- 若key为空，返回错误
    ///- 若key已存在，返回错误
    ///```norun
    ///let t = Trie::new();
    ///t.insert(&[8], "abc").unwrap();
    ///```
    pub fn insert(&mut self, key: &[K], value: V) -> Result<(), ()> {
        if key.is_empty() {
            return Err(());
        }

        let base;
        if let Err(i) = self
            .keys
            .binary_search_by(|item| (**item).as_slice().cmp(&key))
        {
            self.keys.insert(i, Rc::new(key.to_vec()));
            base = Rc::clone(&self.keys[i]);
        } else {
            return Err(());
        }

        let mut nodes = &mut self.nodes;
        let mut idx_key = 0;

        //在已有路径上匹配
        while idx_key < base.len() {
            unsafe {
                match nodes.binary_search_by(|&item| key!(*item)[0].cmp(&base[idx_key])) {
                    Ok(i) => {
                        if let Some(j) = base
                            .iter()
                            .skip(idx_key)
                            .zip(key!(*nodes[i]).iter())
                            .skip(1)
                            .position(|(k1, k2)| k1 != k2)
                        //key[idx_key..]与nodes[i].key之间存在差异项
                        {
                            let keep = nodes[i];
                            nodes[i] = Box::into_raw(Box::new(Node {
                                key: gen_key!(
                                    (*keep).key.base,
                                    (*keep).key.section[0],
                                    (*keep).key.section[0] + j
                                ),
                                value: None,
                                nodes: Vec::with_capacity(2),
                            }));

                            (*keep).key = gen_key_from!((*keep).key, 1 + j, 0);

                            (*nodes[i]).nodes.push(keep);
                            (*nodes[i]).nodes.push(Box::into_raw(Box::new(Node {
                                key: gen_key!(base, idx_key + 1 + j, base.len() - 1),
                                value: Some(value),
                                nodes: Vec::with_capacity(0),
                            })));

                            (*nodes[i]).nodes.sort_by(|&a, &b| key!(*a).cmp(&key!(*b)));

                            return Ok(());
                        //key[idx_key..] == nodes[i].key
                        } else if idx_key + key!(*nodes[i]).len() == base.len() {
                            //若值为空则插入，否则返回错误
                            if (*nodes[i]).value.is_none() {
                                (*nodes[i]).value = Some(value);
                                return Ok(());
                            } else {
                                return Err(());
                            }
                        //key[idx_key..]完全包含在nodes[i].key之中
                        } else if idx_key + key!(*nodes[i]).len() > base.len() {
                            let keep = nodes[i];
                            nodes[i] = Box::into_raw(Box::new(Node {
                                key: gen_key!(base, idx_key, base.len() - 1),
                                value: Some(value),
                                nodes: Vec::with_capacity(1),
                            }));

                            (*keep).key = gen_key_from!((*keep).key, 1 + idx_key, 0);
                            (*nodes[i]).nodes.push(keep);

                            return Ok(());
                        } else {
                            //nodes[i].key完全包含在key[idx_key..]之中，进入下一层继续查找
                            idx_key += key!(*nodes[i]).len();
                            nodes = &mut (*nodes[i]).nodes;
                        }
                    }
                    //查找失败，直接添加新节点
                    Err(i) => {
                        nodes.insert(
                            i,
                            Box::into_raw(Box::new(Node {
                                key: gen_key!(base, idx_key, base.len() - 1),
                                value: Some(value),
                                nodes: Vec::with_capacity(0),
                            })),
                        );
                        return Ok(());
                    }
                };
            }
        }

        unreachable!();
    }

    fn query(&self, key: &[K]) -> &Option<V> {
        if key.is_empty() || self.exists(key).is_err() {
            return &None;
        }

        let mut nodes = &self.nodes;
        let mut idx_key = 0;

        while idx_key < key.len() {
            unsafe {
                match nodes.binary_search_by(|&item| key!(*item)[0].cmp(&key[idx_key])) {
                    Ok(i) => {
                        if key
                            .iter()
                            .skip(idx_key)
                            .zip(key!(*nodes[i]).iter())
                            .skip(1)
                            .any(|(k1, k2)| k1 != k2)
                        //key[idx_key..]与nodes[i].key之间存在差异项
                        //则证明查找对象不存在
                        {
                            return &None;
                        //key[idx_key..]包含在nodes[i].key中
                        //证明查找成功
                        } else if idx_key + key!(*nodes[i]).len() >= key.len() {
                            if (*nodes[i]).value.is_none() {
                                return &None;
                            } else {
                                return &(*nodes[i]).value;
                            }
                        } else {
                            //nodes[i].key完全包含在key[idx_key..]之中，进入下一层继续查找
                            idx_key += key!(*nodes[i]).len();
                            nodes = &(*nodes[i]).nodes;
                        }
                    }
                    //查找失败，返回错误
                    Err(_) => {
                        return &None;
                    }
                };
            }
        }

        unreachable!();
    }

    pub fn exists(&self, key: &[K]) -> Result<(), ()> {
        if self
            .keys
            .binary_search_by(|item| (**item).as_slice().cmp(&key))
            .is_ok()
        {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::random;

    impl TrieKey for u8 {}
    impl TrieKey for u128 {}

    const N: usize = 1117;

    #[test]
    fn mpt() {
        let mut sample = vec![];
        let mut trie = Trie::new();

        (0..N).for_each(|_| sample.push(random::<u128>()));
        for v in sample.iter().cloned() {
            trie.insert(&v.to_be_bytes(), v).unwrap();
        }

        assert_eq!(N, trie.keys.len());
        assert!(0 < trie.nodes.len());
        assert!(trie.nodes.len() <= trie.keys.len());

        for v in sample.iter().cloned() {
            assert!(trie.exists(&v.to_be_bytes()).is_ok());
        }

        for v in sample.iter().cloned() {
            assert_eq!(v, trie.query(&v.to_be_bytes()).unwrap());
        }
    }
}
