//! Reads display information for a Wallet card from the connected iPhone:
//! the artwork currently installed and, when available, the card's last digits and network.

use std::path::PathBuf;

use anyhow::{Context, Result};
use image::DynamicImage;
use serde_json::Value;

use crate::afc::AfcClient;
use crate::device::{ActiveDeviceSession, ConnectionMode};
use crate::wallet_backup::find_device_card_hash;

const ART_CANDIDATES: [&str; 2] = [
    "cardBackgroundCombined@2x.png",
    "cardBackgroundCombined@3x.png",
];
const MAX_PASS_JSON_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, Default)]
pub struct CardInfo {
    pub hash: String,
    pub art_png: Option<Vec<u8>>,
    pub last4: Option<String>,
    pub network: Option<String>,
    /// Names found in the pass directory; written to the log to help diagnose unknown layouts.
    pub files: Vec<String>,
}

pub fn fetch_card_info(
    udid: &str,
    connection_mode: ConnectionMode,
    card_hash: &str,
) -> Result<CardInfo> {
    let session = ActiveDeviceSession::open(Some(udid), connection_mode)
        .context("Failed to open device session to read card info")?;
    let afc = AfcClient::new(&session).context("Failed to open AFC to read card info")?;

    let mut info = CardInfo {
        hash: card_hash.to_string(),
        ..CardInfo::default()
    };
    let Some(resolved) = find_device_card_hash(&afc, card_hash)? else {
        return Ok(info);
    };
    let dir = format!("/var/mobile/Library/Passes/Cards/{resolved}.pkpass");

    info.files = afc.list_directory(&dir).unwrap_or_default();

    for name in ART_CANDIDATES {
        let path = format!("{dir}/{name}");
        if afc.exists(&path) {
            if let Ok(bytes) = afc.read_file(&path) {
                info.art_png = Some(bytes);
                break;
            }
        }
    }

    let pass_json = format!("{dir}/pass.json");
    if afc.file_size(&pass_json).is_some_and(|size| size <= MAX_PASS_JSON_BYTES) {
        if let Ok(bytes) = afc.read_file(&pass_json) {
            if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                let (last4, network) = extract_card_details(&value);
                info.last4 = last4;
                info.network = network;
            }
        }
    }

    Ok(info)
}

fn thumbs_dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| r"C:\Users\Default\AppData\Local".to_string());
    PathBuf::from(base).join(crate::paths::APP_DIR).join("thumbs")
}

fn thumb_path(card_hash: &str) -> PathBuf {
    let safe: String = card_hash
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    thumbs_dir().join(format!("{safe}.png"))
}

/// Stores a small thumbnail of the card artwork so the card list can show it offline.
pub fn save_thumb(card_hash: &str, image: &DynamicImage) {
    let path = thumb_path(card_hash);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = image.thumbnail(192, 192).save_with_format(path, image::ImageFormat::Png);
}

pub fn load_thumb(card_hash: &str) -> Option<DynamicImage> {
    image::open(thumb_path(card_hash)).ok()
}

/// Looks through a pass.json for the account suffix and payment network.
/// Only the last digits are kept; nothing else from the pass is retained.
pub fn extract_card_details(root: &Value) -> (Option<String>, Option<String>) {
    let mut last4 = None;
    let mut network = None;
    walk(root, &mut last4, &mut network);
    if network.is_none() {
        network = network_from_text(root.get("organizationName").and_then(Value::as_str))
            .or_else(|| network_from_text(root.get("description").and_then(Value::as_str)));
    }
    (last4, network)
}

fn walk(value: &Value, last4: &mut Option<String>, network: &mut Option<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let lower = key.to_ascii_lowercase();
                if last4.is_none() && lower.contains("suffix") {
                    if let Some(digits) = child.as_str().and_then(clean_digits) {
                        *last4 = Some(digits);
                    }
                }
                if network.is_none() && (lower.contains("network") || lower == "brand") {
                    if let Some(name) = child.as_str().and_then(|text| network_from_text(Some(text))) {
                        *network = Some(name);
                    }
                }
                walk(child, last4, network);
            }
        }
        Value::Array(items) => {
            for child in items {
                walk(child, last4, network);
            }
        }
        _ => {}
    }
}

fn clean_digits(text: &str) -> Option<String> {
    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 4 {
        Some(digits[digits.len() - 4..].to_string())
    } else {
        None
    }
}

pub fn network_from_text(text: Option<&str>) -> Option<String> {
    let lower = text?.to_ascii_lowercase();
    let name = if lower.contains("american express") || lower.contains("amex") {
        "Amex"
    } else if lower.contains("master") {
        "Mastercard"
    } else if lower.contains("visa") {
        "Visa"
    } else if lower.contains("discover") {
        "Discover"
    } else {
        return None;
    };
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_suffix_and_network_from_nested_pass() {
        let pass = json!({
            "organizationName": "Example Bank",
            "paymentApplications": [{ "paymentNetwork": "MasterCard", "primaryAccountSuffix": "9921" }]
        });
        let (last4, network) = extract_card_details(&pass);
        assert_eq!(last4.as_deref(), Some("9921"));
        assert_eq!(network.as_deref(), Some("Mastercard"));
    }

    #[test]
    fn keeps_only_last_four_digits() {
        assert_eq!(clean_digits("1234567890123456").as_deref(), Some("3456"));
        assert_eq!(clean_digits("12"), None);
    }

    #[test]
    fn network_falls_back_to_organization_name() {
        let pass = json!({ "organizationName": "American Express" });
        assert_eq!(extract_card_details(&pass).1.as_deref(), Some("Amex"));
    }
}
