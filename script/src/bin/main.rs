use bincode;
use celestia_types::consts::appconsts::LATEST_VERSION;
use celestia_types::nmt::Namespace;
use celestia_types::AppVersion;
use celestia_types::Blob;
use hex;
use rsp_client_executor::{
    io::ClientExecutorInput, ChainVariant, ClientExecutor, EthereumVariant, CHAIN_ID_ETH_MAINNET,
    CHAIN_ID_LINEA_MAINNET, CHAIN_ID_OP_MAINNET,
};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use std::fs;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const BLEVM_ELF: &[u8] = include_elf!("blevm");

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    let f = fs::read("input/1/18884864.bin").expect("could not read file");

    // load the prev state + new block
    let input1: ClientExecutorInput = bincode::deserialize(&f).expect("could not deserialize");
    let namespace: Namespace = Namespace::new_v0(&[0; 28]).unwrap();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&input1);
    stdin.write(&namespace);

    let (output, report) = client.execute(BLEVM_ELF, stdin).run().unwrap();
}
