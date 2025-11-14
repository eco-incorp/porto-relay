//! Relay signers.

mod r#dyn;
use alloy::primitives::{B256, Bytes, ChainId};
pub use r#dyn::DynSigner;

mod p256;
pub use p256::{P256Key, P256Signer};

mod webauthn;
pub use webauthn::WebAuthnSigner;

mod lit;
pub use lit::LitFunderSigner;

/// Trait for a [EIP-712] payload signer.
#[async_trait::async_trait]
pub trait Eip712PayLoadSigner: std::fmt::Debug + Send + Sync {
    /// Signs the [EIP-712] payload hash.
    ///
    /// Returns [`Bytes`].
    async fn sign_payload_hash(&self, payload_hash: B256) -> eyre::Result<Bytes>;
}

/// Trait for signing funding requests in cross-chain interop bundles.
///
/// Implementations provide signatures that authorize fund transfers from the funder's account
/// to destination chains. This is used for lazy signing where signatures are generated after
/// source chain escrow confirmation.
#[async_trait::async_trait]
pub trait FunderSigner: std::fmt::Debug + Send + Sync {
    /// Signs a funding request with all necessary cross-chain context.
    ///
    /// Returns the signature bytes that can be included in the intent to authorize fund transfers.
    async fn sign_funding_request(
        &self,
        intent_digest: B256,
        transfers: Vec<crate::types::Transfer>,
        escrow_and_chain_ids: Vec<(B256, ChainId)>,
        destination_chain_id: ChainId,
    ) -> eyre::Result<Bytes>;
}
