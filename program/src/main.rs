#![no_main]
sp1_zkvm::entrypoint!(main);

use celestia_types::{nmt::Namespace, AppVersion, Blob};
use celestia_types::{
    nmt::{NamespaceProof, NamespacedHashExt},
    ExtendedHeader,
};
use nmt_rs::simple_merkle::tree::MerkleHash;
//use nmt_rs::{simple_merkle::proof::Proof, TmSha2Hasher};
use nmt_rs::{simple_merkle::proof::Proof, NamespacedHash, TmSha2Hasher};
use reth_primitives::Block;
use rsp_client_executor::{
    io::ClientExecutorInput, ChainVariant, ClientExecutor, EthereumVariant, CHAIN_ID_ETH_MAINNET,
    CHAIN_ID_LINEA_MAINNET, CHAIN_ID_OP_MAINNET,
};
use tendermint::Hash as TmHash;

pub fn main() {
    println!("cycle-tracker-start: cloning and deserializing inputs");
    let input: ClientExecutorInput = sp1_zkvm::io::read();
    let namespace: Namespace = sp1_zkvm::io::read();
    let header_hash: TmHash = sp1_zkvm::io::read();
    let data_hash_bytes: Vec<u8> = sp1_zkvm::io::read_vec();
    let proof_data_hash_to_celestia_hash: Proof<TmSha2Hasher> = sp1_zkvm::io::read();
    let row_root_multiproof: Proof<TmSha2Hasher> = sp1_zkvm::io::read();
    let nmt_multiproofs: Vec<NamespaceProof> = sp1_zkvm::io::read();
    let row_roots: Vec<NamespacedHash<29>> = sp1_zkvm::io::read();

    let block = input.current_block.clone();
    println!("cycle-tracker-end: cloning and deserializing inputs");

    // 1. Verify that the data root goes into the Celestia block hash
    println!("cycle-tracker-start: verify data root");
    let hasher = TmSha2Hasher {};
    proof_data_hash_to_celestia_hash
        .verify_range(
            header_hash.as_bytes().try_into().unwrap(),
            &[hasher.hash_leaf(&data_hash_bytes)],
        )
        .unwrap();
    println!("cycle-tracker-end: verify data root");

    println!("cycle-tracker-start: serializing EVM block");
    let block_bytes = bincode::serialize(&block).unwrap();
    println!("cycle-tracker-end: serializing EVM block");

    println!("cycle-tracker-start: creating Blob");
    let blob = Blob::new(namespace, block_bytes, AppVersion::V3).unwrap();
    println!("{}", hex::encode(blob.commitment.0));
    println!("cycle-tracker-end: creating Blob");

    println!("cycle-tracker-start: blob to shares");
    let shares = blob.to_shares().unwrap();
    println!("cycle-tracker-end: blob to shares");

    // Verify NMT multiproofs of blob shares into row roots
    let mut start = 0;
    println!("shares len {}", &shares.len());
    println!("nmt multiproofs len {}", &nmt_multiproofs.len());
    for i in 0..nmt_multiproofs.len() {
        let proof = &nmt_multiproofs[i];
        let end = start + (proof.end_idx() as usize - proof.start_idx() as usize);
        proof
            .verify_range(&row_roots[i], &shares[start..end], namespace.into())
            .expect("NMT multiproof into row root failed verification"); // Panicking should prevent an invalid proof from being generated
        start = end;
    }

    // Verify row root inclusion into data root
    let tm_hasher = TmSha2Hasher {};
    let blob_row_root_hashes: Vec<[u8; 32]> = row_roots
        .iter()
        .map(|root| tm_hasher.hash_leaf(&root.to_array()))
        .collect();
    let result = row_root_multiproof.verify_range(
        &data_hash_bytes
            .try_into()
            .expect("we already checked, this should be fine"),
        &blob_row_root_hashes,
    );

    // Commit the Celestia blob commitment for this block
    let blob_commitment = blob.commitment.0;
    sp1_zkvm::io::commit_slice(&blob_commitment);

    // Execute the block
    println!("cycle-tracker-start: executing EVM block");
    let executor = ClientExecutor;
    let header = executor.execute::<EthereumVariant>(input).unwrap(); // panicking should prevent a proof of invalid execution from being generated
    println!("cycle-tracker-end: executing EVM block");

    // Commit the header hash
    println!(
        "cycle-tracker-start: hashing the block header, and commiting its fields as public values"
    );
    let header_hash: Vec<u8> = header.hash_slow().to_vec();
    sp1_zkvm::io::commit_slice(&header_hash);

    // Commit the prev header hash, so we can form a blockchain
    let prev_header_hash: Vec<u8> = header.parent_hash.to_vec();
    sp1_zkvm::io::commit_slice(&prev_header_hash);

    let height: u64 = header.number;
    sp1_zkvm::io::commit(&height);

    let gas_used: u64 = header.gas_used;
    sp1_zkvm::io::commit(&gas_used);

    // beneficiary is the address of the person who collects the fees
    // usually the proposer
    let beneficiary = header.beneficiary.as_slice();
    sp1_zkvm::io::commit_slice(&beneficiary);

    let state_root = header.state_root.as_slice();
    sp1_zkvm::io::commit_slice(&state_root);
    println!(
        "cycle-tracker-end: hashing the block header, and commiting its fields as public values"
    );
}
