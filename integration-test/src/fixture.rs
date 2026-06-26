//! Fixture loading, normalized to canonical schema-prefixed SSZ bytes.
//!
//! Two on-disk layouts auto-detect per file, both collapsing to the same
//! `(input_bytes, output_bytes)` pair. RPC fixtures carry top-level
//! `statelessInputBytes`/`statelessOutputBytes`. EEST `blockchain_test` fixtures
//! carry the same fields per block. `.zst` files are zstd-decompressed.

use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, de::Error as _};

/// Canonical schema-prefixed SSZ input bytes and the expected canonical SSZ
/// `StatelessValidationResult` output bytes.
pub struct Fixture {
    pub name: String,
    pub input_bytes: Vec<u8>,
    pub output_bytes: Vec<u8>,
}

/// RPC layout from `witness-generator-spec-cli`, one block per file.
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

/// Minimal projection of a single EEST block, testable only when both stateless
/// byte fields are present. The `successful_validation` bit is already encoded in
/// `stateless_output_bytes`, so `expectException` is unread.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EestBlock {
    #[serde(default, deserialize_with = "de_hex_opt")]
    stateless_input_bytes: Option<Vec<u8>>,
    #[serde(default, deserialize_with = "de_hex_opt")]
    stateless_output_bytes: Option<Vec<u8>>,
}

/// Loads every fixture from one JSON file, auto-detecting the on-disk layout.
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

    if value.get("statelessInputBytes").is_some() {
        let rpc: RpcFixture = serde_json::from_value(value)?;
        return Ok(vec![Fixture {
            name: format!("rpc-{}-{}", rpc.network, rpc.block_number),
            input_bytes: rpc.stateless_input_bytes,
            output_bytes: rpc.stateless_output_bytes,
        }]);
    }

    let tests: EestFixture =
        serde_json::from_value(value).with_context(|| format!("parse EEST {}", path.display()))?;
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

fn parse_hex(text: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(text.strip_prefix("0x").unwrap_or(text))
}

fn de_hex<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    parse_hex(&String::deserialize(deserializer)?).map_err(D::Error::custom)
}

fn de_hex_opt<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|text| parse_hex(&text))
        .transpose()
        .map_err(D::Error::custom)
}
