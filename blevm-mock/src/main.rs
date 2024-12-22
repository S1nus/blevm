/*
    This is a mock of the blevm program
    Unfortuantely it can't mock the exact inputs, but it can mock the exact outputs
*/
#![no_main]
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let blob_commitment = sp1_zkvm::io::read::<Vec<u8>>();
    let header_hash = sp1_zkvm::io::read::<Vec<u8>>();
    let prev_header_hash = sp1_zkvm::io::read::<Vec<u8>>();
    let height = sp1_zkvm::io::read::<u64>();
    let gas_used = sp1_zkvm::io::read::<u64>();
    let beneficiary = sp1_zkvm::io::read::<Vec<u8>>();
    let state_root = sp1_zkvm::io::read::<Vec<u8>>();
    let celestia_header_hash = sp1_zkvm::io::read::<Vec<u8>>();

    sp1_zkvm::io::commit_slice(&blob_commitment);
    sp1_zkvm::io::commit_slice(&header_hash);
    sp1_zkvm::io::commit_slice(&prev_header_hash);
    sp1_zkvm::io::commit(&height);
    sp1_zkvm::io::commit(&gas_used);
    sp1_zkvm::io::commit_slice(&beneficiary);
    sp1_zkvm::io::commit_slice(&state_root);
    sp1_zkvm::io::commit(&celestia_header_hash);
}
