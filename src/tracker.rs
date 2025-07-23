use crate::Metainfo;
use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Peer {
    peer_id: Option<Vec<u8>>,
    ip: String,
    port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub interval: i64,
    pub peers: Vec<Peer>,
}

pub struct Tracker {
    url: Url,
    info_hash: String,
    peer_id: String,
}

impl Tracker {
    pub fn new(torrent: Metainfo, info_hash: String, peer_id: String) -> Self {
        let url = Url::parse_with_params(
            &torrent.announce,
            &[
                ("port", "6881"),
                ("uploaded", "0"),
                ("downloaded", "0"),
                ("left", torrent.info.length.to_string().as_ref()),
                ("event", "started"),
            ],
        )
        .unwrap();

        Self {
            url,
            info_hash,
            peer_id,
        }
    }

    pub async fn started(&self) -> Result<Response, Box<dyn std::error::Error>> {
        let response = reqwest::get(format!(
            "{}&info_hash={}&peer_id={}",
            self.url, self.info_hash, self.peer_id
        ))
        .await?
        .text()
        .await?;

        Ok(crate::bencode::from_bytes::<Response>(&response.as_bytes()).unwrap())
    }
}
