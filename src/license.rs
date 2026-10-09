//! Trial / license client for the NexusCard license server.
//!
//! Only an anonymous machine hash is sent. Applying a card design asks the server for
//! permission first; a failed run gives the use back.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const DEFAULT_API: &str = "https://scalvache.shop/productos/NexusCard/api/v1";

/// Ed25519 public key of the production license server (base64, 32 bytes).
/// It matches `GET /api/v1/public-key`. A different key can be supplied through the
/// NEXUSCARD_PUBLIC_KEY environment variable to test against a local server.
const SERVER_PUBLIC_KEY_B64: &str = "+5lELhy4TKAKv57rqa+m6wuv0gJmV68qarFhSzLuNsI=";

pub fn api_base() -> String {
    std::env::var("NEXUSCARD_API").unwrap_or_else(|_| DEFAULT_API.to_string())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Price {
    pub amount: f64,
    pub currency: String,
    pub display: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LicenseInfo {
    pub key: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Status {
    #[serde(default)]
    pub plan: String,
    #[serde(default)]
    pub uses_used: u32,
    #[serde(default)]
    pub uses_limit: u32,
    #[serde(default)]
    pub uses_left: Option<u32>,
    #[serde(default)]
    pub license: Option<LicenseInfo>,
    #[serde(default)]
    pub price: Option<Price>,
}

impl Status {
    pub fn is_pro(&self) -> bool {
        self.plan == "pro"
    }
}

pub enum Authorization {
    Granted { event_id: String, status: Status },
    LimitReached(Status),
}

/// Anonymous, stable machine identifier: SHA-256 of the Windows MachineGuid.
pub fn machine_id() -> Result<String> {
    Ok(hash_machine_guid(&machine_guid()?))
}

fn hash_machine_guid(guid: &str) -> String {
    let digest = Sha256::digest(format!("nexuscard|{}", guid.trim().to_lowercase()).as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(windows)]
fn machine_guid() -> Result<String> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY};
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Cryptography", KEY_READ | KEY_WOW64_64KEY)
        .context("Could not read the Windows cryptography registry key")?;
    key.get_value::<String, _>("MachineGuid")
        .context("Could not read MachineGuid")
}

#[cfg(not(windows))]
fn machine_guid() -> Result<String> {
    bail!("Machine identification is only available on Windows")
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(20)))
        .build()
        .into()
}

fn post(path: &str, body: Value) -> Result<(u16, Value)> {
    let url = format!("{}{path}", api_base());
    let mut response = agent()
        .post(&url)
        .header("User-Agent", concat!("NexusCard/", env!("CARGO_PKG_VERSION")))
        .send_json(&body)
        .with_context(|| format!("Could not reach the license server ({url})"))?;
    let status = response.status().as_u16();
    let value = response.body_mut().read_json::<Value>().unwrap_or(Value::Null);
    Ok((status, value))
}

fn error_code(value: &Value) -> String {
    value.get("error").and_then(Value::as_str).unwrap_or("unknown").to_string()
}

pub fn fetch_status(machine: &str) -> Result<Status> {
    let (code, body) = post(
        "/device/status",
        json!({ "machine_id": machine, "app_version": env!("CARGO_PKG_VERSION") }),
    )?;
    if code != 200 {
        bail!("License server answered {code}: {}", error_code(&body));
    }
    Ok(serde_json::from_value(body)?)
}

/// Asks permission to apply a design. On success the use is reserved server-side.
pub fn authorize(machine: &str) -> Result<Authorization> {
    let (code, body) = post(
        "/flash/authorize",
        json!({ "machine_id": machine, "app_version": env!("CARGO_PKG_VERSION") }),
    )?;
    match code {
        200 => {
            let token = body.get("token").and_then(Value::as_str).unwrap_or_default();
            verify_token(token, machine)?;
            let event_id = body
                .get("event_id")
                .and_then(Value::as_str)
                .context("The license server did not return an event id")?
                .to_string();
            Ok(Authorization::Granted { event_id, status: serde_json::from_value(body)? })
        }
        402 => Ok(Authorization::LimitReached(serde_json::from_value(body)?)),
        other => bail!("License server answered {other}: {}", error_code(&body)),
    }
}

/// Reports the outcome; a failed run refunds the reserved use.
pub fn complete(machine: &str, event_id: &str, success: bool) -> Result<Status> {
    let (code, body) = post(
        "/flash/complete",
        json!({ "machine_id": machine, "event_id": event_id, "success": success }),
    )?;
    if code != 200 {
        bail!("License server answered {code}: {}", error_code(&body));
    }
    Ok(serde_json::from_value(body)?)
}

/// Creates a payment and returns the hosted checkout URL to open in the browser.
pub fn checkout(machine: &str) -> Result<String> {
    let (code, body) = post("/checkout", json!({ "machine_id": machine }))?;
    match code {
        200 => body
            .get("url")
            .and_then(Value::as_str)
            .map(str::to_string)
            .context("The license server did not return a payment URL"),
        409 => bail!("Este equipo ya tiene la licencia completa."),
        _ => bail!("No se pudo iniciar el pago ({code}: {})", error_code(&body)),
    }
}

pub fn activate(machine: &str, key: &str) -> Result<Status> {
    let (code, body) = post("/license/activate", json!({ "machine_id": machine, "key": key.trim() }))?;
    match code {
        200 => Ok(serde_json::from_value(body)?),
        404 => bail!("La clave no es válida."),
        409 => bail!("Esta clave ya se usa en el máximo de equipos permitidos."),
        _ => bail!("No se pudo activar la licencia ({code}: {})", error_code(&body)),
    }
}

/// Checks the Ed25519 signature, expiry and machine of an authorization token.
fn verify_token(token: &str, machine: &str) -> Result<()> {
    let key = std::env::var("NEXUSCARD_PUBLIC_KEY").unwrap_or_else(|_| SERVER_PUBLIC_KEY_B64.to_string());
    verify_token_with(&key, token, machine, now_unix())
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn verify_token_with(public_key_b64: &str, token: &str, machine: &str, now: u64) -> Result<()> {
    let (body, signature) = token.split_once('.').context("Malformed authorization token")?;
    let key_bytes: [u8; 32] = STANDARD
        .decode(public_key_b64)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Public key must be 32 bytes"))?;
    let key = VerifyingKey::from_bytes(&key_bytes)?;
    let signature = Signature::from_slice(&URL_SAFE_NO_PAD.decode(signature)?)?;
    key.verify(body.as_bytes(), &signature)
        .map_err(|_| anyhow::anyhow!("Authorization token signature is invalid"))?;

    let payload: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(body)?)?;
    if payload.get("mid").and_then(Value::as_str) != Some(machine) {
        bail!("Authorization token was issued for another machine");
    }
    if payload.get("exp").and_then(Value::as_u64).is_none_or(|exp| exp < now) {
        bail!("Authorization token has expired");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn signed(payload: Value, seed: u8) -> (String, String) {
        let signing = SigningKey::from_bytes(&[seed; 32]);
        let body = URL_SAFE_NO_PAD.encode(payload.to_string());
        let signature = URL_SAFE_NO_PAD.encode(signing.sign(body.as_bytes()).to_bytes());
        let public = STANDARD.encode(signing.verifying_key().to_bytes());
        (format!("{body}.{signature}"), public)
    }

    #[test]
    fn machine_hash_is_stable_and_anonymous() {
        let a = hash_machine_guid("ABCD-1234");
        assert_eq!(a, hash_machine_guid(" abcd-1234 "));
        assert_eq!(a.len(), 64);
        assert!(!a.contains("abcd"));
        assert_ne!(a, hash_machine_guid("other"));
    }

    #[test]
    fn accepts_a_valid_token() {
        let (token, public) = signed(json!({ "mid": "m1", "exp": 2000 }), 7);
        assert!(verify_token_with(&public, &token, "m1", 1000).is_ok());
    }

    #[test]
    fn rejects_other_machine_expired_or_forged_tokens() {
        let (token, public) = signed(json!({ "mid": "m1", "exp": 2000 }), 7);
        assert!(verify_token_with(&public, &token, "m2", 1000).is_err());
        assert!(verify_token_with(&public, &token, "m1", 3000).is_err());

        let (forged, _) = signed(json!({ "mid": "m1", "exp": 2000 }), 9);
        assert!(verify_token_with(&public, &forged, "m1", 1000).is_err());

        let tampered = token.replacen('.', "x.", 1);
        assert!(verify_token_with(&public, &tampered, "m1", 1000).is_err());
        assert!(verify_token_with(&public, "garbage", "m1", 1000).is_err());
    }

    #[test]
    fn embedded_public_key_is_a_valid_ed25519_key() {
        let bytes = STANDARD.decode(SERVER_PUBLIC_KEY_B64).unwrap();
        assert_eq!(bytes.len(), 32);
        assert!(VerifyingKey::from_bytes(&bytes.try_into().unwrap()).is_ok());
    }

    #[test]
    fn status_parses_the_server_shape() {
        let status: Status = serde_json::from_value(json!({
            "plan": "free", "uses_used": 1, "uses_limit": 3, "uses_left": 2, "license": null,
            "price": { "amount": 10000, "currency": "COP", "display": "$10.000 COP" }
        }))
        .unwrap();
        assert!(!status.is_pro());
        assert_eq!(status.uses_left, Some(2));
        assert_eq!(status.price.unwrap().currency, "COP");
    }

    /// Needs a running server: NEXUSCARD_API=http://localhost:3300/productos/NexusCard/api/v1
    /// and NEXUSCARD_LIVE_KEY=<license key issued by the admin API>.
    /// Run with: cargo test live_roundtrip -- --ignored --nocapture
    #[test]
    #[ignore]
    fn live_roundtrip() {
        let machine = hash_machine_guid("live-test-machine");
        let status = fetch_status(&machine).unwrap();
        assert_eq!(status.plan, "free");

        let mut events = Vec::new();
        for _ in 0..3 {
            match authorize(&machine).unwrap() {
                Authorization::Granted { event_id, .. } => events.push(event_id),
                Authorization::LimitReached(_) => panic!("limit reached too early"),
            }
        }
        assert!(matches!(authorize(&machine).unwrap(), Authorization::LimitReached(_)));

        // A failed run refunds one use, so exactly one more authorization is possible.
        assert_eq!(complete(&machine, &events[0], false).unwrap().uses_left, Some(1));
        assert!(matches!(authorize(&machine).unwrap(), Authorization::Granted { .. }));
        assert!(matches!(authorize(&machine).unwrap(), Authorization::LimitReached(_)));

        let key = std::env::var("NEXUSCARD_LIVE_KEY").expect("NEXUSCARD_LIVE_KEY");
        assert!(activate(&machine, "NXC-NOPE-NOPE-NOPE").is_err());
        assert!(activate(&machine, &key).unwrap().is_pro());
        // Reinstall on another machine with the same key.
        let other = hash_machine_guid("another-pc");
        assert!(activate(&other, &key).unwrap().is_pro());
        assert!(matches!(authorize(&machine).unwrap(), Authorization::Granted { .. }));
    }
}
