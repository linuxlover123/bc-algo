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

type Byte = u8;
const BYTE_SIZ: usize = std::mem::size_of::<Byte>();

//> 预置bit集合，优化位运算的效率
const BIT_SET: [Byte; 8] = [
    0b00000001, 0b00000010, 0b00000100, 0b00001000, 0b00010000, 0b00100000, 0b01000000, 0b10000000,
];

//> 以Byte为对象进行编解码的抽象数据类型，适配所有的数据类型
type HuffmanTree = Node;
pub struct Node {
    left: Option<Arc<RwLock<Node>>>,
    right: Option<Arc<RwLock<Node>>>,
    data: Option<Byte>,
}

//> 编码表使用线性索引优化性能
type EncodeTable = [Vec<Byte>; 256];
//> 解码表的用途仅是还原huffman tree，无需索引，亦与存储顺序无关
type DecodeTable = Vec<(Vec<Byte>, Byte)>;

//> 解码需要的信息
pub struct Encoded {
    //encoded data received from some sender[s]
    data: Vec<Byte>,
    //the number of bit[s] at the last byte for aligning to BYTE_SIZ
    pad_len: usize,
}

macro_rules! walk_on_tree {
    ($encoded: expr, $t: expr, $byte_idx: expr, $bit_idx: expr) => {
        if check_bit($encoded.data[$byte_idx], $bit_idx) {
            if let Some(node) = Arc::clone(&$t).read().unwrap().right.as_ref() {
                $t = Arc::clone(node);
            } else {
                return Err(());
            }
        } else {
            if let Some(node) = Arc::clone(&$t).read().unwrap().left.as_ref() {
                $t = Arc::clone(node);
            } else {
                return Err(());
            }
        }

        $bit_idx += 1;
        $bit_idx %= BYTE_SIZ;
        if 0 == $bit_idx {
            $byte_idx += 1;
        }
        if $byte_idx >= $encoded.data.len() {
            break;
        }
    };
}

pub trait Huffman {
    //> generate en[de]code-table
    //- @data：用于生成(编/解)码表的样本数据集
    fn gen_table(data: &[Byte]) -> (EncodeTable, DecodeTable) {
        unimplemented!();
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

    //> 基本的编码函数——单线程、数据不分片
    //- @data[in]: those to be encoded
    //- @table[in]: code-table for compression
    fn encode(data: &[Byte], table: &EncodeTable) -> Encoded {
        //计算编码结果所需空间，超过usize最大值会**panic**
        let mut len = data.iter().map(|i| table[*i as usize].len()).sum();
        let pad_len = len % BYTE_SIZ;
        len = if 0 == pad_len { len / 8 } else { 1 + len / 8 };
        let mut res = Encoded {
            data: Vec::with_capacity(len),
            pad_len: pad_len,
        };

        //编码
        let mut byte_idx = 0usize;
        let mut bit_idx = 0usize;
        for i in 0..data.len() {
            for j in table[data[i] as usize].iter() {
                if 1 as Byte == *j {
                    res.data[byte_idx] = set_bit(res.data[byte_idx], bit_idx);
                }
                bit_idx += 1;
                bit_idx %= BYTE_SIZ;
                if 0 == bit_idx {
                    byte_idx += 1;
                }
            }
        }

        res
    }

    //> 首先解码全体数据，之后再将末尾pad_len的数据弹出
    //- #: 若在末尾pad_len位数据之前出现解码错误，返回Err(())，否则返回Ok(Vec<Byte>)
    //- @encoded[in]: encoded data and meta
    //- @tree[in]: the huffman tree which data has been encoded by
    fn decode(encoded: &Encoded, tree: Arc<RwLock<HuffmanTree>>) -> Result<Vec<Byte>, ()> {
        let mut res = vec![];
        let mut t = Arc::clone(&tree);
        let mut byte_idx = 0usize;
        let mut bit_idx = 0usize;

        loop {
            //非叶点节上不会有数据
            if let Some(v) = Arc::clone(&t).read().unwrap().data {
                res.push(v);
                t = Arc::clone(&tree);
            } else {
                walk_on_tree!(encoded, t, byte_idx, bit_idx);
            }
        }

        if 0 < encoded.pad_len {
            byte_idx = encoded.data.len() - 1;
            bit_idx = BYTE_SIZ - encoded.pad_len;
            loop {
                //非叶点节上不会有数据
                if let Some(_) = Arc::clone(&t).read().unwrap().data {
                    res.pop();
                    t = Arc::clone(&tree);
                } else {
                    walk_on_tree!(encoded, t, byte_idx, bit_idx);
                }
            }
        }

        Ok(res)
    }
}

//> n取值范围：[0, 7]
//- #: 若第n位bit为1，返回true，否则返回false
#[inline]
fn check_bit(data: Byte, n: usize) -> bool {
    if 0 < data & BIT_SET[n] {
        true
    } else {
        false
    }
}

//> It is cheaper to return a Byte than use a pointer
#[inline]
fn set_bit(data: Byte, n: usize) -> Byte {
    data | BIT_SET[n]
}

#[cfg(test)]
mod tests {
    //use super::*;
    //use rayon::prelude::*;
    //use std::collections::HashMap;

    #[test]
    fn huffman() {}
}
