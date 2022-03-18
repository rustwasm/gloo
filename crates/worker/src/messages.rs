use serde::{Deserialize, Serialize};

use crate::handler_id::HandlerId;
use crate::traits::Worker;

/// Serializable messages to worker
#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum ToWorker<W>
where
    W: Worker,
{
    /// Client is connected
    Connected(HandlerId),
    /// Incoming message to Worker
    ProcessInput(HandlerId, W::Input),
    /// Client is disconnected
    Disconnected(HandlerId),
    /// Worker should be terminated
    Destroy,
}

/// Serializable messages sent by worker to consumer
#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum FromWorker<W>
where
    W: Worker,
{
    /// Worker sends this message when `wasm` bundle has loaded.
    WorkerLoaded,
    /// Outgoing message to consumer
    ProcessOutput(HandlerId, W::Output),
}

/// Message packager, based on serde::Serialize/Deserialize
pub(crate) trait Packed {
    /// Pack serializable message into Vec<u8>
    fn pack(&self) -> Vec<u8>;
    /// Unpack deserializable message of byte slice
    fn unpack(data: &[u8]) -> Self;
}

impl<T> Packed for T
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn pack(&self) -> Vec<u8> {
        bincode::serialize(&self).expect("can't serialize an worker message")
    }

    fn unpack(data: &[u8]) -> Self {
        bincode::deserialize(data).expect("can't deserialize an worker message")
    }
}
