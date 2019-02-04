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
    children: Vec<Node<K, V>>,
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
        let mut children = &mut self.0;

        //在已有路径上查找
        for i in 0..key.len() {
            match children.binary_search_by(|item| item.key.cmp(&key[i])) {
                Ok(j) => {
                    children = &mut children[j].children;
                }
                Err(mut j) => {
                    //扩展新路径至倒数第二个位置
                    for k in key[i..].iter().take(key.len() - i - 1).cloned() {
                        children.insert(
                            j,
                            Node {
                                key: k,
                                value: None,
                                children: Vec::with_capacity(1),
                            },
                        );
                        children = &mut children[j].children;
                        j = 0;
                    }

                    //末端插入数值
                    children.insert(
                        j,
                        Node {
                            key: key[key.len() - 1].clone(),
                            value: Some(value),
                            children: Vec::with_capacity(0),
                        },
                    );

                    return Ok(());
                }
            };
        }
        unreachable!();
    }

    fn remove(&mut self, key: &[K]) -> Result<(), ()> {
        if key.is_empty() {
            return Err(());
        }

        let mut children = &mut self.0;
        let mut children_prev = children as *mut Vec<Node<K, V>>;

        let mut idx = 0;
        for i in 0..key.len() {
            match children.binary_search_by(|item| item.key.cmp(&key[i])) {
                Ok(j) => {
                    idx = j;
                    children_prev = children as *mut Vec<Node<K, V>>;
                    children = &mut children[j].children;
                }
                Err(_) => {
                    //不存在则返回错误
                    return Err(());
                }
            };
        }

        //TODO: 递归向上回收无值(空白)路径
        unsafe {
            (*children_prev)[idx].value = None;
            if (*children_prev)[idx].children.is_empty() {
                (*children_prev).remove(idx);
            }
        }
        Ok(())
    }

    fn query(&self, key: &[K]) -> &Option<V> {
        if key.is_empty() {
            return &None;
        }

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

        &children_prev[idx_children].value
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
