use alloc::vec::Vec;

use alloy_genesis::ChainConfig;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use stateless::{ExecutionWitness, UncompressedPublicKey};

use crate::stateless::new_payload_request::NewPayloadRequest;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatelessValidatorRethInput {
    pub new_payload_request: NewPayloadRequest,
    pub witness: ExecutionWitness,
    #[serde_as(as = "alloy_genesis::serde_bincode_compat::ChainConfig<'_>")]
    pub chain_config: ChainConfig,
    pub public_keys: Vec<UncompressedPublicKey>,
}
