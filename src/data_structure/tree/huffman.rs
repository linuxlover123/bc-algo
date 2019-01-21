//! ## 哈夫曼树与哈夫曼编码
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

use std::collections::HashMap;;
use rayon::prelude::*;

const BITSET: [u8; 8] = {
    b00000001,
    b00000010,
    b00000100,
    b00001000,
    b00010000,
    b00100000,
    b01000000,
    b10000000,
};

type HuffmanTree = Node;
struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,

    data: Option<u8>,
}

struct EncodedRes {
    data: Vec<u8>,
    pad_len: i32, //[0, 7]
}

///编解码对应表：
///- 编码表使用线性索引优化性能
///- 解码表的用途仅是还原 huffman 树，无需索引，亦与存储顺序无关
type EncodeTable = [Vec<u8>; 256];
type DecodeTable = Vec<(Vec<u8>, u8);

//各项均有默认实现
pub trait Huffman {
    //生成编解码表
    //@param data: 样本数据，生成(编/解)码表的基础
    fn gen_table(data: &[u8]) -> (EncodeTable, DecodeTable) {

    }

    //从解码表中还原 HuffmanTree
    //@param dt: 由同一样本数据生成的或从编码方获取的码表
    fn restore_tree(dt: DecodeTable) -> HuffmanTree {

    }

    fn encode(data:&[u8]) -> Vec<u8> {

    }

    fn decode(data:&[u8]) -> Vec<u8> {

    }
}

#[inline]
fn check_bit(data: u8, n: u8/*[0, 7]*/) -> bool {
    data & BITSET[n]
}

#[inline]
fn set_bit(data: u8, n: u8/*[0, 7]*/) -> u8 {
    data | BITSET[n]
}

//generate an encoded result
//@param bits: assert!(0 == bits.len() % 8)
fn encode_gen_res(bits: &[u8]) -> Vec<u8> {
    let mut res = vec![];
    let mut d = 0u8;
    let mut base = 0;
    for _ in 0..(bits.len() / 8) {
        for j in 0..8 {
            if 1 == bits[base + j] {
                d = set_bit(d, j);
            }
        }
        res.push(d);
        d = 0;
        base += 8;
    }

    res
}

//generate a decoded result
//@param data: encoded data recvd from a sender
//@param tree: the huffman tree which data has been encoded by
fn decode_gen_res(encoded: EncodedRes, tree: &HuffmanTree) -> Result<Vec<u8>, ()> {
    let mut res = vec![];
    let mut t = tree;
    let mut base = encoded.data;
    let mut offset = 0;

    for i in 0..encoded.data.len() {
        for j in offset..8u8 {
            if check_bit(base[i], j) {
                t = t.right;
                if None == t.data {
                    continue;
                } else {

                }
            } else {
                t = t.left;
                if None == t.data {
                    continue;
                } else {

                }

            }
        }


        res.push(t.data.unwrap());
        t = tree;
    }

    Ok(res)
}





#[cfg(test)]
mod tests {

    #[test]
    fn huffman() {
    }
}
