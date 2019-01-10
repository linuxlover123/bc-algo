# bc-algo
## data structure and algorithms for blockchain    
    
> 使用 Rust 语言**全新实现**的数据结构与算法集合，    
> 其中的大部分与区块链领域中某个实际应用场景相关；    
> 短期目标是加深自身对**区块链底层设施**的理解，    
> 长期目标是创造出一套先进的**自有区块链解决方案**；    
> 实现中追求**算法层面的极致**效率和优雅实现，    
> 但**不苛求语言层面的优化**（如：基于标准库而不是 core 库，不使用 unsafe 代码等）。    
> 正所谓“光说不练假把式”，看十个别人的算法实现，不如自己亲手实现一个！    

## 配套文档
```
cargo doc --open
```

## 完成进度
#### linked list
- [x] one-way linked list
- [x] two-way linked list

#### tree
- [ ] AVL tree
- [ ] red-black tree
- [ ] B tree
- [ ] B+ tree
- [ ] merkle tree
- [ ] sorted merlke tree
- [ ] trie
- [ ] patricia trie
- [ ] MPT

#### graph
- [ ] DAG
- [ ] skip list
- [ ] base graph
- [ ] base net
- [ ] complex net

#### bloom filter
- [ ] single hash bloom filter
- [ ] multi hash bloom filter

#### serialize and deserialize
- [ ] ethereum RLP

#### P2P routing algorithms
- [ ] kademlia
- [ ] S/kademlia
- [ ] coral

#### consensus algorithms
- [ ] bitcoin POW
- [ ] ethereum POW
- [ ] raft
