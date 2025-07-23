use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    length: i64,
    path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Info<'a> {
    Single {
        length: i64,
        name: String,
        #[serde(rename = "piece length")]
        piece_length: i64,
        #[serde(borrow)]
        #[serde(with = "serde_bytes")]
        pieces: &'a [u8],
    },
    Multi {
        files: Vec<File>,
        name: String,
        #[serde(rename = "piece length")]
        piece_length: i64,
        #[serde(borrow)]
        #[serde(with = "serde_bytes")]
        pieces: &'a [u8],
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metainfo<'a> {
    pub announce: String,
    comment: String,
    #[serde(rename = "created by")]
    created_by: String,
    #[serde(rename = "creation date")]
    creation_date: i64,
    #[serde(borrow)]
    pub info: Info<'a>,
    #[serde(rename = "url-list")]
    url_list: Vec<String>,
}

impl Metainfo<'_> {
    pub fn info_hash(&self) -> Result<Vec<u8>, crate::bencode::Error> {
        Ok(sha1(&crate::bencode::to_bytes(&self.info)?))
    }
}

fn sha1(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    hasher.finalize().into_iter().collect()
}
