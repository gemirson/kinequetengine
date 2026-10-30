//! KCE Swarm — Distributed systems module.
//!
//! Implements Sharding, Gossip, and Consensus for the KineContext Engine cluster.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod consensus;
pub mod gossip;
pub mod sharding;

pub use consensus::{CrdtPheromones, RaftLite};
pub use gossip::{GossipNode, MessageType, SwarmPacket};
pub use sharding::{NodeId, NodeLoad, Shard, ShardRouter};
