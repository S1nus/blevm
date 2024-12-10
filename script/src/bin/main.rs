use bincode;
use celestia_types::consts::appconsts::LATEST_VERSION;
use celestia_types::AppVersion;
use celestia_types::Blob;
use celestia_types::{
    nmt::{Namespace, NamespaceProof},
    ExtendedHeader,
};
use hex;
use nmt_rs::simple_merkle::tree::MerkleHash;
use nmt_rs::{
    simple_merkle::{db::MemDb, proof::Proof, tree::MerkleTree},
    TmSha2Hasher,
};
use tendermint::{hash::Algorithm, Hash as TmHash};
use tendermint_proto::{
    v0_37::{types::BlockId as RawBlockId, version::Consensus as RawConsensusVersion},
    Protobuf,
};

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
    let namespace: Namespace = Namespace::new_v0(&[15; 10]).unwrap(); // [15; 10] is just a randomly chosen valid namespace

    let header: ExtendedHeader =
        serde_json::from_str(&fs::read_to_string("header.json").unwrap()).unwrap();
    println!("header hash {:?}", header.hash());

    let hasher = TmSha2Hasher {};
    let mut header_field_tree: MerkleTree<MemDb<[u8; 32]>, TmSha2Hasher> =
        MerkleTree::with_hasher(hasher);

    let field_bytes = vec![
        Protobuf::<RawConsensusVersion>::encode_vec(header.header.version),
        header.header.chain_id.clone().encode_vec(),
        header.header.height.encode_vec(),
        header.header.time.encode_vec(),
        Protobuf::<RawBlockId>::encode_vec(header.header.last_block_id.unwrap_or_default()),
        header
            .header
            .last_commit_hash
            .unwrap_or_default()
            .encode_vec(),
        header.header.data_hash.unwrap_or_default().encode_vec(),
        header.header.validators_hash.encode_vec(),
        header.header.next_validators_hash.encode_vec(),
        header.header.consensus_hash.encode_vec(),
        header.header.app_hash.clone().encode_vec(),
        header
            .header
            .last_results_hash
            .unwrap_or_default()
            .encode_vec(),
        header.header.evidence_hash.unwrap_or_default().encode_vec(),
        header.header.proposer_address.encode_vec(),
    ];
    for leaf in field_bytes {
        header_field_tree.push_raw_leaf(&leaf);
    }
    let computed_header_hash = header_field_tree.root();
    let (data_hash_bytes_from_tree, data_hash_proof) = header_field_tree.get_index_with_proof(6);
    let data_hash_from_tree = TmHash::decode_vec(&data_hash_bytes_from_tree).unwrap();
    assert_eq!(
        data_hash_from_tree.as_bytes(),
        header.header.data_hash.unwrap().as_bytes()
    );
    assert_eq!(header.hash().as_ref(), header_field_tree.root());
    let hasher = TmSha2Hasher {};
    data_hash_proof
        .verify_range(
            &header_field_tree.root(),
            &[hasher.hash_leaf(&data_hash_bytes_from_tree)],
        )
        .unwrap();

    let row_root_multiproof: Proof<TmSha2Hasher> =
        serde_json::from_str(&fs::read_to_string("row_root_multiproof.json").unwrap()).unwrap();
    println!(
        "row root multiproof len {:?}",
        row_root_multiproof.siblings().len()
    );

    let nmt_multiproofs: Vec<NamespaceProof> =
        serde_json::from_str(&fs::read_to_string("nmt_multiproofs.json").unwrap()).unwrap();
    println!("num nmt multiproofs {:?}", nmt_multiproofs.len());

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&input1);
    stdin.write(&namespace);
    stdin.write(&header.header.hash());
    stdin.write_vec(data_hash_bytes_from_tree);
    stdin.write(&data_hash_proof);
    stdin.write(&row_root_multiproof);
    stdin.write(&nmt_multiproofs);

    let (output, report) = client.execute(BLEVM_ELF, stdin).run().unwrap();
}
