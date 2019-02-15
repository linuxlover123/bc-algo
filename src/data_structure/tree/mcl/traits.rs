pub trait AsBytes: std::fmt::Debug + Clone + Eq + PartialEq + Ord + PartialOrd {
    fn as_bytes(&self) -> Box<[u8]>;
}

impl AsBytes for String {
    fn as_bytes(&self) -> Box<[u8]> {
        self.as_bytes().to_vec().into_boxed_slice()
    }
}

impl AsBytes for char {
    fn as_bytes(&self) -> Box<[u8]> {
        (*self as u32).as_bytes()
    }
}

impl AsBytes for Box<[char]> {
    fn as_bytes(&self) -> Box<[u8]> {
        let convert = self.iter().map(|i| *i as u32).collect::<Vec<u32>>();
        convert.as_bytes()
    }
}

impl AsBytes for Vec<char> {
    fn as_bytes(&self) -> Box<[u8]> {
        let convert = self.iter().map(|i| *i as u32).collect::<Vec<u32>>();
        convert.as_bytes()
    }
}

macro_rules! impl_as_bytes {
    (@$obj: ty) => {
        impl AsBytes for $obj {
            fn as_bytes(&self) -> Box<[u8]> {
                Box::new(self.to_le_bytes())
            }
        }
    };
    ($obj: ty) => {
        impl AsBytes for $obj {
            fn as_bytes(&self) -> Box<[u8]> {
                let mut res = vec![];
                self.iter().for_each(|i| res.extend(&i.to_le_bytes()));
                res.into_boxed_slice()
            }
        }
    };
}

impl_as_bytes!(@u8);
impl_as_bytes!(@u16);
impl_as_bytes!(@u32);
impl_as_bytes!(@u64);
impl_as_bytes!(@u128);
impl_as_bytes!(@usize);

impl_as_bytes!(@i8);
impl_as_bytes!(@i16);
impl_as_bytes!(@i32);
impl_as_bytes!(@i64);
impl_as_bytes!(@i128);
impl_as_bytes!(@isize);

impl_as_bytes!(Box<[u8]>);
impl_as_bytes!(Box<[u16]>);
impl_as_bytes!(Box<[u32]>);
impl_as_bytes!(Box<[u64]>);
impl_as_bytes!(Box<[u128]>);
impl_as_bytes!(Box<[usize]>);

impl_as_bytes!(Box<[i8]>);
impl_as_bytes!(Box<[i16]>);
impl_as_bytes!(Box<[i32]>);
impl_as_bytes!(Box<[i64]>);
impl_as_bytes!(Box<[i128]>);
impl_as_bytes!(Box<[isize]>);

impl_as_bytes!(Vec<u8>);
impl_as_bytes!(Vec<u16>);
impl_as_bytes!(Vec<u32>);
impl_as_bytes!(Vec<u64>);
impl_as_bytes!(Vec<u128>);
impl_as_bytes!(Vec<usize>);

impl_as_bytes!(Vec<i8>);
impl_as_bytes!(Vec<i16>);
impl_as_bytes!(Vec<i32>);
impl_as_bytes!(Vec<i64>);
impl_as_bytes!(Vec<i128>);
impl_as_bytes!(Vec<isize>);
