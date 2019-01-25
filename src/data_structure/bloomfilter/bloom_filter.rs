//! ## 布隆过滤器
//!
//! #### 算法说明
//! - bloom filter
//! - 原版布隆过滤器无法在源数据删除某些元素之后进行对应的更新；
//! - 但对于类似区块链这种只增不删的场景，特别适合；
//! - 用于快速检索大数据集，以允许少量错判为代价，兼顾时间与空间两方面的效率；
//! - 难点在于如何根据具体的场景，选定最优的哈希函数、哈希次数及索引数组的容量；
//! - 合适的索引数组容量计算公式：m = kn / LN_2， k指哈希函数数量，n指被索引的数据总量。
//!
//! #### 应用场景
//! - 区块链轻节点校验交易：由于轻节点仅有区块头信息，并无完整的交易数据，故首先要粗略定位至可能含有目标交易的区块，之后只向全节点请求经过布隆过滤器筛选出的一个或多个区块的数据。
//!
//! #### 实现属性
//! - <font color=Green>√</font> 多线程安全
//! - <font color=Green>√</font> 无 unsafe 代码
//!

use ring::digest::{Context, SHA1};

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
const K: usize = 1;
const M: usize = ((K * N) as f32 / std::f32::consts::LN_2) as usize / BYTE_BITS * BYTE_BITS;
const BLOOM_SIZ: usize = M / BYTE_BITS;

#[derive(Debug, Default)]
pub struct BloomFilter {
    filter: Vec<u8>,
    item_cnt: usize,
    bit_used: usize,
}

///元素位置
#[derive(Default)]
pub struct Position {
    byte_idx: usize,
    bit_idx: usize,
}

impl BloomFilter {
    pub fn new() -> BloomFilter {
        BloomFilter {
            filter: vec![0; BLOOM_SIZ],
            item_cnt: 0,
            bit_used: 0,
        }
    }

    pub fn clear(&mut self) {
        self.filter = vec![0; BLOOM_SIZ];
        self.item_cnt = 0;
        self.bit_used = 0;
    }

    fn hash(item: &[u8]) -> Position {
        let mut context = Context::new(&SHA1);
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

        Position {
            byte_idx: id / BYTE_BITS,
            bit_idx: id % BYTE_BITS,
        }
    }

    ///- @item[in]: 要添加的元素
    pub fn set(&mut self, item: &[u8]) -> Position {
        let p = Self::hash(item);
        if !self.find_by_position(&p) {
            self.bit_used += 1;
        }
        self.item_cnt += 1;
        self.filter[p.byte_idx] = set_bit(self.filter[p.byte_idx], p.bit_idx);
        p
    }

    ///- @item[in]: 要查找的元素
    pub fn find(&self, item: &[u8]) -> Option<Position> {
        let p = Self::hash(item);
        if self.find_by_position(&p) {
            Some(p)
        } else {
            None
        }
    }

    pub fn find_by_position(&self, p: &Position) -> bool {
        check_bit(self.filter[p.byte_idx], p.bit_idx)
    }

    pub fn false_positive_cnt(&self) -> usize {
        self.item_cnt - self.bit_used
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

#[cfg(test)]
mod test {
    use super::*;
    use rand::random;

    #[test]
    fn bloom_filter() {
        let mut bf = BloomFilter::new();
        for _ in 0..N {
            let a = random::<usize>();
            bf.set(&a.to_le_bytes());
        }

        assert_eq!(N, bf.item_cnt);
        dbg!(bf.false_positive_cnt());

        bf.clear();

        let item = 1i32.to_le_bytes();
        bf.set(&item);
        assert!(bf.find(&item).is_some());
        assert_eq!(1, bf.item_cnt);
        assert_eq!(bf.bit_used, bf.item_cnt);
        let item = 2u64.to_le_bytes();
        assert!(bf.find(&item).is_none());
    }
}
