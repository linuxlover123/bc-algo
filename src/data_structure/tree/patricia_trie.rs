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
//! - <font color=Red>×</font> 无 unsafe 代码

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

pub trait TrieKey: Debug + Clone + Eq + Ord + PartialEq + PartialOrd {}

#[derive(Default)]
pub struct Trie<K: TrieKey, V: Clone>(Vec<*mut Node<K, V>>);

pub struct Node<K: TrieKey, V: Clone> {
    key: Vec<K>,
    value: Option<V>,
    children: Vec<*mut Node<K, V>>,
}

impl<K: TrieKey, V: Clone> Trie<K, V> {
    fn new() -> Trie<K, V> {
        Trie(vec![])
    }

    fn insert(&mut self, key: &[K], value: V) -> Result<(), ()> {
        if key.is_empty() {
            return Err(());
        }

        let mut children = &mut self.0;
        let mut idx_children = 0;
        let mut idx_key = 0;

        //在已有路径上匹配
        while idx_key < key.len() {
            unsafe {
                match children.binary_search_by(|&item| (*item).key[0].cmp(&key[idx_key])) {
                    Ok(i) => {
                        if let Some(j) = key
                            .iter()
                            .skip(idx_key)
                            .zip((*children[i]).key.iter())
                            .skip(1)
                            .position(|(k1, k2)| k1 != k2)
                        //key[idx_key..]与children[i].key之间存在差异项
                        {
                            let keep = children[i];
                            children[i] = Box::into_raw(Box::new(Node {
                                key: (*keep).key[..=j].to_vec(),
                                value: None,
                                children: Vec::with_capacity(2),
                            }));

                            (*keep).key.drain(..=j);
                            (*keep).key.shrink_to_fit();

                            (*children[i]).children.push(keep);
                            (*children[i]).children.push(Box::into_raw(Box::new(Node {
                                key: key[idx_key + 1 + j..].to_vec(),
                                value: Some(value),
                                children: Vec::with_capacity(0),
                            })));

                            (*children[i])
                                .children
                                .sort_by(|&a, &b| (*a).key.cmp(&(*b).key));

                            return Ok(());
                        //key[idx_key..] == children[i].key
                        } else if idx_key + (*children[i]).key.len() == key.len() {
                            //若值为空则插入，否则返回错误
                            if (*children[i]).value.is_none() {
                                (*children[i]).value = Some(value);
                                return Ok(());
                            } else {
                                return Err(());
                            }
                        //key[idx_key..]完全包含在children[i].key之中
                        } else if idx_key + (*children[i]).key.len() > key.len() {
                            let keep = children[i];
                            children[i] = Box::into_raw(Box::new(Node {
                                key: key[idx_key..key.len()].to_vec(),
                                value: Some(value),
                                children: Vec::with_capacity(1),
                            }));

                            (*keep).key.drain(..(key.len() - idx_key - 1));
                            (*keep).key.shrink_to_fit();
                            (*children[i]).children.push(keep);

                            return Ok(());
                        } else {
                            //children[i].key完全包含在key[idx_key..]之中，进入下一层继续查找
                            idx_key += (*children[i]).key.len();
                            children = &mut (*children[i]).children;
                        }
                    }
                    //查找失败，跳出循环，直接添加新节点
                    Err(i) => {
                        idx_children = i;
                        break;
                    }
                };
            }
        }

        //无分叉，且需要延伸既有路径时，会运行至此
        //延长路径，插入value
        let mut item = Node {
            key: key[idx_key..].to_vec(),
            value: Some(value),
            children: Vec::with_capacity(0),
        };
        item.key.shrink_to_fit();
        children.insert(idx_children, Box::into_raw(Box::new(item)));

        Ok(())
    }

    fn inner_query(&self, key: &[K]) -> Box<Option<*mut *mut Node<K, V>>> {
        if key.is_empty() {
            return Box::new(None);
        }

        let mut children = &self.0;
        let mut idx_key = 0;

        while idx_key < key.len() {
            unsafe {
                match children.binary_search_by(|&item| (*item).key[0].cmp(&key[idx_key])) {
                    Ok(i) => {
                        if key
                            .iter()
                            .skip(idx_key)
                            .zip((*children[i]).key.iter())
                            .skip(1)
                            .any(|(k1, k2)| k1 != k2)
                        //key[idx_key..]与children[i].key之间存在差异项
                        //则证明查找对象不存在
                        {
                            return Box::new(None);
                        //key[idx_key..]包含在children[i].key中
                        //证明查找成功
                        } else if idx_key + (*children[i]).key.len() >= key.len() {
                            if (*children[i]).value.is_none() {
                                return Box::new(None);
                            } else {
                                return Box::new(Some(Box::into_raw(Box::new(children[i]))));
                            }
                        } else {
                            //children[i].key完全包含在key[idx_key..]之中，进入下一层继续查找
                            idx_key += (*children[i]).key.len();
                            children = &(*children[i]).children;
                        }
                    }
                    //查找失败，返回错误
                    Err(_) => {
                        return Box::new(None);
                    }
                };
            }
        }

        panic!("BUG!")
    }

    fn query(&self, key: &[K]) -> Option<V> {
        unsafe {
            self.inner_query(key)
                .and_then(|node| (**node).value.clone())
        }
    }

    fn replace(&mut self, key: &[K], value: V) -> Result<Option<V>, ()> {
        if let Some(mut v) = *self.inner_query(key) {
            let old;
            unsafe {
                old = (**v).value.clone();
                (**v).value = Some(value);
            }
            Ok(old)
        } else {
            Err(())
        }
    }

    fn remove(&mut self, key: &[K]) -> Result<Option<V>, ()> {
        if let Some(mut v) = *self.inner_query(key) {
            let old;
            unsafe {
                old = (**v).value.clone();
                (**v).value = None;
                //合并路径
                if 1 == (**v).children.len() {
                    let keep = (**v).children.pop().unwrap();
                    (*keep).key = [&(**v).key, &(*keep).key]
                        .iter()
                        .flat_map(|&k| k.clone())
                        .collect::<Vec<K>>();
                    *v = keep;
                }
            }
            Ok(old)
        } else {
            Err(())
        }
    }
}

impl<K: TrieKey, V: Clone> Deref for Trie<K, V> {
    type Target = Vec<*mut Node<K, V>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K: TrieKey, V: Clone> DerefMut for Trie<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::random;

    impl TrieKey for u8 {}
    impl TrieKey for u128 {}

    #[test]
    fn trie() {
        let mut sample = vec![];
        let mut trie = Trie::new();

        (0..1117).for_each(|_| sample.push(random::<u128>()));
        for v in sample.iter().cloned() {
            trie.insert(&v.to_be_bytes(), v).unwrap();
        }

        assert!(0 < trie.len());

        for v in sample.iter().cloned() {
            assert_eq!(v, trie.query(&v.to_be_bytes()).unwrap());
        }

        for v in sample[10..].iter().cloned() {
            assert!(trie.remove(&v.to_be_bytes()).is_ok());
            assert!(trie.query(&v.to_be_bytes()).is_none());
        }

        assert!(trie.replace(&sample[10].to_be_bytes(), 999u128).is_err());
        assert_eq!(
            Some(sample[1]),
            trie.replace(&sample[1].to_be_bytes(), 999u128).unwrap()
        );
        assert_eq!(999u128, trie.query(&sample[1].to_be_bytes()).unwrap());
    }
}
