//! ## Trie
//!
//! #### 算法说明
//! - 前缀搜索树。
//!
//! #### 应用场景
//! - 数据检索，其结果具有绝对唯一性。
//!
//! #### 实现属性
//! - <font color=Red>×</font> 多线程安全
//! - <font color=Red>×</font> 无 unsafe 代码

use std::ops::{Deref, DerefMut};

pub trait TrieKey: Clone + Eq + Ord + PartialEq + PartialOrd {}

#[derive(Default)]
pub struct Trie<K: TrieKey, V>(Vec<Node<K, V>>);

#[derive(Default)]
pub struct Node<K: TrieKey, V> {
    key: K,
    value: Option<V>,
    nodes: Vec<Node<K, V>>,
}

impl<K: TrieKey, V> Trie<K, V> {
    fn new() -> Trie<K, V> {
        Trie(vec![])
    }

    fn insert(&mut self, key: &[K], value: V) -> Result<(), ()> {
        //key 不能为空
        if key.is_empty() {
            return Err(());
        }
        let mut nodes = &mut self.0;
        let mut idx_nodes = 0;
        let mut idx_key = 0;

        //在已有路径上查找
        for i in 0..key.len() {
            match nodes.binary_search_by(|item| item.key.cmp(&key[i])) {
                Ok(j) => {
                    nodes = &mut nodes[j].nodes;
                    idx_key = i;
                }
                Err(j) => {
                    idx_nodes = j;
                    idx_key = i;
                    break;
                }
            };
        }

        //扩展新路径至倒数第二个位置
        for k in key[idx_key..].iter().take(key.len() - idx_key - 1).cloned() {
            nodes.insert(
                idx_nodes,
                Node {
                    key: k,
                    value: None,
                    nodes: Vec::with_capacity(1),
                },
            );
            nodes = &mut nodes[idx_nodes].nodes;
            idx_nodes = 0;
        }

        //末端插入数值
        nodes.insert(
            idx_nodes,
            Node {
                key: key[key.len() - 1].clone(),
                value: Some(value),
                nodes: Vec::with_capacity(0),
            },
        );

        Ok(())
    }

    fn remove(&mut self, key: &[K]) -> Result<(), ()> {
        if key.is_empty() {
            return Err(());
        }

        let mut nodes = &mut self.0;
        let mut nodes_prev = nodes as *mut Vec<Node<K, V>>;

        let mut idx = 0;
        for i in 0..key.len() {
            match nodes.binary_search_by(|item| item.key.cmp(&key[i])) {
                Ok(j) => {
                    idx = j;
                    nodes_prev = nodes as *mut Vec<Node<K, V>>;
                    nodes = &mut nodes[j].nodes;
                }
                Err(_) => {
                    //不存在则返回错误
                    return Err(());
                }
            };
        }

        //TODO: 递归向上回收无值(空白)路径
        unsafe {
            (*nodes_prev)[idx].value = None;
            if (*nodes_prev)[idx].nodes.is_empty() {
                (*nodes_prev).remove(idx);
            }
        }
        Ok(())
    }

    fn query(&self, key: &[K]) -> &Option<V> {
        if key.is_empty() {
            return &None;
        }

        let mut nodes = &self.0;
        let mut nodes_prev = &self.0;
        let mut idx_nodes = 0;

        for i in 0..key.len() {
            match nodes.binary_search_by(|item| item.key.cmp(&key[i])) {
                Ok(j) => {
                    nodes_prev = nodes;
                    nodes = &nodes[j].nodes;
                    idx_nodes = j;
                }
                Err(_) => {
                    return &None;
                }
            };
        }

        &nodes_prev[idx_nodes].value
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
