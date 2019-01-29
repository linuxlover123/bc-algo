//! ## Patricia Trie
//!
//! #### 算法说明
//! - 压缩前缀搜索树。
//!
//! #### 应用场景
//! - 数据检索，其结果具有绝对唯一性。
//!
//! #### 实现属性
//! - <font color=Red>×</font> 多线程安全
//! - <font color=Green>√</font> 无 unsafe 代码

//TODO
use std::ops::{Deref, DerefMut};

pub trait TrieKey: Clone + Eq + Ord + PartialEq + PartialOrd {}

#[derive(Default)]
pub struct Trie<K: TrieKey, V>(Vec<Node<K, V>>);

#[derive(Default)]
pub struct Node<K: TrieKey, V> {
    key: Vec<K>,
    value: Option<V>,
    children: Vec<Node<K, V>>,
}

impl<K: TrieKey, V> Trie<K, V> {
    fn new() -> Trie<K, V> {
        Trie(vec![])
    }

    fn insert(&mut self, key: &[K], value: V) -> Result<(), ()> {
        let mut children = &mut self.0;
        let mut idx_children = 0;
        let mut idx_key = 0;

        //在已有路径上查找
        while idx_key < key.len() {
            match children.binary_search_by(|item| item.key[0].cmp(&key[idx_key])) {
                Ok(i) => {
                    if let Some(j) = key
                        .iter()
                        .zip(children.key.iter())
                        .skip(1)
                        .position(|(k1, k2)| k1 != k2)
                    {
                        //key与children[i].key之间存在差异项
                        idx_key += j;
                        let new = Node {
                            key: children[i].key[(1 + j)..].clone(),
                            value: None,
                            children: vec![],
                        };
                        children[i].key.truncate(1 + j);
                    } else {
                        //key完全包含在children[i].key之中

                    }

                    children = &mut children[i].children;
                }
                Err(i) => {
                    idx_children = i;
                    break;
                }
            };

            idx_key += 1;
        }

        //扩展新路径至倒数第二个位置
        for k in key
            .iter()
            .skip(idx_key - 1)
            .take(key.len() - idx_key - 1)
            .cloned()
        {
            children.insert(
                idx_children,
                Node {
                    key: k,
                    value: None,
                    children: Vec::with_capacity(1),
                },
            );
            children = &mut children[idx_children].children;
            idx_children = 0;
        }

        if key.is_empty() {
            Err(())
        } else {
            //末端插入数值
            children.insert(
                idx_children,
                Node {
                    key: key[key.len() - 1].clone(),
                    value: Some(value),
                    children: Vec::with_capacity(0),
                },
            );

            Ok(())
        }
    }

    fn remove(&mut self, key: &[K]) -> Result<(), ()> {
        let mut children = &mut self.0;
        for i in 0..key.len() {
            match children.binary_search_by(|item| item.key.cmp(&key[i])) {
                Ok(j) => {
                    if 1 == children.len() {
                        children.clear();
                        return Ok(());
                    }
                    children = &mut children[j].children;
                }
                Err(_) => {
                    //不存在则返回错误
                    return Err(());
                }
            };
        }

        //只有key为空时才会到达此处
        Err(())
    }

    fn query(&self, key: &[K]) -> &Option<V> {
        let mut children = &self.0;
        let mut children_prev = &self.0;
        let mut idx_children = 0;

        for i in 0..key.len() {
            match children.binary_search_by(|item| item.key.cmp(&key[i])) {
                Ok(j) => {
                    children_prev = children;
                    children = &children[j].children;
                    idx_children = j;
                }
                Err(_) => {
                    return &None;
                }
            };
        }

        if key.is_empty() {
            &None
        } else {
            &children_prev[idx_children].value
        }
    }
}

impl<K: TrieKey, V> Node<K, V> {
    fn new(key: K, value: Option<V>) -> Node<K, V> {
        Node {
            key,
            value,
            children: Vec::with_capacity(0),
        }
    }
}

impl<K: TrieKey, V> Deref for Trie<K, V> {
    type Target = Vec<Node<K, V>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K: TrieKey, V> DerefMut for Trie<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl TrieKey for u8 {}

    #[test]
    fn trie() {
        let mut trie = Trie::new();

        for i in 0u128..100 {
            trie.insert(&i.to_le_bytes(), i).unwrap();
        }

        assert!(0 < trie.len());

        for i in 0u128..100 {
            assert_eq!(i, trie.query(&i.to_le_bytes()).unwrap());
        }

        assert!(trie.remove(&0u128.to_le_bytes()).is_ok());
        assert!(trie.query(&0u128.to_le_bytes()).is_none());
    }
}
