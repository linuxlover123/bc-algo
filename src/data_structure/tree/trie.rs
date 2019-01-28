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
//! - <font color=Green>√</font> 无 unsafe 代码

//use std::cmp::{PartialOrd, Ord}

struct Trie<T, V>
where
    T: PartialOrd + Ord,
{
    key: T,
    value: V,
    children: Vec<Trie<T, V>>,
}
