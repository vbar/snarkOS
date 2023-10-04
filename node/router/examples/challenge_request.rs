#[macro_use]
extern crate afl;

use bytes::BytesMut;
use snarkos_node_router_messages::{ChallengeRequest, MessageTrait};

type CurrentNetwork = snarkvm::prelude::Testnet3;

fn main() {
    fuzz!(|data: &[u8]| {
        let mut buf = BytesMut::with_capacity(data.len());
        buf.extend_from_slice(data);
        let msg = match ChallengeRequest::<CurrentNetwork>::deserialize(buf) {
            Ok(..) => "OK",
            Err(..) => "error",
        };
        eprintln!("{}", msg);
    });
}
