use celestia_types::{nmt::Namespace, nmt::NamespaceProof, AppVersion, Blob};
use nmt_rs::{simple_merkle::proof::Proof, NamespacedHash, TmSha2Hasher};
use rsp_client_executor::{
    io::ClientExecutorInput, ChainVariant, ClientExecutor, EthereumVariant, CHAIN_ID_ETH_MAINNET,
    CHAIN_ID_LINEA_MAINNET, CHAIN_ID_OP_MAINNET,
};
use serde::{Deserialize, Serialize};
use tendermint::Hash as TmHash;

#[derive(Serialize, Deserialize)]
pub struct BlevmInput {
    pub input: ClientExecutorInput,
    pub namespace: Namespace,
    pub celestia_header_hash: TmHash,
    pub data_hash: TmHash,
    pub proof_data_hash_to_celestia_hash: Proof<TmSha2Hasher>,
    pub row_root_multiproof: Proof<TmSha2Hasher>,
    pub nmt_multiproofs: Vec<NamespaceProof>,
    pub row_roots: Vec<NamespacedHash<29>>,
}

#[derive(Serialize, Deserialize)]
pub struct BlevmOutput {
    pub blob_commitment: [u8; 32],
    pub header_hash: [u8; 32],
    pub prev_header_hash: [u8; 32],
    pub height: u64,
    pub gas_used: u64,
    pub beneficiary: [u8; 20],
    pub state_root: [u8; 32],
    pub celestia_header_hash: [u8; 32],
}

#[derive(Serialize, Deserialize)]
pub struct BlevmAggOutput {
    pub newest_header_hash: [u8; 32],
    pub oldest_header_hash: [u8; 32],
    pub celestia_header_hashes: Vec<[u8; 32]>,
    pub newest_state_root: [u8; 32],
    pub newest_height: u64,
}
