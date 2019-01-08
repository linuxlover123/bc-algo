#![allow(dead_code)]

mod data_structure;
mod p2p_routing;
mod consensus;
mod other;

pub use crate::data_structure::linkedlist;
pub use crate::data_structure::linkedlist::one_way::OneWayLinkedList;
pub use crate::data_structure::linkedlist::two_way::TwoWayLinkedList;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
