mod buffer;
use buffer::Buffer;

use sha2::{Digest, Sha256};

// Verification key of blevm-mock (Dec 22 2024)
// 0x001a3232969a5caac2de9a566ceee00641853a058b1ce1004ab4869f75a8dc59

const BLEVM_MOCK_VERIFICATION_KEY: [u32; 8] = [
    0x001a3232, 0x969a5caa, 0xc2de9a56, 0x6ceee006, 0x41853a05, 0x8b1ce100, 0x4ab4869f, 0x75a8dc59,
];

pub fn main() {
    let public_values1: Vec<u8> = sp1_zkvm::io::read();
    let public_values2: Vec<u8> = sp1_zkvm::io::read();

    let proof1_values_hash = Sha256::digest(&public_values1);
    let proof2_values_hash = Sha256::digest(&public_values2);

    sp1_zkvm::lib::verify::verify_sp1_proof(
        &BLEVM_MOCK_VERIFICATION_KEY,
        &proof1_values_hash.into(),
    );
    sp1_zkvm::lib::verify::verify_sp1_proof(
        &BLEVM_MOCK_VERIFICATION_KEY,
        &proof2_values_hash.into(),
    );

    let buffer1 = Buffer::from(&public_values1);
    let buffer2 = Buffer::from(&public_values2);
}
