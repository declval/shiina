mod bencode;
mod tracker;

use bencode::Error;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tracker::Tracker;

const PROGRAM: &str = "shiina";

#[derive(Debug, Serialize, Deserialize)]
struct Info<'a> {
    length: i64,
    name: String,
    #[serde(rename = "piece length")]
    piece_length: i64,
    #[serde(borrow)]
    #[serde(with = "serde_bytes")]
    pieces: &'a [u8],
}

#[derive(Debug, Serialize, Deserialize)]
struct Metainfo<'a> {
    announce: String,
    comment: String,
    #[serde(rename = "created by")]
    created_by: String,
    #[serde(rename = "creation date")]
    creation_date: i64,
    #[serde(borrow)]
    info: Info<'a>,
    #[serde(rename = "url-list")]
    url_list: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = if let Some(file_name) = std::env::args().skip(1).next() {
        file_name
    } else {
        usage();
        std::process::exit(1);
    };

    let contents = match std::fs::read(&file_name) {
        Ok(contents) => contents,
        Err(message) => {
            eprintln!("{}: {}", file_name, message);
            std::process::exit(1);
        }
    };

    let torrent = match bencode::from_bytes::<Metainfo>(&contents) {
        Ok(torrent) => torrent,
        Err(err) => {
            eprintln!("{}: {}", file_name, err);
            std::process::exit(1);
        }
    };

    let info_bytes = match bencode::to_bytes(&torrent.info) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("{}: {}", file_name, err);
            std::process::exit(1);
        }
    };

    let info_hash = sha1(&info_bytes);

    let mut peer_id = [0u8; 20];
    rand::fill(&mut peer_id);

    let tracker = Tracker::new(torrent, url_encode(&info_hash), url_encode(&peer_id));

    let response = tracker.started().await?;

    for peer in response.peers {
        println!("{:?}", peer);
    }

    Ok(())
}

fn usage() {
    eprintln!("Usage: {} <torrent file>", PROGRAM);
}

fn url_encode(bytes: &[u8]) -> String {
    let mut res = String::new();
    bytes
        .into_iter()
        .for_each(|byte| res.push_str(&format!("%{:02x}", byte)));
    res
}

fn sha1(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    hasher.finalize().into_iter().collect()
}
