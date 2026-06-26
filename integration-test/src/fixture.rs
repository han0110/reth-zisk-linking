//! Fixture loading, normalized to canonical schema-prefixed SSZ bytes.
//!
//! Two on-disk layouts are supported and auto-detected per file, both collapsing
//! to the same `(input_bytes, output_bytes)` pair so every guest runs one path.
//! The RPC fixtures (`witness-generator-spec-cli`) carry top-level
//! `statelessInputBytes`/`statelessOutputBytes` in `*.json.zst` files. The EEST
//! `blockchain_test` fixtures carry the same fields per block, precomputed by
//! `ethereum/execution-specs`. Files ending in `.zst` are transparently
//! zstd-decompressed.

use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, de::Error as _};

/// A fixture normalized to canonical schema-prefixed SSZ input bytes and the
/// expected canonical SSZ output bytes, regardless of on-disk layout.
pub struct Fixture {
    /// Human-readable identifier.
    pub name: String,
    /// Canonical schema-prefixed SSZ input bytes fed to the guest.
    pub input_bytes: Vec<u8>,
    /// Expected SSZ `StatelessValidationResult` output bytes, as produced by the
    /// reference (execution-specs for EEST, witness-generator-spec-cli for RPC).
    pub output_bytes: Vec<u8>,
}

/// RPC layout from `witness-generator-spec-cli`. Canonical bytes precomputed at
/// the top level, one block per file.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RpcFixture {
    network: String,
    block_number: u64,
    #[serde(deserialize_with = "de_hex")]
    stateless_input_bytes: Vec<u8>,
    #[serde(deserialize_with = "de_hex")]
    stateless_output_bytes: Vec<u8>,
}

/// EEST `blockchain_test` file, a map from test identifier to its block list.
type EestFixture = BTreeMap<String, EestTest>;

/// Minimal projection of an EEST `blockchain_test` body.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EestTest {
    blocks: Vec<EestBlock>,
}

/// Minimal projection of a single EEST block. A block is testable only when both
/// stateless byte fields are present; the expected `successful_validation` bit is
/// already encoded in `stateless_output_bytes`, so `expectException` is not read.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EestBlock {
    #[serde(default, deserialize_with = "de_hex_opt")]
    stateless_input_bytes: Option<Vec<u8>>,
    #[serde(default, deserialize_with = "de_hex_opt")]
    stateless_output_bytes: Option<Vec<u8>>,
}

/// Loads every fixture from one JSON file, auto-detecting the on-disk layout and
/// transparently decompressing a `.json.zst` file.
pub fn load(path: &Path) -> Result<Vec<Fixture>> {
    let raw = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let bytes = if path.extension().is_some_and(|ext| ext == "zst") {
        zstd::stream::decode_all(raw.as_slice())
            .with_context(|| format!("decompress {}", path.display()))?
    } else {
        raw
    };
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?;

    // The RPC layout carries canonical bytes at the top level.
    if value.get("statelessInputBytes").is_some() {
        let rpc: RpcFixture = serde_json::from_value(value)?;
        return Ok(vec![Fixture {
            name: format!("rpc-{}-{}", rpc.network, rpc.block_number),
            input_bytes: rpc.stateless_input_bytes,
            output_bytes: rpc.stateless_output_bytes,
        }]);
    }

    // Otherwise an EEST blockchain_test file with precomputed bytes per block.
    let tests: EestFixture = serde_json::from_value(value)
        .with_context(|| format!("parse EEST {}", path.display()))?;
    Ok(tests
        .into_iter()
        .flat_map(|(test_id, test)| {
            test.blocks
                .into_iter()
                .enumerate()
                .filter_map(move |(idx, block)| {
                    let (input_bytes, output_bytes) = block
                        .stateless_input_bytes
                        .zip(block.stateless_output_bytes)?;
                    (!input_bytes.is_empty()).then(|| Fixture {
                        name: format!("{test_id}#block{idx}"),
                        input_bytes,
                        output_bytes,
                    })
                })
        })
        .collect())
}

/// Deserializes a `0x`-prefixed hex string into raw bytes.
fn de_hex<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    let text = text.strip_prefix("0x").unwrap_or(&text);
    hex::decode(text).map_err(D::Error::custom)
}

/// Deserializes an optional `0x`-prefixed hex string into raw bytes.
fn de_hex_opt<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|text| {
            let text = text.strip_prefix("0x").unwrap_or(&text);
            hex::decode(text).map_err(D::Error::custom)
        })
        .transpose()
}
