pub mod v1 {
    //! ## Merkle Tree
    //!
    //! #### 算法说明
    //! - 哈希树。
    //!
    //! #### 应用场景
    //! - 数据检验，存在性证明。
    //!
    //! #### 实现属性
    //! - <font color=Red>×</font> 多线程安全
    //! - <font color=Green>√</font> 无 unsafe 代码

    use ring::digest::{Context, SHA1};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::{Rc, Weak};

    pub struct Merkle {
        tree: Option<Rc<RefCell<MerkleTree>>>,
        leaves: HashMap<Vec<u8>, Rc<RefCell<MerkleTree>>>,
    }

    #[derive(Default, Debug)]
    pub struct MerkleTree {
        hash: Vec<u8>,
        parent: Option<Rc<RefCell<MerkleTree>>>,
        brother_left: Option<Weak<RefCell<MerkleTree>>>,
        brother_right: Option<Rc<RefCell<MerkleTree>>>,
    }

    #[derive(Clone, Debug)]
    pub struct Proof {
        prepend: bool,
        hash: Vec<u8>,
    }

    impl Merkle {
        //should be a tail recursion
        fn gen(mut todo: Vec<Rc<RefCell<MerkleTree>>>) -> Vec<Rc<RefCell<MerkleTree>>> {
            if 1 == todo.len() {
                return todo;
            }

            if 0 < todo.len() % 2 {
                todo.push(Rc::new(RefCell::new(MerkleTree {
                    hash: vec![],
                    parent: None,
                    brother_left: None,
                    brother_right: None,
                })));
            }

            let mut res = Vec::with_capacity(todo.len() / 2);
            let mut hashsig;
            let todo = todo.chunks(2);
            for pair in todo {
                hashsig = pair[0].borrow().hash.clone();
                hashsig.extend(pair[1].borrow().hash.iter());
                hashsig = hash(&hashsig);

                res.push(Rc::new(RefCell::new(MerkleTree {
                    hash: hashsig,
                    parent: None,
                    brother_left: None,
                    brother_right: None,
                })));

                pair[0].borrow_mut().parent = Some(Rc::clone(res.last().unwrap()));
                pair[1].borrow_mut().parent = Some(Rc::clone(res.last().unwrap()));

                pair[0].borrow_mut().brother_right = Some(Rc::clone(&pair[1]));
                pair[1].borrow_mut().brother_left = Some(Rc::downgrade(&Rc::clone(&pair[0])));
            }

            Self::gen(res)
        }

        pub fn new(mut leaves: Vec<Vec<u8>>) -> Option<Merkle> {
            let mut res = Merkle {
                tree: None,
                leaves: HashMap::new(),
            };

            if leaves.is_empty() {
                return None;
            } else if 1 == leaves.len() {
                res.tree = Some(Rc::new(RefCell::new(MerkleTree {
                    hash: leaves[0].clone(),
                    parent: None,
                    brother_left: None,
                    brother_right: None,
                })));
                res.leaves.insert(
                    leaves.pop().unwrap(),
                    Rc::clone(&res.tree.as_ref().unwrap()),
                );
                return Some(res);
            }

            //确保索引形成偶数对
            if 0 < leaves.len() % 2 {
                leaves.push(vec![]);
            }

            let todo = leaves
                .into_iter()
                .map(|hash| {
                    let leaf = Rc::new(RefCell::new(MerkleTree {
                        hash: hash.clone(),
                        parent: None,
                        brother_left: None,
                        brother_right: None,
                    }));
                    res.leaves.insert(hash, Rc::clone(&leaf));
                    leaf
                })
                .collect::<Vec<Rc<RefCell<MerkleTree>>>>();

            res.tree = Some(Self::gen(todo).pop().unwrap());
            Some(res)
        }

        fn get_proof(me: Rc<RefCell<MerkleTree>>, res: &mut Vec<Proof>) {
            if me.borrow().parent.is_some() {
                let next;
                if let Some(v) = me.borrow().brother_right.as_ref() {
                    res.push(Proof {
                        prepend: false,
                        hash: v.borrow().hash.clone(),
                    });
                    next = Rc::clone(&me.borrow().parent.as_ref().unwrap());
                } else if let Some(v) = me.borrow().brother_left.as_ref() {
                    res.push(Proof {
                        prepend: true,
                        hash: v.upgrade().unwrap().borrow().hash.clone(),
                    });
                    next = Rc::clone(&me.borrow().parent.as_ref().unwrap());
                } else {
                    panic!("BUG");
                }

                Self::get_proof(next, res);
            } else {
                return;
            }
        }

        ///unsorted merkle tree can ONLY give positive proof
        pub fn proof(&self, hash: Vec<u8>) -> Option<Vec<Proof>> {
            if let Some(v) = self.leaves.get(&hash) {
                let mut res = vec![];
                res.push(Proof {
                    prepend: false,
                    hash: v.borrow().hash.clone(),
                });
                Self::get_proof(Rc::clone(&v), &mut res);

                Some(res)
            } else {
                None
            }
        }

        pub fn calculate_root(
            hash_path: &[Proof],
            hasher: impl Fn(&[u8]) -> Vec<u8>,
        ) -> Option<Vec<u8>> {
            let res = hash_path[0].clone();
            if hash_path.is_empty() {
                return None;
            } else if 1 == hash_path.len() {
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
                            prev.hash.extend(last.hash.iter());
                            prev.hash = hasher(&prev.hash);
                        }
                        prev
                    })
                    .hash,
            )
        }
    }

    #[inline]
    fn hash(item: &[u8]) -> Vec<u8> {
        let mut context = Context::new(&SHA1);
        context.update(item);
        context.finish().as_ref().to_vec()
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn merkle() {
            let mut sample = vec![];
            for i in 0i32..40 {
                sample.push(hash(&i.to_le_bytes()));
            }

            let merkle = Merkle::new(sample.clone()).unwrap();

            //positive proof
            sample.into_iter().for_each(|i| {
                assert_eq!(
                    &merkle.tree.as_ref().unwrap().borrow().hash,
                    &Merkle::calculate_root(&merkle.proof(i).unwrap(), hash).unwrap()
                );
            });
        }
    }
}

pub mod v2 {
    //! ## Merkle Tree
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
                    h.extend(res[toplayer_idx][i + 1].iter());
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
                            prev.hash.extend(last.hash.iter());
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
}
