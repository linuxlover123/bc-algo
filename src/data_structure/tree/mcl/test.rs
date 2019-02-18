macro_rules! source_type_test {
    ($name: ident, $type: ty) => {
        mod $name {
            use super::super::*;
            use rand::random;
            use std::collections::HashSet;

            pub fn rand() -> Vec<impl AsBytes> {
                let mut hset = HashSet::new();
                (0..500).for_each(|_| {
                    hset.insert(random::<$type>());
                });
                let mut sample = vec![];
                hset.into_iter().for_each(|i| sample.push(i));
                sample
            }

            pub fn rand_box() -> Vec<impl AsBytes> {
                let mut hset = HashSet::new();
                (0..500).for_each(|_| {
                    hset.insert(
                        (0..10)
                            .into_iter()
                            .map(|_| random::<$type>())
                            .collect::<Box<[$type]>>(),
                    );
                });
                let mut sample = vec![];
                hset.into_iter().for_each(|i| sample.push(i));
                sample
            }

            pub fn rand_vec() -> Vec<impl AsBytes> {
                let mut hset = HashSet::new();
                (0..500).for_each(|_| {
                    hset.insert(
                        (0..10)
                            .into_iter()
                            .map(|_| random::<$type>())
                            .collect::<Vec<$type>>(),
                    );
                });
                let mut sample = vec![];
                hset.into_iter().for_each(|i| sample.push(i));
                sample
            }

            pub fn $name<T: AsBytes>(sample: Vec<T>) {
                let mut hashsigs = vec![];
                let mut mcl = MCL::default();

                for v in sample.iter().cloned() {
                    hashsigs.push(mcl.put(v).unwrap());
                }

                assert_eq!(sample.len(), mcl.item_cnt());
                assert_eq!(hashsigs.len(), mcl.item_cnt());
                assert_eq!(mcl.item_cnt_realtime(), mcl.item_cnt());

                assert!(mcl.merklesig().is_some());
                for (v, h) in sample.iter().zip(hashsigs.iter()) {
                    assert_eq!(v, &mcl.get(h).unwrap());
                    assert!(mcl.proof(h).unwrap());
                }

                //测试有序性
                let mut hashsigs2 = hashsigs.clone();
                hashsigs2.sort();
                let mut node = MCL::get_lowest_node(mcl.get_inner(&hashsigs2[0]).unwrap());
                let mut left = Rc::clone(&node);
                assert!(Weak::upgrade(&node.left).is_none());
                for h in &hashsigs2[1..] {
                    dbg!(1);
                    node = MCL::get_lowest_node(mcl.get_inner(h).unwrap());
                    assert!(Rc::ptr_eq(&left, &Weak::upgrade(&node.left).unwrap()));
                    left = Rc::clone(&node);
                }

                //测试移除后的成确性
                for (v, h) in sample.iter().zip(hashsigs.iter()) {
                    assert_eq!(v, &*mcl.remove(h).unwrap().value);
                    assert!(mcl.get(h).is_none());
                    assert!(!mcl.proof(h).unwrap());
                }

                assert_eq!(0, mcl.item_cnt());
                assert_eq!(mcl.item_cnt_realtime(), mcl.item_cnt());
            }
        }

        #[test]
        fn $name() {
            let sample0 = $name::rand();
            let sample1 = $name::rand_box();
            let sample2 = $name::rand_vec();

            $name::$name(sample0);
            $name::$name(sample1);
            $name::$name(sample2);
        }
    };
}

source_type_test!(mcl_u8, u8);
//source_type_test!(mcl_u16, u16);
//source_type_test!(mcl_u32, u32);
//source_type_test!(mcl_u64, u64);
//source_type_test!(mcl_u128, u128);
//source_type_test!(mcl_usize, usize);
//source_type_test!(mcl_i8, i8);
//source_type_test!(mcl_i16, i16);
//source_type_test!(mcl_i32, i32);
//source_type_test!(mcl_i64, i64);
//source_type_test!(mcl_i128, i128);
//source_type_test!(mcl_isize, isize);
//source_type_test!(mcl_char, char);
