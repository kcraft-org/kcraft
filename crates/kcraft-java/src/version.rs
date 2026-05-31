use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JavaVersion {
    raw: String,
    major: i32,
    minor: i32,
    security: i32,
    parseable: bool,
    prerelease: String,
}

impl JavaVersion {
    pub fn new(s: &str) -> Self {
        let mut v = JavaVersion {
            raw: s.to_string(),
            major: 0,
            minor: 0,
            security: 0,
            parseable: false,
            prerelease: String::new(),
        };
        v.parse_internal();
        v
    }

    fn parse_internal(&mut self) {
        let pattern = if self.raw.starts_with("1.") {
            regex::Regex::new(
                r"1[.](?P<major>[0-9]+)([.](?P<minor>[0-9]+))?(_(?P<security>[0-9]+)?)?(-(?P<prerelease>[a-zA-Z0-9]+))?"
            ).unwrap()
        } else {
            regex::Regex::new(
                r"(?P<major>[0-9]+)([.](?P<minor>[0-9]+))?([.](?P<security>[0-9]+))?(-(?P<prerelease>[a-zA-Z0-9]+))?"
            ).unwrap()
        };

        if let Some(caps) = pattern.captures(&self.raw) {
            self.parseable = true;
            self.major = caps
                .name("major")
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            self.minor = caps
                .name("minor")
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            self.security = caps
                .name("security")
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            self.prerelease = caps
                .name("prerelease")
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
        }
    }

    pub fn major(&self) -> i32 {
        self.major
    }
    pub fn minor(&self) -> i32 {
        self.minor
    }
    pub fn security(&self) -> i32 {
        self.security
    }
    pub fn raw(&self) -> &str {
        &self.raw
    }
    pub fn is_parseable(&self) -> bool {
        self.parseable
    }

    pub fn requires_perm_gen(&self) -> bool {
        if self.parseable {
            self.major < 8
        } else {
            true
        }
    }
}

impl From<String> for JavaVersion {
    fn from(s: String) -> Self {
        JavaVersion::new(&s)
    }
}

impl From<&str> for JavaVersion {
    fn from(s: &str) -> Self {
        JavaVersion::new(s)
    }
}

impl fmt::Display for JavaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl Ord for JavaVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        if !self.parseable || !other.parseable {
            return self.raw.cmp(&other.raw);
        }

        let disc = |major: i32| -> i32 {
            if major > 8 {
                -major
            } else {
                major
            }
        };

        match disc(self.major).cmp(&disc(other.major)) {
            Ordering::Equal => {}
            other => return other,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            other => return other,
        }
        match self.security.cmp(&other.security) {
            Ordering::Equal => {}
            other => return other,
        }

        let self_pre = !self.prerelease.is_empty();
        let other_pre = !other.prerelease.is_empty();

        match (self_pre, other_pre) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (true, true) => self.prerelease.cmp(&other.prerelease),
            (false, false) => Ordering::Equal,
        }
    }
}

impl PartialOrd for JavaVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_version_parse_old() {
        let v = JavaVersion::new("1.8.0_51");
        assert_eq!(v.major(), 8);
        assert_eq!(v.minor(), 0);
        assert_eq!(v.security(), 51);
        assert!(v.is_parseable());
    }

    #[test]
    fn test_java_version_parse_new() {
        let v = JavaVersion::new("17.0.1");
        assert_eq!(v.major(), 17);
        assert_eq!(v.minor(), 0);
        assert_eq!(v.security(), 1);
        assert!(v.is_parseable());
    }

    #[test]
    fn test_java_version_compare() {
        let v1 = JavaVersion::new("1.8.0_51");
        let v2 = JavaVersion::new("1.8.0_101");
        let v3 = JavaVersion::new("17.0.1");
        let v4 = JavaVersion::new("11.0.2");
        let v5 = JavaVersion::new("1.8.0_51");

        assert!(v1 < v2);
        assert!(v2 > v1);
        assert!(v1 == v5);
        assert!(v3 < v4); // 17 < 11 (because major > 8 is negated)
    }

    #[test]
    fn test_java_requires_perm_gen() {
        let v1 = JavaVersion::new("1.7.0");
        let v2 = JavaVersion::new("1.8.0");
        let v3 = JavaVersion::new("17.0.1");

        assert!(v1.requires_perm_gen());
        assert!(!v2.requires_perm_gen());
        assert!(!v3.requires_perm_gen());
    }
}
