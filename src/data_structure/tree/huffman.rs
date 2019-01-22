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

type Byte = u8;
type ByteLen = u8; //[0, 7]
const ByteSiz: usize = std::mem::size_of::<BYTE>();

//> 预置bit集合，优化位运算的效率
const BitSet: [Byte; 8] = [
    b00000001, b00000010, b00000100, b00001000, b00010000, b00100000, b01000000, b10000000,
];

//> 以Byte为对象进行编解码的抽象数据类型，适配所有的数据类型
type HuffmanTree = Node;
struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,

    data: Option<Byte>,
}

//> 编码表使用线性索引优化性能
type EncodeTable = [Vec<Byte>; 256];
//> 解码表的用途仅是还原huffman tree，无需索引，亦与存储顺序无关
type DecodeTable = Vec<(Vec<Byte>, Byte)>;

pub trait Huffman {
    //> generate en[de]code-table
    //- @data：用于生成(编/解)码表的样本数据集
    fn gen_table(data: &[Byte]) -> (EncodeTable, DecodeTable) {}

    //> restore the HuffmanTree from a decode-table
    //- @dt: 由同一样本数据生成的或从编码方获取的码表
    fn restore_tree(dt: DecodeTable) -> HuffmanTree {}

    //
    //
    fn encode(data: &[Byte]) -> Vec<Byte> {}

    //
    //
    fn decode(data: &[Byte]) -> Vec<Byte> {}
}

//- #: 若第n位bit为1，返回true，否则返回false
#[inline]
fn check_bit(data: Byte, n: ByteLen) -> bool {
    data & BitSet[n]
}

//> 返回一个Byte比转入指针更廉价
#[inline]
fn set_bit(data: Byte, n: ByteLen) -> Byte {
    data | BitSet[n]
}

//> generate encoded result
//- #: the final encoded data
//- @bits[in]: assert!(0 == bits.len() % 8)
//- @res[out]: encoded result will be written here
fn encode_gen_res(bits: &[Byte], res: Vec<Byte>) -> Vec<Byte> {
    let mut byte: Byte = 0;
    let mut base = 0usize;
    for _ in 0..(bits.len() / 8) {
        for j in 0..8 {
            if 1 == bits[base + j as usize] {
                byte = set_bit(byte, j);
            }
        }
        res.push(byte);
        byte = 0;
        base += 8;
    }
}

struct Encoded<'a> {
    //the huffman tree which data has been encoded by
    tree: &'a HuffmanTree,
    //encoded data received from some sender[s]
    data: Vec<Byte>,
    //the number of bit[s] at the last byte for aligning to ByteSiz
    pad_len: ByteLen,
}

//> generate decoded result
//- #: success will be Ok(Vec<Byte>), otherwise Err(())
//- @encoded[in]: encoded data and meta
//- @res[out]: decoded result will be written here
fn decode_gen_res(encoded: Encoded, res: Vec<Byte>) -> Result<Vec<Byte>, ()> {
    let mut t = tree;
    let mut offset = 0;

    for i in 0..data.len() {
        for j in offset..8 {
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;
    use std::collections::HashMap;

    #[test]
    fn huffman() {}
}
