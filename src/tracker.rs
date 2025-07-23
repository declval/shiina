use crate::download::Download;
use reqwest::Url;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Peer {
    peer_id: Option<Vec<u8>>,
    ip: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct Response {
    interval: i64,
    peers: Vec<Peer>,
}

pub struct Tracker {
    announce: String,
    interval: i64,
}

impl Tracker {
    pub fn new(announce: String) -> Self {
        Self {
            announce,
            interval: 0,
        }
    }

    async fn request(
        &mut self,
        download: &mut Download,
        event: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut params = Vec::from([
            ("downloaded", download.downloaded.to_string()),
            ("event", event.to_string()),
            ("left", download.left().to_string()),
            ("port", crate::PORT.to_string()),
            ("uploaded", download.uploaded.to_string()),
        ]);

        if let Some(ip) = crate::IP {
            params.push(("ip", ip));
        }

        let url = Url::parse_with_params(&self.announce, params).unwrap();

        // Add these params separatly to avoid default URL encoding
        let url = format!(
            "{}&info_hash={}&peer_id={}",
            url,
            url_encode(&download.info_hash),
            url_encode(&download.peer_id),
        );

        println!("request: {}", url);

        let response = reqwest::get(url).await?.bytes().await?;

        let response = match crate::bencode::from_bytes::<Response>(&response) {
            Ok(response) => response,
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        };

        println!("response: {:?}", response);

        self.interval = response.interval;

        download.peers = response.peers;

        println!("download: {:?}", download);

        Ok(())
    }

    pub async fn completed(
        &mut self,
        download: &mut Download,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.request(download, "completed").await
    }

    pub async fn started(
        &mut self,
        download: &mut Download,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.request(download, "started").await
    }

    pub async fn stopped(
        &mut self,
        download: &mut Download,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.request(download, "stopped").await
    }
}

fn url_encode(bytes: &[u8]) -> String {
    let mut res = String::new();
    bytes
        .into_iter()
        .for_each(|byte| res.push_str(&format!("%{:02x}", byte)));
    res
}
