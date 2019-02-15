//! ## Routing Skip List
//!
//! #### 算法说明
//! - suit for memory query of big-data(many GBs, etc.),
//! with short key(usually primary-type: i32/usize/u128/...etc.))
//! - 用于解决大型有序数据不适合使用线性结构存储(增删时需要大量移动元素)；
//! - 首先使用byte_routing()进行路由分区；
//! - 之后使用Partricia Trie在分区内对key进行分段搜索，每一段的所在的数据层，使用跳表管理；
//! - 综合查询效率高于lg(_2)(^n)。
//!
//! #### 应用场景
//! - 内存中的大型数据检索。
//!
//! #### 实现属性
//! - <font color=Green>√</font> 多线程安全
//! - <font color=Red>×</font> 无 unsafe 代码

//use crate::skip_list;

#[cfg(test)]
mod test {
    //TODO
}
