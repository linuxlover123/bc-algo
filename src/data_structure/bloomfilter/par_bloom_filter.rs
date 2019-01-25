//! ## 布隆过滤器
//!
//! #### 算法说明
//! - partial bloom filter
//! - 较原始版本准确率更高，且分区间互相独立，可并发计算哈希，效率也更高；
//! - 用于快速检索大数据集，以允许少量错判为代价，兼顾时间与空间两方面的效率；
//! - 难点在于如何根据具体的场景，选定最优的哈希函数、哈希次数及索引数组的容量；
//! - 合适的索引数组容量计算公式：m = kn/ln2， k指哈希函数数量，n指被索引的数据总量。
//!
//! #### 应用场景
//! - 区块链轻节点校验交易：由于轻节点仅有区块头信息，并无完整的交易数据，故首先要粗略定位至可能含有目标交易的区块，之后只向全节点请求经过布隆过滤器筛选出的一个或多个区块的数据。
//!
//! #### 实现属性
//! - <font color=Green>√</font> 多线程安全
//! - <font color=Green>√</font> 无 unsafe 代码
//!

use ring::digest::{Context, SHA1, SHA256, SHA384};
use std::ops::{Deref, DerefMut};

const USIZE_SIZ: usize = std::mem::size_of::<usize>();
const BYTE_BITS: usize = 8;
const BIT_SET: [u8; BYTE_BITS] = [
    0b0000_0001,
    0b0000_0010,
    0b0000_0100,
    0b0000_1000,
    0b0001_0000,
    0b0010_0000,
    0b0100_0000,
    0b1000_0000,
];

const N: usize = 100_0000;
const K: usize = 3;
const M: usize = (N as f32 / std::f32::consts::LN_2) as usize / BYTE_BITS * BYTE_BITS;
const BLOOM_SIZ: usize = M / BYTE_BITS;

#[derive(Debug, Default)]
pub struct BloomFilter {
    filter: Vec<u8>,
    item_cnt: usize,
    bit_used: usize,
}
pub struct ParBloomFilter(Vec<BloomFilter>);

///元素位置
#[derive(Default)]
pub struct Position {
    byte_idx: usize,
    bit_idx: usize,
}
pub struct ParPosition(Vec<Position>);

impl ParBloomFilter {
    pub fn new() -> ParBloomFilter {
        let mut res = ParBloomFilter(vec![]);
        for _ in 0..K {
            res.push(BloomFilter {
                filter: vec![0; BLOOM_SIZ],
                item_cnt: 0,
                bit_used: 0,
            });
        }

        res
    }

    pub fn clear(&mut self) {
        self.iter_mut().for_each(|me| {
            me.filter = vec![0; BLOOM_SIZ];
            me.item_cnt = 0;
            me.bit_used = 0;
        })
    }

    fn hash(item: &[u8]) -> ParPosition {
        let mut idset = vec![];
        for algo in &[&SHA1, &SHA256, &SHA384] {
            let mut context = Context::new(algo);
            context.update(item);
            let id = context.finish();
            let id = id.as_ref();
            assert!(USIZE_SIZ < id.len());

            let mut buf = [0; USIZE_SIZ];
            id[0..USIZE_SIZ]
                .iter()
                .enumerate()
                .for_each(|(i, v)| buf[i] = *v);
            let id = usize::from_le_bytes(buf) % M;
            idset.push(id);
        }

        let mut res = ParPosition(vec![]);
        for id in idset {
            res.push(Position {
                byte_idx: id / BYTE_BITS,
                bit_idx: id % BYTE_BITS,
            });
        }

        res
    }

    ///- @item[in]: 要添加的元素
    pub fn set(&mut self, item: &[u8]) -> ParPosition {
        let par_p = Self::hash(item);

        self.iter_mut().zip(par_p.iter()).for_each(|(me, p)| {
            if !check_bit(me.filter[p.byte_idx], p.bit_idx) {
                me.bit_used += 1;
            }
            me.item_cnt += 1;
            me.filter[p.byte_idx] = set_bit(me.filter[p.byte_idx], p.bit_idx);
        });

        par_p
    }

    ///- @item[in]: 要查找的元素
    pub fn find(&self, item: &[u8]) -> Option<ParPosition> {
        let par_p = Self::hash(item);
        if self
            .iter()
            .zip(par_p.iter())
            .any(|(me, p)| !check_bit(me.filter[p.byte_idx], p.bit_idx))
        {
            None
        } else {
            Some(par_p)
        }
    }

    pub fn false_positive_cnt(&self) -> Vec<usize> {
        let mut res = vec![];
        self.iter().for_each(|me| {
            res.push(me.item_cnt - me.bit_used);
        });
        res
    }
}

//n取值范围：[0, 7]
//- #: 若第n位bit为1，返回true，否则返回false
#[inline]
fn check_bit(data: u8, n: usize) -> bool {
    0 < data & BIT_SET[n]
}

//cheaper to return a u8 than use a pointer ?
#[inline]
fn set_bit(data: u8, n: usize) -> u8 {
    data | BIT_SET[n]
}

impl Deref for ParBloomFilter {
    type Target = Vec<BloomFilter>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for ParPosition {
    type Target = Vec<Position>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ParBloomFilter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DerefMut for ParPosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::random;

    #[test]
    fn par_bloom_filter() {
        let mut bf = ParBloomFilter::new();
        for _ in 0..N {
            let a = random::<usize>();
            bf.set(&a.to_le_bytes());
        }

        for i in bf.iter() {
            assert_eq!(N, i.item_cnt);
            dbg!(bf.false_positive_cnt());
        }

        bf.clear();

        let item = 1i32.to_le_bytes();
        bf.set(&item);
        assert!(bf.find(&item).is_some());
        for i in 0..K {
            assert_eq!(1, bf[i].item_cnt);
            assert_eq!(bf[i].bit_used, bf[i].item_cnt);
        }
        let item = 2u64.to_le_bytes();
        assert!(bf.find(&item).is_none());
    }
}
