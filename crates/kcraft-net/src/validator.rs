use sha1::Digest;

pub trait Validator: Send + Sync {
    fn init(&mut self);
    fn write(&mut self, data: &[u8]);
    fn abort(&mut self);
    fn validate(&mut self) -> bool;
}

pub struct ChecksumValidator {
    algorithm: HashAlgorithm,
    hasher: Option<Box<dyn HashEngine>>,
    expected: Option<String>,
    result: Option<String>,
}

#[allow(dead_code)]
enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
}

#[allow(dead_code)]
trait HashEngine: Send + Sync {
    fn update(&mut self, data: &[u8]);
    fn finalize(&mut self) -> String;
    fn box_clone(&self) -> Box<dyn HashEngine>;
}

struct Md5Engine(md5::Md5);

impl HashEngine for Md5Engine {
    fn update(&mut self, data: &[u8]) {
        use md5::Digest;
        self.0.update(data);
    }
    fn finalize(&mut self) -> String {
        use md5::Digest;
        hex::encode(self.0.finalize_reset())
    }
    fn box_clone(&self) -> Box<dyn HashEngine> {
        Box::new(Md5Engine(md5::Md5::new()))
    }
}

struct Sha256Engine(sha2::Sha256);

impl HashEngine for Sha256Engine {
    fn update(&mut self, data: &[u8]) {
        sha2::Digest::update(&mut self.0, data);
    }
    fn finalize(&mut self) -> String {
        hex::encode(sha2::Digest::finalize_reset(&mut self.0))
    }
    fn box_clone(&self) -> Box<dyn HashEngine> {
        Box::new(Sha256Engine(sha2::Sha256::new()))
    }
}

struct Sha1Engine(sha1::Sha1);

impl HashEngine for Sha1Engine {
    fn update(&mut self, data: &[u8]) {
        use sha1::Digest;
        self.0.update(data);
    }
    fn finalize(&mut self) -> String {
        use sha1::Digest;
        hex::encode(self.0.finalize_reset())
    }
    fn box_clone(&self) -> Box<dyn HashEngine> {
        Box::new(Sha1Engine(sha1::Sha1::new()))
    }
}

impl ChecksumValidator {
    pub fn new_md5() -> Self {
        ChecksumValidator {
            algorithm: HashAlgorithm::Md5,
            hasher: Some(Box::new(Md5Engine(md5::Md5::new()))),
            expected: None,
            result: None,
        }
    }

    pub fn new_sha1() -> Self {
        ChecksumValidator {
            algorithm: HashAlgorithm::Sha1,
            hasher: Some(Box::new(Sha1Engine(sha1::Sha1::new()))),
            expected: None,
            result: None,
        }
    }

    pub fn new_sha256() -> Self {
        ChecksumValidator {
            algorithm: HashAlgorithm::Sha256,
            hasher: Some(Box::new(Sha256Engine(sha2::Sha256::new()))),
            expected: None,
            result: None,
        }
    }

    pub fn with_expected(mut self, expected: String) -> Self {
        self.expected = Some(expected);
        self
    }

    pub fn digest(&self) -> Option<&str> {
        self.result.as_deref()
    }
}

impl Validator for ChecksumValidator {
    fn init(&mut self) {
        self.hasher = Some(match self.algorithm {
            HashAlgorithm::Md5 => Box::new(Md5Engine(md5::Md5::new())),
            HashAlgorithm::Sha1 => Box::new(Sha1Engine(sha1::Sha1::new())),
            HashAlgorithm::Sha256 => Box::new(Sha256Engine(sha2::Sha256::new())),
        });
        self.result = None;
    }

    fn write(&mut self, data: &[u8]) {
        if let Some(ref mut hasher) = self.hasher {
            hasher.update(data);
        }
    }

    fn abort(&mut self) {
        self.hasher = None;
        self.result = None;
    }

    fn validate(&mut self) -> bool {
        let digest = match self.hasher.take() {
            Some(mut h) => h.finalize(),
            None => return false,
        };
        self.result = Some(digest.clone());
        match &self.expected {
            // Use constant-time comparison to prevent timing attacks when
            // verifying download checksums against attacker-controlled data.
            Some(expected) => constant_time_eq(&digest, expected),
            None => true,
        }
    }
}

/// Constant-time byte comparison to prevent timing side-channel attacks.
/// Compares two strings in O(n) time regardless of where they differ.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}
pub struct JsonParsingValidator<T: serde::de::DeserializeOwned> {
    data: Vec<u8>,
    result: Option<T>,
}

impl<T: serde::de::DeserializeOwned> JsonParsingValidator<T> {
    pub fn new() -> Self {
        JsonParsingValidator {
            data: Vec::new(),
            result: None,
        }
    }

    pub fn result(&self) -> Option<&T> {
        self.result.as_ref()
    }
}

impl<T: serde::de::DeserializeOwned> Default for JsonParsingValidator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: serde::de::DeserializeOwned + Send + Sync> Validator for JsonParsingValidator<T> {
    fn init(&mut self) {
        self.data.clear();
        self.result = None;
    }

    fn write(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    fn abort(&mut self) {
        self.data.clear();
        self.result = None;
    }

    fn validate(&mut self) -> bool {
        match serde_json::from_slice::<T>(&self.data) {
            Ok(val) => {
                self.result = Some(val);
                true
            }
            Err(e) => {
                tracing::warn!("JSON parse error: {}", e);
                false
            }
        }
    }
}
