#![allow(dead_code)]

pub mod data_structure;
pub mod p2p_routing;
pub mod consensus;
pub mod other;

//pub use crate::data_structure::linkedlist;
//pub use crate::data_structure::tree;
//pub use crate::data_structure::graph;
//pub use crate::data_structure::bloomfilter;
//pub use crate::p2p_routing::kademlia;
//pub use crate::p2p_routing::s_kademlia;
//pub use crate::p2p_routing::coral;
//pub use crate::consensus::bitcoin_pow;
//pub use crate::consensus::ethereum_pow;
//pub use crate::consensus::raft;
//pub use crate::other::ethereum_rlp;


pub use crate::data_structure::linkedlist::one_way::OneWayLinkedList;
pub use crate::data_structure::linkedlist::two_way::TwoWayLinkedList;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
