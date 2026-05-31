use std::io::Read;
use std::path::Path;

use sha1::{Digest, Sha1};
use sha2::Sha512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha1,
    Sha512,
    Murmur2,
}

pub fn hash_file(path: &Path, algorithm: HashAlgorithm) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("Cannot open file: {}", e))?;

    match algorithm {
        HashAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            let mut buf = [0u8; 8192];
            loop {
                let n = file
                    .read(&mut buf)
                    .map_err(|e| format!("Read error: {}", e))?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            let mut buf = [0u8; 8192];
            loop {
                let n = file
                    .read(&mut buf)
                    .map_err(|e| format!("Read error: {}", e))?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        HashAlgorithm::Murmur2 => {
            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .map_err(|e| format!("Read error: {}", e))?;
            Ok(murmur2_hash(&data))
        }
    }
}

pub fn hash_data(data: &[u8], algorithm: HashAlgorithm) -> String {
    match algorithm {
        HashAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
        HashAlgorithm::Murmur2 => murmur2_hash(data),
    }
}

fn murmur2_hash(data: &[u8]) -> String {
    let filtered: Vec<u8> = data
        .iter()
        .filter(|&&b| b != b'\t' && b != b'\n' && b != b'\r' && b != b' ')
        .copied()
        .collect();

    let seed: u32 = 1;
    let m: u32 = 0x5bd1e995;
    let r: u32 = 24;

    let len = filtered.len();
    let mut h: u32 = seed ^ (len as u32);

    let mut i = 0;
    while i + 4 <= len {
        let mut k: u32 = filtered[i] as u32
            | (filtered[i + 1] as u32) << 8
            | (filtered[i + 2] as u32) << 16
            | (filtered[i + 3] as u32) << 24;

        k = k.wrapping_mul(m);
        k ^= k >> r;
        k = k.wrapping_mul(m);

        h = h.wrapping_mul(m);
        h ^= k;

        i += 4;
    }

    match len - i {
        3 => {
            h ^= (filtered[i + 2] as u32) << 16;
            h ^= (filtered[i + 1] as u32) << 8;
            h ^= filtered[i] as u32;
            h = h.wrapping_mul(m);
        }
        2 => {
            h ^= (filtered[i + 1] as u32) << 8;
            h ^= filtered[i] as u32;
            h = h.wrapping_mul(m);
        }
        1 => {
            h ^= filtered[i] as u32;
            h = h.wrapping_mul(m);
        }
        _ => {}
    }

    h ^= h >> 13;
    h = h.wrapping_mul(m);
    h ^= h >> 15;

    h.to_string()
}

pub fn create_hasher(algorithm: HashAlgorithm) -> Box<dyn Hasher> {
    match algorithm {
        HashAlgorithm::Sha1 => Box::new(Sha1Hasher { inner: Sha1::new() }),
        HashAlgorithm::Sha512 => Box::new(Sha512Hasher {
            inner: Sha512::new(),
        }),
        HashAlgorithm::Murmur2 => Box::new(Murmur2Hasher::new()),
    }
}

pub trait Hasher: Send {
    fn update(&mut self, data: &[u8]);
    fn finalize(&self) -> String;
}

struct Sha1Hasher {
    inner: Sha1,
}

impl Hasher for Sha1Hasher {
    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }
    fn finalize(&self) -> String {
        format!("{:x}", self.inner.clone().finalize())
    }
}

struct Sha512Hasher {
    inner: Sha512,
}

impl Hasher for Sha512Hasher {
    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }
    fn finalize(&self) -> String {
        format!("{:x}", self.inner.clone().finalize())
    }
}

struct Murmur2Hasher {
    data: Vec<u8>,
}

impl Murmur2Hasher {
    fn new() -> Self {
        Murmur2Hasher { data: Vec::new() }
    }
}

impl Hasher for Murmur2Hasher {
    fn update(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }
    fn finalize(&self) -> String {
        murmur2_hash(&self.data)
    }
}
