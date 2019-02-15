macro_rules! source_type_test {
    ($name: ident, $type: ty) => {
        mod $name {
            use super::super::*;
            use rand::random;

            pub fn rand() -> Vec<impl AsBytes> {
                let mut sample = vec![];
                (0..500).for_each(|_| sample.push(random::<$type>()));
                sample.sort();
                sample.dedup();
                sample
            }

            pub fn rand_box() -> Vec<impl AsBytes> {
                let mut sample = vec![];
                (0..500).for_each(|_| {
                    sample.push(
                        (0..10)
                            .into_iter()
                            .map(|_| random::<$type>())
                            .collect::<Box<[$type]>>(),
                    )
                });
                sample.sort();
                sample.dedup();
                sample
            }

            pub fn rand_vec() -> Vec<impl AsBytes> {
                let mut sample = vec![];
                (0..500).for_each(|_| {
                    sample.push(
                        (0..10)
                            .into_iter()
                            .map(|_| random::<$type>())
                            .collect::<Vec<$type>>(),
                    )
                });
                sample.sort();
                sample.dedup();
                sample
            }

            pub fn $name<T: AsBytes>(sample: Vec<T>) {
                let mut hashsigs = vec![];
                let mut sl = SkipList::default();

                for v in sample.iter().cloned() {
                    hashsigs.push(sl.put(v).unwrap());
                }

                assert_eq!(sample.len(), sl.item_cnt());
                assert_eq!(hashsigs.len(), sl.item_cnt());
                assert_eq!(sl.item_cnt_realtime(), sl.item_cnt());

                assert!(sl.root_merklesig().is_some());
                for (v, h) in sample.iter().zip(hashsigs.iter()) {
                    assert_eq!(v, &sl.get(h).unwrap());
                    assert!(sl.proof(h).unwrap());
                }
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

source_type_test!(msl_char, char);
source_type_test!(msl_u8, u8);
source_type_test!(msl_u16, u16);
source_type_test!(msl_u32, u32);
source_type_test!(msl_u64, u64);
source_type_test!(msl_u128, u128);
source_type_test!(msl_usize, usize);
source_type_test!(msl_i8, i8);
source_type_test!(msl_i16, i16);
source_type_test!(msl_i32, i32);
source_type_test!(msl_i64, i64);
source_type_test!(msl_i128, i128);
source_type_test!(msl_isize, isize);
