use bincode;
use celestia_types::consts::appconsts::LATEST_VERSION;
use celestia_types::AppVersion;
use celestia_types::{blob::Commitment, Blob, TxConfig};
use celestia_types::{
    nmt::{Namespace, NamespaceProof},
    ExtendedHeader,
};
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

use celestia_rpc::{BlobClient, Client};
use core::cmp::max;
use rsp_client_executor::{
    io::ClientExecutorInput, ChainVariant, ClientExecutor, EthereumVariant, CHAIN_ID_ETH_MAINNET,
    CHAIN_ID_LINEA_MAINNET, CHAIN_ID_OP_MAINNET,
};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use std::fs;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const BLEVM_ELF: &[u8] = include_elf!("blevm");

#[tokio::main]
async fn main() {
    let namespace: Namespace =
        Namespace::new_v0(&hex::decode("0f0f0f0f0f0f0f0f0f0f").unwrap()).unwrap();

    /*let input_bytes = fs::read("input/1/18884864.bin").expect("could not read file");
    let input: ClientExecutorInput =
        bincode::deserialize(&input_bytes).expect("could not deserialize");
    let block = input.current_block.clone();
    let block_bytes = bincode::serialize(&block).unwrap();
    let blob = Blob::new(namespace, block_bytes, AppVersion::V3).unwrap();
    println!("{}", hex::encode(blob.commitment.0))*/

    /*let height: u64 = client
        .blob_submit(&[blob.clone()], TxConfig::default())
        .await
        .unwrap();

    let blob_from_chain = client
        .blob_get(height, namespace, blob.commitment.clone())
        .await
        .unwrap();*/

    let token = std::env::var("CELESTIA_NODE_AUTH_TOKEN").expect("Token not provided");
    let client = Client::new("ws://localhost:26658", Some(&token))
        .await
        .expect("Failed creating rpc client");

    let blob = client
        .blob_get(
            2988873,
            namespace,
            Commitment(
                hex::decode("7946cf528edd745efb25201246e647d5aaca3fb52bf0f606b695b4ab0bd33f4c")
                    .unwrap()
                    .try_into()
                    .unwrap(),
            ),
        )
        .await
        .expect("Failed getting blob");
    println!("script {}", hex::encode(blob.commitment.0));

    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    let f = fs::read("input/1/18884864.bin").expect("could not read file");

    // load the prev state + new block
    let input1: ClientExecutorInput = bincode::deserialize(&f).expect("could not deserialize");

    let namespace: Namespace = Namespace::new_v0(&[15; 10]).unwrap(); // [15; 10] is just a randomly chosen valid namespace

    let header: ExtendedHeader =
        serde_json::from_str(&fs::read_to_string("header.json").unwrap()).unwrap();
    println!("header hash {:?}", header.hash());

    let eds_row_roots = header.dah.row_roots();
    let eds_column_roots = header.dah.column_roots();
    let eds_size: u64 = eds_row_roots.len().try_into().unwrap();
    let ods_size = eds_size / 2;

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

    let blob_index: u64 = blob.index.unwrap();
    // calculate the blob_size, measured in "shares".
    let blob_size: u64 = max(1, blob.data.len() as u64 / 512);
    let first_row_index: u64 = blob_index.div_ceil(eds_size) - 1;
    let ods_index = blob.index.unwrap() - (first_row_index * ods_size);

    let last_row_index: u64 = (ods_index + blob_size).div_ceil(ods_size) - 1;

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
    stdin.write(&eds_row_roots[first_row_index as usize..(last_row_index + 1) as usize].to_vec());

    let (output, report) = client.execute(BLEVM_ELF, stdin).run().unwrap();
}
