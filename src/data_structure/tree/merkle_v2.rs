//! ## Sorted Merkle Tree
//!
//! #### 算法说明
//! - 使用多维Vec实现的哈希树。
//!
//! #### 应用场景
//! - 数据检验，存在性证明。
//!
//! #### 实现属性
//! - <font color=Red>×</font> 多线程安全
//! - <font color=Green>√</font> 无 unsafe 代码

use ring::digest::{Context, SHA1};
use std::ops::{Deref, DerefMut};

type HashSig = Vec<u8>;
type HashLayer = Vec<HashSig>;

#[derive(Debug)]
struct Merkle(Vec<HashLayer>);

impl Deref for Merkle {
    type Target = Vec<HashLayer>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Merkle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[inline]
fn hash(item: &[u8]) -> Vec<u8> {
    let mut context = Context::new(&SHA1);
    context.update(item);
    context.finish().as_ref().to_vec()
}

#[derive(Clone, Debug)]
pub struct Proof {
    prepend: bool,
    hash: HashSig,
}

impl Merkle {
    pub fn new(leaf_layer: HashLayer) -> Option<Merkle> {
        if leaf_layer.is_empty() {
            return None;
        }

        let mut res = Merkle(vec![leaf_layer]);
        if 1 == res[0].len() {
            return Some(res);
        }

        //for binary_search()
        res[0].sort();

        //leaf_layer pad
        if 0 < res[0].len() % 2 {
            res[0].push(vec![]);
        }

        let mut next_layer;
        let mut toplayer_idx;
        let mut i;
        let mut h;
        while 1 < res[res.len() - 1].len() {
            next_layer = vec![];
            toplayer_idx = res.len() - 1;
            i = 0;
            while i < res[toplayer_idx].len() {
                h = res[toplayer_idx][i].clone();
                h.append(&mut (res[toplayer_idx][i + 1].clone()));
                next_layer.push(hash(&h));
                i += 2;
            }

            if 1 < next_layer.len() && 0 < next_layer.len() % 2 {
                next_layer.push(vec![]);
            }
            res.push(next_layer);
        }

        Some(res)
    }

    ///unsorted merkle tree can ONLY give positive proof
    pub fn proof(&self, hashsig: Vec<u8>) -> Option<Vec<Proof>> {
        if let Ok(mut idx) = self[0][..].binary_search(&hashsig) {
            let mut res = vec![];
            res.push(Proof {
                prepend: false,
                hash: self[0][idx].clone(),
            });

            //排除root层
            for layer in self.iter().take(self.len() - 1) {
                if 0 == idx % 2 {
                    //自身在左，兄弟节点一定在右
                    res.push(Proof {
                        prepend: false,
                        hash: layer[idx + 1].clone(),
                    });
                } else {
                    //自身在右，则不可能是第一个元素，兄弟节点一定在左
                    res.push(Proof {
                        prepend: true,
                        hash: layer[idx - 1].clone(),
                    });
                }

                //计算向上一层(father layer)中的`父`索引
                idx /= 2;
            }

            Some(res)
        } else {
            None
        }
    }

    pub fn calculate_root(
        hash_path: &[Proof],
        hasher: impl Fn(&[u8]) -> Vec<u8>,
    ) -> Option<Vec<u8>> {
        if hash_path.is_empty() {
            return None;
        }

        let res = hash_path[0].clone();
        if 1 == hash_path.len() {
            return Some(res.hash);
        }

        Some(
            hash_path
                .iter()
                .skip(1)
                .fold(res, |mut prev, last| {
                    if last.prepend {
                        let mut h = last.hash.clone();
                        h.append(&mut prev.hash);
                        prev.hash = hasher(&h);
                    } else {
                        prev.hash.append(&mut last.hash.clone());
                        prev.hash = hasher(&prev.hash);
                    }
                    prev
                })
                .hash,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn merkle() {
        let mut sample = vec![];
        for i in 0i32..4 {
            sample.push(hash(&i.to_le_bytes()));
        }

        let merkle = Merkle::new(sample.clone()).unwrap();

        //positive proof
        sample.into_iter().for_each(|i| {
            assert_eq!(
                &merkle[merkle.len() - 1][0],
                &Merkle::calculate_root(&merkle.proof(i).unwrap(), hash).unwrap()
            );
        });
    }
}
