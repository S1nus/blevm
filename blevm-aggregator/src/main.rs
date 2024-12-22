#![no_main]
sp1_zkvm::entrypoint!(main);

mod buffer;
use buffer::Buffer;

// Verification key of blevm-mock (Dec 22 2024)
// 0x001a3232969a5caac2de9a566ceee00641853a058b1ce1004ab4869f75a8dc59

const BLEVM_MOCK_VERIFICATION_KEY: &[u8] =
    hex::decode("001a3232969a5caac2de9a566ceee00641853a058b1ce1004ab4869f75a8dc59").unwrap();

pub fn main() {}
