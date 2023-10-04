#[macro_use]
extern crate afl;

use bytes::BytesMut;
use snarkos_node_router_messages::{BlockResponse, MessageTrait};

type CurrentNetwork = snarkvm::prelude::Testnet3;

fn main() {
    fuzz!(|data: &[u8]| {
        let mut buf = BytesMut::with_capacity(data.len());
        buf.extend_from_slice(data);
        let msg = match BlockResponse::<CurrentNetwork>::deserialize(buf) {
            Ok(response) => match response.blocks.deserialize_blocking() {
                Ok(..) => "OK",
                Err(..) => "inner error",
            },
            Err(..) => "outer error",
        };
        eprintln!("{}", msg);
    });
}
