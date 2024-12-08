#![no_main]
sp1_zkvm::entrypoint!(main);

use celestia_types::{nmt::Namespace, AppVersion, Blob};
use reth_primitives::Block;
use rsp_client_executor::{
    io::ClientExecutorInput, ChainVariant, ClientExecutor, EthereumVariant, CHAIN_ID_ETH_MAINNET,
    CHAIN_ID_LINEA_MAINNET, CHAIN_ID_OP_MAINNET,
};

pub fn main() {
    println!("cycle-tracker-start: cloning and deserializing inputs");
    let input: ClientExecutorInput = sp1_zkvm::io::read();
    let namespace: Namespace = sp1_zkvm::io::read();

    // The clone here is not ideal
    let block = input.current_block.clone();
    println!("cycle-tracker-end: cloning and deserializing inputs");

    println!("cycle-tracker-start: serializing EVM block");
    let block_bytes = bincode::serialize(&block).unwrap();
    println!("cycle-tracker-end: serializing EVM block");
    println!("cycle-tracker-start: creating Blob");
    let blob = Blob::new(namespace, block_bytes, AppVersion::V3).unwrap();
    println!("cycle-tracker-end: creating Blob");

    // Commit the Celestia blob commitment for this block
    let blob_commitment = blob.commitment.0;
    sp1_zkvm::io::commit_slice(&blob_commitment);

    // Execute the block
    println!("cycle-tracker-start: executing EVM block");
    let executor = ClientExecutor;
    let header = executor.execute::<EthereumVariant>(input).unwrap(); // unwrap will prevent a proof of invalid execution from being generated
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
