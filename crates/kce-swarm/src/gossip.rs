//! FT-029 — Swarm Gossip Protocol.
//!
//! P2P communication for syncing feromônios, antigens, and graph deltas via UDP.

use bytes::{Buf, BufMut};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use uuid::Uuid;

/// Swarm Message Types (FT-029 AC-011).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    PheromoneSync = 0x01,
    AntigenSync = 0x02,
    GraphDeltaSync = 0x03,
}

impl TryFrom<u8> for MessageType {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x01 => Ok(Self::PheromoneSync),
            0x02 => Ok(Self::AntigenSync),
            0x03 => Ok(Self::GraphDeltaSync),
            _ => Err(format!("unknown message type: 0x{:02x}", v)),
        }
    }
}

/// Swarm Binary Packet structure (FT-029 AC-010).
#[derive(Debug, Clone)]
pub struct SwarmPacket {
    pub source_node_id: Uuid,
    pub sequence_number: u32,
    pub message_type: MessageType,
    pub causal_watermark: u64,
    pub payload: Vec<u8>,
}

const MAGIC_BYTE: u8 = 0x03;

impl SwarmPacket {
    /// Serialize packet to binary buffer.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(30 + self.payload.len());
        buf.put_u8(MAGIC_BYTE);
        buf.put_slice(self.source_node_id.as_bytes());
        buf.put_u32(self.sequence_number);
        buf.put_u8(self.message_type as u8);
        buf.put_u64(self.causal_watermark);
        buf.put_slice(&self.payload);
        buf
    }

    /// Deserialize packet from binary buffer (FT-029 AC-010, AC-002).
    pub fn deserialize(mut data: &[u8]) -> Result<Self, String> {
        if data.len() < 30 {
            return Err("packet too short".into());
        }
        if data.get_u8() != MAGIC_BYTE {
            return Err("invalid magic byte".into());
        }

        let mut node_bytes = [0u8; 16];
        data.copy_to_slice(&mut node_bytes);
        let source_node_id = Uuid::from_bytes(node_bytes);

        let sequence_number = data.get_u32();
        let message_type = MessageType::try_from(data.get_u8())?;
        let causal_watermark = data.get_u64();
        let payload = data.to_vec();

        Ok(Self {
            source_node_id,
            sequence_number,
            message_type,
            causal_watermark,
            payload,
        })
    }
}

/// Gossip Node for P2P communication.
pub struct GossipNode {
    pub local_id: Uuid,
    socket: Arc<UdpSocket>,
    peers: Arc<parking_lot::RwLock<Vec<SocketAddr>>>,
}

impl GossipNode {
    pub async fn new(local_id: Uuid, addr: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Self {
            local_id,
            socket: Arc::new(socket),
            peers: Arc::new(parking_lot::RwLock::new(Vec::new())),
        })
    }

    pub fn add_peer(&self, addr: SocketAddr) {
        self.peers.write().push(addr);
    }

    /// Broadcast message to K random neighbors (FT-029 AC-014).
    pub async fn broadcast(&self, message_type: MessageType, watermark: u64, payload: Vec<u8>) {
        let packet = SwarmPacket {
            source_node_id: self.local_id,
            sequence_number: rand::random(),
            message_type,
            causal_watermark: watermark,
            payload,
        };
        let data = packet.serialize();

        let peers = self.peers.read();
        // Epidemic limited broadcast: pick K=3 random neighbors
        let k = 3;
        let targets = if peers.len() <= k {
            peers.clone()
        } else {
            // Simplified random selection for MVP
            peers.iter().take(k).copied().collect()
        };

        for target in targets {
            let _ = self.socket.send_to(&data, target).await;
        }
    }

    /// Main loop for listening to gossip messages.
    pub async fn listen<F>(&self, handler: F) -> std::io::Result<()> 
    where F: Fn(SwarmPacket) + Send + Sync + 'static
    {
        let mut buf = [0u8; 4096];
        loop {
            let (len, _addr) = self.socket.recv_from(&mut buf).await?;
            if let Ok(packet) = SwarmPacket::deserialize(&buf[..len]) {
                handler(packet);
            }
        }
    }
}
