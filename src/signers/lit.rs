use alloy::primitives::{B256, Bytes, ChainId};
use eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

use crate::signers::FunderSigner;
use crate::types::Transfer;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(10);

/// Lit Protocol-based funder signer implementation.
///
/// Uses Lit Actions to generate funder signatures via an HTTP server that manages
/// PKP authentication and session signatures.
#[derive(Debug, Clone)]
pub struct LitFunderSigner {
    endpoint: Url,
    ipfs_cid: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LitRequest {
    ipfs_cid: String,
    js_params: Params,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Params {
    intent_digest: B256,
    transfers: Vec<Transfer>,
    escrow_and_chain_ids: Vec<EscrowAndChainId>,
    destination_chain_id: ChainId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EscrowAndChainId {
    escrow_id: B256,
    chain_id: ChainId,
}

#[derive(Debug, Deserialize)]
struct LitResponse {
    signatures: Signatures,
}

#[derive(Debug, Deserialize)]
struct Signatures {
    sig: SignatureData,
}

#[derive(Debug, Deserialize)]
struct SignatureData {
    signature: String,
}

impl LitFunderSigner {
    /// Creates a new Lit Actions funder signer.
    pub fn new(endpoint: Url, ipfs_cid: String) -> Self {
        Self { endpoint, ipfs_cid, client: reqwest::Client::new() }
    }

    async fn sign_funding_request(
        &self,
        intent_digest: B256,
        transfers: Vec<Transfer>,
        escrow_and_chain_ids: Vec<(B256, ChainId)>,
        destination_chain_id: ChainId,
    ) -> Result<Bytes> {
        let params = Params {
            intent_digest,
            transfers,
            escrow_and_chain_ids: escrow_and_chain_ids
                .into_iter()
                .map(|(escrow_id, chain_id)| EscrowAndChainId { escrow_id, chain_id })
                .collect(),
            destination_chain_id,
        };

        let request = LitRequest { ipfs_cid: self.ipfs_cid.clone(), js_params: params };

        let response = self
            .client
            .post(self.endpoint.clone())
            .json(&request)
            .send()
            .await
            .map_err(|e| eyre!("Failed to call Lit Actions endpoint: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return Err(eyre!("Lit Actions returned error status {}: {}", status, error_text));
        }

        let lit_response: LitResponse = response
            .json()
            .await
            .map_err(|e| eyre!("Failed to parse Lit Actions response: {}", e))?;

        let signature_hex = lit_response.signatures.sig.signature;
        let signature_bytes = Bytes::from_str(&signature_hex)
            .map_err(|e| eyre!("Failed to parse signature hex: {}", e))?;

        Ok(signature_bytes)
    }
}

#[async_trait::async_trait]
impl FunderSigner for LitFunderSigner {
    async fn sign_funding_request(
        &self,
        intent_digest: B256,
        transfers: Vec<Transfer>,
        escrow_and_chain_ids: Vec<(B256, ChainId)>,
        destination_chain_id: ChainId,
    ) -> Result<Bytes> {
        for i in 0..MAX_RETRIES {
            let signature = self
                .sign_funding_request(
                    intent_digest,
                    transfers.clone(),
                    escrow_and_chain_ids.clone(),
                    destination_chain_id,
                )
                .await;

            match (i, signature) {
                (_, Ok(signature)) => return Ok(signature),
                (i, Err(err)) if i == MAX_RETRIES - 1 => return Err(err),
                _ => sleep(RETRY_DELAY).await,
            }
        }

        unreachable!()
    }
}

impl std::fmt::Display for LitFunderSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LitFunderSigner(endpoint={}, ipfs_cid={})", self.endpoint, self.ipfs_cid)
    }
}
