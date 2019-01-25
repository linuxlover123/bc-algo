//! ## 哈夫曼编码
//!
//! #### 算法说明
//! - 一种前缀树：无共同前缀树。
//!
//! #### 应用场景
//! - 通过将定长编码转换为变长编码的方式，实现数据的无损压缩。
//!
//! #### 实现属性
//! - <font color=Green>√</font> 多线程安全
//! - <font color=Green>√</font> 无 unsafe 代码

use rayon::prelude::*;
use std::sync::Arc;

const BYTE_BITS: usize = 8;
const MB: usize = 1024 * 1024;

//预置bit集合，优化位运算的效率
const BIT_SET: [u8; 8] = [
    0b0000_0001,
    0b0000_0010,
    0b0000_0100,
    0b0000_1000,
    0b0001_0000,
    0b0010_0000,
    0b0100_0000,
    0b1000_0000,
];

///以u8为对象进行编解码的抽象数据类型，适配所有的数据类型
pub struct HuffmanTree {
    left: Option<Arc<HuffmanTree>>,
    right: Option<Arc<HuffmanTree>>,
    data: Option<u8>,
}

//使用线性索引以优化性能，以Byte自身为索引
//解码表的值为每个Byte及对应的权重
type EncodeTable = Vec<Vec<u8>>;
type DecodeTable = Vec<(u8, usize)>;

///解码所需要的信息
pub struct Encoded {
    //encoded data received from some sender[s]
    data: Vec<u8>,
    //the number of bit[s] at the last byte for aligning to BYTE_BITS
    pad_len: usize,
}

pub struct Source<'a> {
    data: Arc<&'a [u8]>,
    //需要处理的数据区域索引范围：[start_idx, end_idx)
    section: [usize; 2],
}

//walk on tree
//- @tree[in]: huffman tree
//- @route[in]: routing path of a leaf node, used for middle cache
//- @entb_orig[out]: middle-result of entb
fn traversal(tree: Arc<HuffmanTree>, route: &mut Vec<u8>, entb_orig: &mut Vec<(Vec<u8>, u8)>) {
    if let Some(v) = tree.data {
        entb_orig.push((route.clone(), v));
        return;
    }
    if let Some(ref node) = tree.left {
        route.push(0);
        traversal(Arc::clone(node), route, entb_orig);
        route.pop();
    }
    if let Some(ref node) = tree.right {
        route.push(1);
        traversal(Arc::clone(node), route, entb_orig);
        route.pop();
    }
}

///gen the HuffmanTree from a decode-table
///- @table[in]: code-table for decompression
#[allow(clippy::ptr_arg)]
pub fn gen_tree(table: &DecodeTable) -> HuffmanTree {
    assert!(2 < table.len());
    let mut root = HuffmanTree {
        left: Some(Arc::new(HuffmanTree {
            left: None,
            right: None,
            data: Some(table[0].0),
        })),
        right: Some(Arc::new(HuffmanTree {
            left: None,
            right: None,
            data: Some(table[1].0),
        })),
        data: None,
    };

    let mut prev_weight = table[0].1 + table[1].1;
    for i in table.iter().skip(2) {
        let leaf = HuffmanTree {
            left: None,
            right: None,
            data: Some(i.0),
        };

        let mut r = HuffmanTree {
            left: None,
            right: None,
            data: None,
        };

        if i.1 <= prev_weight {
            r.left = Some(Arc::new(leaf));
            r.right = Some(Arc::new(root));
        } else {
            r.left = Some(Arc::new(root));
            r.right = Some(Arc::new(leaf));
        }

        root = r;
        prev_weight += i.1;
    }

    root
}

///generate en[de]code-table
///- @data：用于生成(编/解)码表的样本数据集
pub fn gen_table(data: &[u8]) -> (EncodeTable, DecodeTable) {
    const TB_SIZ: usize = 1 + u8::max_value() as usize;
    let mut detb = Vec::with_capacity(TB_SIZ);
    for i in 0..TB_SIZ {
        detb.push((i as u8, 0));
    }
    for i in data {
        detb[*i as usize].1 += 1;
    }

    detb.sort_unstable_by(|a, b| a.1.cmp(&b.1));
    let root = gen_tree(&detb);

    let mut entb_orig = vec![];
    let mut route = vec![];
    traversal(Arc::new(root), &mut route, &mut entb_orig);

    let mut entb = Vec::with_capacity(TB_SIZ);
    entb_orig.sort_unstable_by(|a, b| a.1.cmp(&b.1));
    entb_orig.into_iter().for_each(|(v, _)| entb.push(v));

    (entb, detb)
}

///并行编码函数，大于1MB的数据分片
///- @data[in]: those to be encoded
///- @table[in]: code-table for compression
#[allow(clippy::ptr_arg)]
pub fn encode_batch(data: &[u8], table: &EncodeTable) -> Vec<Encoded> {
    let data = Arc::new(data);
    let table = Arc::new(table);
    if MB < data.len() {
        let mut i = 0;
        let mut d = vec![];
        while i < data.len() {
            d.push(Source {
                data: Arc::clone(&data),
                section: [i, i + MB],
            });
            i += MB;
        }

        d.into_par_iter()
            .map(|part| encode(part, Arc::clone(&table)))
            .collect::<Vec<Encoded>>()
    } else {
        let end = data.len();
        vec![encode(
            Source {
                data,
                section: [0, end],
            },
            table,
        )]
    }
}

///基本的编码函数，数据不分片
///- @data[in]: those to be encoded
///- @table[in]: code-table for compression
#[allow(clippy::ptr_arg)]
pub fn encode(source: Source, table: Arc<&EncodeTable>) -> Encoded {
    //计算编码结果所需空间，超过usize最大值会**panic**
    let data = &source.data[source.section[0]..source.section[1]];
    let mut len = data.iter().map(|i| table[*i as usize].len()).sum();
    let pad_len = (BYTE_BITS - len % BYTE_BITS) % BYTE_BITS;
    len = if 0 == pad_len {
        len / BYTE_BITS
    } else {
        1 + len / BYTE_BITS
    };

    //执行编码
    let mut res = Encoded {
        data: Vec::with_capacity(len),
        pad_len,
    };
    for _ in 0..len {
        res.data.push(0u8);
    }

    let mut byte_idx = 0usize;
    let mut bit_idx = 0usize;
    for i in 0..data.len() {
        for j in table[data[i] as usize].iter() {
            if 1u8 == *j {
                res.data[byte_idx] = set_bit(res.data[byte_idx], bit_idx);
            }
            bit_idx += 1;
            if BYTE_BITS == bit_idx {
                byte_idx += 1;
                bit_idx %= BYTE_BITS;
            }
        }
    }

    res
}

///批量解码
#[allow(clippy::ptr_arg)]
pub fn decode_batch(encoded: &[Encoded], table: &DecodeTable) -> Result<Vec<u8>, ()> {
    let tree = Arc::new(gen_tree(table));
    let mut res = vec![];
    for v in encoded
        .par_iter()
        .map(|part| decode(part, Arc::clone(&tree)))
        .collect::<Vec<Result<Vec<u8>, ()>>>()
    {
        res.append(&mut v?);
    }

    Ok(res)
}

///解码
///- #: 若在末尾pad_len位数据之前出现解码错误，返回Err(())，否则返回Ok(Vec<u8>)
///- @encoded[in]: encoded data and meta
///- @tree[in]: the huffman tree which data has been encoded by
pub fn decode(encoded: &Encoded, tree: Arc<HuffmanTree>) -> Result<Vec<u8>, ()> {
    let mut res = vec![];

    if !encoded.data.is_empty() {
        let mut t = &tree;
        let mut tt;
        let mut byte_idx = 0usize;
        let mut bit_idx = 0usize;
        loop {
            if check_bit(encoded.data[byte_idx], bit_idx) {
                tt = t.right.as_ref();
            } else {
                tt = t.left.as_ref();
            }

            if let Some(node) = tt {
                t = node;
            } else {
                return Err(());
            }

            //no data will exists on root&&non-leaf nodes
            if let Some(v) = t.data {
                res.push(v);
                t = &tree;
            }

            bit_idx += 1;
            if byte_idx == encoded.data.len() - 1 && bit_idx == BYTE_BITS - encoded.pad_len {
                break;
            }
            if BYTE_BITS == bit_idx {
                byte_idx += 1;
                bit_idx %= BYTE_BITS;
            }
        }
    }

    Ok(res)
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
mod tests {
    use super::*;
    use rand::random;
    //use std::time;

    fn worker(base: &[u8], source: &[u8]) -> Vec<u8> {
        let (entb, detb) = gen_table(base);
        let encoded = encode_batch(source, &entb);

        decode_batch(&encoded, &detb).unwrap()
    }

    #[test]
    fn huffman() {
        let base = (0..MB * 10).map(|_| random::<u8>()).collect::<Vec<u8>>();
        let source = &base;
        assert_eq!(source, &worker(&base, &source));

        for _i in 0..100 {
            let source = (0..100 + random::<usize>() % 99)
                .map(|_| random::<u8>())
                .collect::<Vec<u8>>();
            assert_eq!(source, worker(&base, &source));
        }

        let base = r"000000000000000000000000000a01201234012345678956789345678000000000
            ;lkjf;中国lhgqk;z`3`3@#$&^%&*^(*)_*)lqjpogjqpojr[ qpk['gkvlosdnh[2 lll1271>";
        let source = base.as_bytes().to_owned();
        assert_eq!(source, worker(base.as_bytes(), &source));
    }
}
