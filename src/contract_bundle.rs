//! This module provides simple utilities for loading and parsing `.contract` files in
//! context of `drink` tests.

use std::sync::Arc;

use contract_metadata::ContractMetadata;
use contract_transcode::ContractMessageTranscoder;

use anyhow::Result;

/// A struct representing the result of parsing a `.contract` bundle file.
///
/// It can be used with the following methods of the `Session` struct:
/// - `deploy_bundle`
/// - `deploy_bundle_and`
/// - `upload_bundle`
/// - `upload_bundle_and`
#[derive(Clone)]
pub struct ContractBundle {
    /// WASM blob of the contract
    pub wasm: Vec<u8>,
    /// Transcoder derived from the ABI/metadata
    pub transcoder: Arc<ContractMessageTranscoder>,
}

impl ContractBundle {
    /// Load and parse the information in a `.contract` bundle under `path`, producing a
    /// `ContractBundle` struct.
    pub fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let metadata: ContractMetadata = ContractMetadata::load(&path).map_err(|e| {
            anyhow::format_err!("Failed to load the contract file:\n{e:?}")
        })?;

        let ink_metadata = serde_json::from_value(serde_json::Value::Object(
            metadata.abi,
        ))
        .map_err(|e| {
            anyhow::format_err!("Failed to parse metadata from the contract file:\n{e:?}")
        })?;

        let transcoder = Arc::new(ContractMessageTranscoder::new(ink_metadata));

        let wasm = metadata
            .source
            .wasm
            .ok_or(anyhow::format_err!(
                "Failed to get the WASM blob from the contract file".to_string(),
            ))?
            .0;

        Ok(Self { wasm, transcoder })
    }
}
