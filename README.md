# bc-algo
## data structure and algorithms for blockchain    
    
> 使用 Rust 语言**全新实现**的数据结构与算法集合，    
> 其中的大部分与区块链领域中某个实际应用场景相关；    
> 短期目标是加深自身对**区块链底层设施**的理解，    
> 长期目标是创造出一套先进的**自有区块链解决方案**；    
> 实现中追求**算法层面的极致**效率和优雅实现，    
> 但**不苛求语言层面的优化**（如：基于标准库而不是 core 库等）。    
> 正所谓“光说不练假把式”，看十个别人的算法实现，不如自己亲手实现一个！    

## 配套文档
```
cargo doc --open
```

## 完成进度

#### 布隆过滤器
> Bloom Filter
> - [x] bloom filter
> - [x] partial bloom filter

#### 链表
> Linked List
> - [x] one-way linked list
> - [x] two-way linked list

#### 树
> Tree
> - [x] huffman tree(include a complete huffman-serde impl)
> - [x] merlke tree(v1, classic-tree style)
> - [x] merlke tree(v2, vector-layer style)
> - [x] trie
> - [x] patricia trie
> - [x] MPT
> - [ ] B tree
> - [ ] B+ tree
> - [ ] AVL tree
> - [ ] red black tree

#### 图
> Graph
> - [ ] DAG
> - [ ] skip list
> - [ ] base graph
> - [ ] base net
> - [ ] complex net

#### 共识算法
> Consensus Algorithms
> - [ ] bitcoin POW
> - [ ] ethereum POW
> - [ ] raft

#### P2P 路由算法
> P2P Routing Algorithms
> - [ ] kademlia
> - [ ] S/kademlia
> - [ ] coral

#### [反]序列化
> Serialize and Deserialize
> - [ ] ethereum RLP

#### 应用数学知识优化算法实现
> Optimize algorithm using mathematical theorems
> - [ ] 初、高中数学定理...
> - [ ] 微积分
> - [ ] 概率论与数理统计
> - [ ] 线性代数与线性规划
> - [ ] ...
