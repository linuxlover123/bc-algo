# bc-algo
> 使用 Rust 语言**全新实现**的数据结构与算法集合，    
> 其中的大部分与区块链领域中某个实际应用场景相关；    
> 短期目标是加深自身对**区块链底层设施**的理解，    
> 长期目标是创造出一套先进的**自有区块链解决方案**；    
> 实现中追求**算法层面的极致**效率和优雅实现，    
> 但**不苛求语言层面的优化**（如：基于标准库而不是 core 库等）。    
> 正所谓“光说不练假把式”，看十个别人的算法实现，不如自己亲手实现一个！    

# 文档
```
cargo doc --open
```

# Develop For Production
#### Tree
> - [ ] [patricia trie](src/data_structure/tree/patricia_trie)
> - [ ] [cross list](src/data_structure/tree/cross_list)
> - [ ] [SPT](src/data_structure/tree/spt)(Sharding Patricia Trie)
> - [ ] [SCL](src/data_structure/tree/scl)(Sharding Cross-List:
suit for memory query of big-data(many GBs, etc.), with short key(usually primary-type: i32/usize/u128/...etc.))
> - [ ] [SCLPT](src/data_structure/tree/sclpt)(Sharding Cross-List Patricia-Trie:
suit for memory query of big-data(many GBs, etc.) with long key(string/Vec/Box/...etc.))
> - [x] [MPT](src/data_structure/tree/mpt)(Merkle Patricia Trie)
> - [ ] [MCL](src/data_structure/tree/mcl)(Merkle Cross List)

#### SerDe
> - [ ] [RLP](src/data_structure/tree/rlp)(RLP [de]serialize algorithm used in ethereum)

#### P2P Sharding Algorithms
> - [ ] kademlia
> - [ ] S/kademlia
> - [ ] coral

#### Consensus Algorithms
> - [ ] bitcoin POW
> - [ ] ethereum POW
> - [ ] raft

# Only For Exercise
#### Bloom Filter
> - [x] [origin](src/data_structure/bloomfilter/origin.rs)
> - [x] [partial](src/data_structure/bloomfilter/partial.rs)

#### Graph
> - [ ] DAG

#### Tree
> - [x] [trie](src/draft_for_exercise/tree/trie.rs)
> - [x] [patricia trie](src/draft_for_exercise/tree/patricia_trie.rs): [v1, common use], [v2, append only style]
> - [x] [merlke tree](src/draft_for_exercise/tree/merkle.rs): [v1, classic-tree style], [v2, vector-layer style]
> - [x] [huffman tree](src/draft_for_exercise/tree/huffman.rs)(include a complete huffman-serde implement)
> - [ ] B
> - [ ] B+
> - [ ] red-black tree
> - [ ] AVL

#### Linked List
> - [x] [one-way](src/draft_for_exercise/linkedlist/one_way.rs)
> - [x] [two-way](src/draft_for_exercise/linkedlist/two_way.rs)

#### Optimize algorithm using mathematical theorems
> - [ ] 数学定理
> - [ ] 微积分
> - [ ] 概率论与数理统计
> - [ ] 线性代数与线性规划
> - [ ] ...
