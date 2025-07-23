mod bencode;
mod download;
mod metainfo;
mod tracker;

use crate::download::Download;
use crate::metainfo::Metainfo;
use crate::tracker::Tracker;
use std::env;
use std::error;
use std::fs;
use std::process;

const IP: Option<String> = None;
const PEER_ID_PREFIX: &str = "-sh0010-";
const PORT: u16 = 6881;
const PROGRAM: &str = "shiina";

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let file_name = if let Some(file_name) = env::args().skip(1).next() {
        file_name
    } else {
        usage();
        process::exit(1);
    };

    let contents = match fs::read(&file_name) {
        Ok(contents) => contents,
        Err(message) => {
            eprintln!("{}: {}", file_name, message);
            process::exit(1);
        }
    };

    let torrent = match bencode::from_bytes::<Metainfo>(&contents) {
        Ok(torrent) => torrent,
        Err(err) => {
            eprintln!("{}: {}", file_name, err);
            process::exit(1);
        }
    };

    let mut tracker = Tracker::new(torrent.announce.clone());

    let mut download = Download::new(&torrent);

    tracker.started(&mut download).await?;

    Ok(())
}

fn usage() {
    eprintln!("Usage: {} <torrent file>", PROGRAM);
}
