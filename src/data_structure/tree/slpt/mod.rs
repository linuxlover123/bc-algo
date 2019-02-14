//! ## Skip List Partricia Trie
//!
//! #### 算法说明
//! - 用于解决大型有序数据不适合使用线性结构存储(增删时需要大量移动元素)；
//! - 对外接口表现为Partricia Trie；
//! - 内部每一层数据使用跳表管理；
//! - 跳表的内部实现：每一层使用一把读写锁，支持有限的读写并行；
//! - 综合查询效率高于lg(_2)(^n)。
//!
//! #### 应用场景
//! - 内存中的大型数据检索。
//!
//! #### 实现属性
//! - <font color=Green></font> 多线程安全
//! - <font color=Red>×</font> 无 unsafe 代码

//use crate::partricia_trie;
//use crate::skip_list;

#[cfg(test)]
mod test{
    //TODO
}
