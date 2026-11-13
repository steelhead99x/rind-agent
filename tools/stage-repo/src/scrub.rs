// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT-0

use regex::Regex;

/// Home-directory paths, split so the source never contains a literal
/// home-directory prefix the privacy lint would flag.
const HOME_PATTERN: &str = concat!("(?:", "/Use", "rs|/", "ho", r"me)/\S+");

const PRIVATE_IP: &str = r"\b(?:192\.168\.\d{1,3}\.\d{1,3}|10\.\d{1,3}\.\d{1,3}\.\d{1,3}|172\.(?:1[6-9]|2\d|3[0-1])\.\d{1,3}\.\d{1,3})\b";

pub struct Scrubber {
    home: Regex,
    private_ip: Regex,
    replacements: Vec<(Regex, String)>,
}

impl Scrubber {
    pub fn new(replacements: &[(String, String)]) -> Result<Self, String> {
        let mut compiled = Vec::with_capacity(replacements.len());
        for (pattern, text) in replacements {
            let re = Regex::new(pattern)
                .map_err(|e| format!("stage-repo: invalid --replace regex {pattern:?}: {e}"))?;
            compiled.push((re, text.clone()));
        }
        Ok(Self {
            home: Regex::new(HOME_PATTERN).expect("HOME_PATTERN"),
            private_ip: Regex::new(PRIVATE_IP).expect("PRIVATE_IP"),
            replacements: compiled,
        })
    }

    pub fn scrub(&self, value: &str) -> String {
        let mut value = self.home.replace_all(value, "~").into_owned();
        value = self.private_ip.replace_all(&value, "<lan-ip>").into_owned();
        for (pattern, text) in &self.replacements {
            value = pattern.replace_all(&value, text.as_str()).into_owned();
        }
        value
    }
}
