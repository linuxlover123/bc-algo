//! ## 哈夫曼编码
//!
//! #### 算法说明
//! - 一种前缀树。
//!
//! #### 应用场景
//! - 数据压缩。
//!
//! #### 实现属性
//! - <font color=Green>√</font> 多线程安全
//! - <font color=Green>√</font> 无 unsafe 代码

use std::sync::Arc;
use std::sync::RwLock;

const BYTE_BITS: usize = 8;

//> 预置bit集合，优化位运算的效率
const BIT_SET: [u8; 8] = [
    0b00000001, 0b00000010, 0b00000100, 0b00001000, 0b00010000, 0b00100000, 0b01000000, 0b10000000,
];

//> 以u8为对象进行编解码的抽象数据类型，适配所有的数据类型
type HuffmanTree = Node;
struct Node {
    left: Option<Arc<RwLock<Node>>>,
    right: Option<Arc<RwLock<Node>>>,
    data: Option<u8>,
}

//> 编码表使用线性索引以优化性能
type EncodeTable = Vec<Vec<u8>>;
//> 解码表的用途仅是还原huffman tree，无需索引，亦与存储顺序无关
type DecodeTable = Vec<(Vec<u8>, u8)>;

//> 解码需要的信息
struct Encoded {
    //encoded data received from some sender[s]
    data: Vec<u8>,
    //the number of bit[s] at the last byte for aligning to BYTE_BITS
    pad_len: usize,
}

//> walk on tree
//- @tree: huffman tree
//- @route: routing path of a leaf node
//- @detb: decode-table
fn traversal(tree: Arc<RwLock<HuffmanTree>>, route: &mut Vec<u8>, detb: &mut Vec<(Vec<u8>, u8)>) {
    if let Some(v) = tree.read().unwrap().data {
        detb.push((route.clone(), v));
        return;
    }

    let t = tree.read().unwrap();
    if let Some(ref node) = t.left {
        route.push(0);
        traversal(Arc::clone(node), route, detb);
        route.pop();
    }
    if let Some(ref node) = t.right {
        route.push(1);
        traversal(Arc::clone(node), route, detb);
        route.pop();
    }
}

//> generate en[de]code-table
//- @data：用于生成(编/解)码表的样本数据集
fn gen_table(data: &[u8]) -> (EncodeTable, DecodeTable) {
    const TB_SIZ: usize = 1 + u8::max_value() as usize;
    let mut cnter: [(u8, usize); TB_SIZ] = [(0, 0); TB_SIZ];
    for i in 0..TB_SIZ {
        cnter[i].0 = i as u8;
    }
    for i in data {
        cnter[*i as usize].1 += 1;
    }
    cnter.sort_unstable_by(|a, b| a.1.cmp(&b.1));

    assert!(2 < TB_SIZ);
    let mut root = HuffmanTree {
        left: Some(Arc::new(RwLock::new(HuffmanTree {
            left: None,
            right: None,
            data: Some(cnter[0].0),
        }))),
        right: Some(Arc::new(RwLock::new(HuffmanTree {
            left: None,
            right: None,
            data: Some(cnter[1].0),
        }))),
        data: None,
    };

    let mut prev_weight = cnter[0].1 + cnter[1].1;
    for i in 2..cnter.len() {
        let leaf = HuffmanTree {
            left: None,
            right: None,
            data: Some(cnter[i].0),
        };

        let mut r = HuffmanTree {
            left: None,
            right: None,
            data: None,
        };

        if cnter[i].1 <= prev_weight {
            r.left = Some(Arc::new(RwLock::new(leaf)));
            r.right = Some(Arc::new(RwLock::new(root)));
        } else {
            r.left = Some(Arc::new(RwLock::new(root)));
            r.right = Some(Arc::new(RwLock::new(leaf)));
        }

        root = r;
        prev_weight += cnter[i].1;
    }

    let mut detb = vec![];
    let mut route = vec![];
    traversal(Arc::new(RwLock::new(root)), &mut route, &mut detb);

    detb.sort_unstable_by(|a, b| a.1.cmp(&b.1));
    let mut entb = Vec::with_capacity(TB_SIZ);
    for (v, _) in &detb {
        entb.push(v.clone());
    }

    (entb, detb)
}

//> 基本的编码函数——单线程、数据不分片
//- @data[in]: those to be encoded
//- @table[in]: code-table for compression
fn encode(data: &[u8], table: &EncodeTable) -> Encoded {
    //计算编码结果所需空间，超过usize最大值会**panic**
    let mut len = data.iter().map(|i| table[*i as usize].len()).sum();
    let pad_len = (BYTE_BITS - len % BYTE_BITS) % BYTE_BITS;
    len = if 0 == pad_len {
        len / BYTE_BITS
    } else {
        1 + len / BYTE_BITS
    };
    let mut res = Encoded {
        data: Vec::with_capacity(len),
        pad_len: pad_len,
    };
    (0..len).for_each(|_| {
        res.data.push(0u8);
    });

    //编码
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

//> restore the HuffmanTree from a decode-table
//- @table[in]: code-table for decompression
fn restore_tree(table: DecodeTable) -> Arc<RwLock<HuffmanTree>> {
    let root = Some(Arc::new(RwLock::new(HuffmanTree {
        left: None,
        right: None,
        data: None,
    })));

    let mut t = Arc::clone(root.as_ref().unwrap());
    let mut tt;
    for (x, v) in table {
        for i in 0..x.len() {
            {
                tt = Arc::clone(&t);
                let mut lt = tt.write().unwrap(); //locked tree
                if 0 == x[i] {
                    match lt.left {
                        None => {
                            lt.left = Some(Arc::new(RwLock::new(HuffmanTree {
                                left: None,
                                right: None,
                                data: None,
                            })));
                        }
                        _ => {}
                    };
                    t = Arc::clone(lt.left.as_ref().unwrap());
                } else {
                    match lt.right {
                        None => {
                            lt.right = Some(Arc::new(RwLock::new(HuffmanTree {
                                left: None,
                                right: None,
                                data: None,
                            })));
                        }
                        _ => {}
                    }
                    t = Arc::clone(lt.right.as_ref().unwrap());
                }
            }
        }

        t.write().unwrap().data = Some(v);
        t = Arc::clone(root.as_ref().unwrap());
    }

    root.unwrap()
}

//> 首先解码全体数据，之后再将末尾pad_len的数据弹出
//- #: 若在末尾pad_len位数据之前出现解码错误，返回Err(())，否则返回Ok(Vec<u8>)
//- @encoded[in]: encoded data and meta
//- @tree[in]: the huffman tree which data has been encoded by
fn decode(encoded: &Encoded, tree: Arc<RwLock<HuffmanTree>>) -> Result<Vec<u8>, ()> {
    let mut res = vec![];

    if 0 < encoded.data.len() {
        let mut t = Arc::clone(&tree);
        let mut byte_idx = 0usize;
        let mut bit_idx = 0usize;
        loop {
            if check_bit(encoded.data[byte_idx], bit_idx) {
                if let Some(node) = Arc::clone(&t).read().unwrap().right.as_ref() {
                    t = Arc::clone(node);
                } else {
                    return Err(());
                }
            } else {
                if let Some(node) = Arc::clone(&t).read().unwrap().left.as_ref() {
                    t = Arc::clone(node);
                } else {
                    return Err(());
                }
            }

            //no data will exists on root&&non-leaf nodes
            if let Some(v) = Arc::clone(&t).read().unwrap().data {
                res.push(v);
                t = Arc::clone(&tree);
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

//> n取值范围：[0, 7]
//- #: 若第n位bit为1，返回true，否则返回false
#[inline]
fn check_bit(data: u8, n: usize) -> bool {
    if 0 < data & BIT_SET[n] {
        true
    } else {
        false
    }
}

//> It is cheaper to return a u8 than use a pointer
#[inline]
fn set_bit(data: u8, n: usize) -> u8 {
    data | BIT_SET[n]
}

#[cfg(test)]
mod tests {
    use super::*;
    //use rayon::prelude::*;

    fn worker(base: &[u8], source: &[u8]) -> Vec<u8> {
        let (entb, detb) = gen_table(&base);
        let tree = restore_tree(detb);
        let encoded = encode(&source, &entb);

        decode(&encoded, tree).unwrap()
    }

    #[test]
    fn huffman() {
        let base = [
            1u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8,
            2u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 3u8, 3u8,
            3u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 3u8, 3u8, 3u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 2u8, 2u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ];
        let source = [99u8, 1u8];
        assert_eq!(source[..], worker(&base, &source)[..]);

        let base = r"000000000000000000000000000a01201234012345678956789345678000000000
            ;lkjf;中国lhgqk;z`3`3@#$&^%&*^(*)_*)lqjpogjqpojr[ qpk['gkvlosdnh[2 lll1271>";
        let source = base;
        assert_eq!(
            *source.as_bytes(),
            worker(base.as_bytes(), source.as_bytes())[..]
        );
    }
}
