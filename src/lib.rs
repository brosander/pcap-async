//! # pcap-async
//!
//! Async/await wrappers [pcap-sys](https://github.com/protectwise/pcap-sys).
//!
//!```no_run
//! use futures::StreamExt;
//! use pcap_async::{Config, Handle, PacketStream};
//! use std::sync::Arc;
//!
//! fn main() {
//!     let handle = Handle::lookup().expect("No handle created");
//!     smol::run(async move {
//!         let mut provider = PacketStream::new(Config::default(), Arc::clone(&handle))
//!             .expect("Could not create provider")
//!             .boxed();
//!         while let Some(packets) = provider.next().await {
//!
//!         }
//!         handle.interrupt();
//!     });
//! }
#![deny(unused_must_use, unused_imports, bare_trait_objects)]
#![allow(dead_code, unused_imports)]
pub mod bpf;
mod bridge_stream;
mod config;
pub mod errors;
mod handle;
mod info;
mod packet;
pub mod pcap_util;
mod stats;
mod stream;

pub use crate::{
    bridge_stream::BridgeStream, config::Config, errors::Error, handle::Handle, info::Info,
    packet::Packet, stats::Stats, stream::PacketStream, stream::StreamItem,
};
pub use byteorder::{BigEndian, LittleEndian, NativeEndian, WriteBytesExt};
use log::*;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use std::path::PathBuf;

    #[test]
    fn capture_from_file() {
        let _ = env_logger::try_init();

        let pcap_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("4SICS-GeekLounge-151020.pcap");

        info!("Benchmarking against {:?}", pcap_path.clone());

        let clone_path = pcap_path.clone();

        let handle = Handle::file_capture(clone_path.to_str().expect("No path found"))
            .expect("No handle created");

        let mut cfg = Config::default();
        cfg.with_max_packets_read(5000);

        let packets = smol::run(async move {
            let packet_provider =
                PacketStream::new(Config::default(), std::sync::Arc::clone(&handle))
                    .expect("Failed to build");
            let fut_packets = packet_provider.collect::<Vec<_>>();
            let packets: Result<Vec<_>, Error> = fut_packets.await.into_iter().collect();
            let packets = packets
                .expect("Could not get packets")
                .iter()
                .flatten()
                .count();

            handle.interrupt();

            packets
        });

        assert_eq!(packets, 246137);
    }
}
