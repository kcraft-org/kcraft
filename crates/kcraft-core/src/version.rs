use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Version {
    raw: String,
    sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Section {
    num_valid: bool,
    num_part: i64,
    string_part: String,
    full_string: String,
}

impl Section {
    fn parse(s: &str) -> Self {
        let trimmed = s.trim();
        let mut num_valid = false;
        let mut digits_end = 0;

        for (i, ch) in trimmed.char_indices() {
            if ch.is_ascii_digit() {
                digits_end = i + 1;
                num_valid = true;
            } else {
                break;
            }
        }

        let num_part: i64 = if num_valid {
            trimmed[..digits_end].parse().unwrap_or(0)
        } else {
            0
        };

        if num_valid {
            Section {
                num_valid: true,
                num_part,
                string_part: trimmed[digits_end..].to_string(),
                full_string: trimmed.to_string(),
            }
        } else {
            Section {
                num_valid: false,
                num_part: 0,
                string_part: String::new(),
                full_string: trimmed.to_string(),
            }
        }
    }
}

impl Ord for Section {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.num_valid && other.num_valid {
            match self.num_part.cmp(&other.num_part) {
                Ordering::Equal => self.string_part.cmp(&other.string_part),
                other => other,
            }
        } else {
            self.full_string.cmp(&other.full_string)
        }
    }
}

impl PartialOrd for Section {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Version {
    pub fn new(raw: String) -> Self {
        let sections = raw
            .split('.')
            .map(Section::parse)
            .collect();
        Version { raw, sections }
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    pub fn parse(input: &str) -> Result<Self, String> {
        if input.is_empty() {
            return Err("Empty version string".to_string());
        }
        Ok(Version::new(input.to_string()))
    }


}

impl FromStr for Version {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::parse(s)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let max_len = self.sections.len().max(other.sections.len());
        for i in 0..max_len {
            let a = self.sections.get(i).cloned().unwrap_or_else(|| Section {
                num_valid: true,
                num_part: 0,
                string_part: String::new(),
                full_string: "0".to_string(),
            });
            let b = other.sections.get(i).cloned().unwrap_or_else(|| Section {
                num_valid: true,
                num_part: 0,
                string_part: String::new(),
                full_string: "0".to_string(),
            });
            match a.cmp(&b) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.19.2").unwrap();
        assert_eq!(v.raw(), "1.19.2");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.19.2").unwrap();
        let v2 = Version::parse("1.20").unwrap();
        let v3 = Version::parse("1.19.2").unwrap();
        assert!(v1 < v2);
        assert!(v1 == v3);
        assert!(v2 > v1);
    }

    #[test]
    fn test_version_comparison_complex() {
        let v1 = Version::parse("1.7.10").unwrap();
        let v2 = Version::parse("1.8").unwrap();
        let v3 = Version::parse("1.7.2").unwrap();
        assert!(v1 < v2);
        assert!(v3 < v1);
        assert!(v3 < v2);
    }

    #[test]
    fn test_version_with_suffix() {
        let v1 = Version::parse("1.19-rc1").unwrap();
        let v2 = Version::parse("1.19").unwrap();
        let v3 = Version::parse("1.19-snapshot").unwrap();
        assert!(v1 > v2);
        assert!(v3 > v1);
    }

    #[test]
    fn test_empty_fails() {
        assert!(Version::parse("").is_err());
    }
}
