use crate::metainfo::Info;
use crate::metainfo::Metainfo;
use crate::tracker::Peer;
use std::io::Write;

#[derive(Debug)]
pub struct Download {
    pub downloaded: i64,
    pub info_hash: Vec<u8>,
    length: i64,
    pub peer_id: Vec<u8>,
    pub peers: Vec<Peer>,
    pub uploaded: i64,
}

impl Download {
    pub fn new(metainfo: &Metainfo) -> Self {
        let info_hash = match metainfo.info_hash() {
            Ok(bytes) => bytes,
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        };

        let mut peer_id = [0u8; 20];
        rand::fill(&mut peer_id);
        peer_id
            .as_mut()
            .write(crate::PEER_ID_PREFIX.as_bytes())
            .unwrap();

        let length = match metainfo.info {
            Info::Single { length, .. } => length,
            _ => 0,
        };

        Self {
            downloaded: 0,
            info_hash,
            length,
            peer_id: peer_id.to_vec(),
            peers: Vec::new(),
            uploaded: 0,
        }
    }

    pub fn left(&self) -> i64 {
        self.length
    }
}
