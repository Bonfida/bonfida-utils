use std::{
    net::{IpAddr, SocketAddr},
    path::Path,
    sync::{Arc, RwLock},
};

use bonfida_localnet::test_validator::TestValidator;
pub use solana_core::validator::Validator;
use solana_core::{
    tower_storage,
    validator::{ValidatorConfig, ValidatorStartProgress},
};
use solana_gossip::cluster_info::Node;
use solana_rpc::rpc::JsonRpcConfig;
use solana_sdk::{
    clock::Slot,
    signature::{read_keypair_file, Keypair},
    signer::Signer,
};
use solana_streamer::socket::SocketAddrSpace;

pub fn main() {
    let keypair = Arc::new(read_keypair_file("genesis/identity.json").unwrap());
    let ledger_path = Path::new("genesis");
    let vote_kp = read_keypair_file("genesis/vote-account.json").unwrap();
    let node = Node::new_localhost_with_pubkey(&keypair.pubkey());
    let bind_address = solana_net_utils::parse_host("127.0.0.1").unwrap();
    let authorized_voter_keypair = Arc::new(RwLock::new(vec![Arc::clone(&keypair)]));
    let start_progress = Arc::new(RwLock::new(ValidatorStartProgress::default()));
    let validator_config = ValidatorConfig {
        require_tower: false,
        tower_storage: Arc::new(tower_storage::FileTowerStorage::new(
            ledger_path.to_path_buf(),
        )),
        rpc_config: JsonRpcConfig {
            enable_rpc_transaction_history: true,
            enable_extended_tx_metadata_storage: true,
            faucet_addr: Some(solana_net_utils::parse_host_port("127.0.0.1:9900").unwrap()),
            health_check_slot_distance: 150,
            rpc_threads: num_cpus::get(),
            rpc_niceness_adj: 0,
            full_api: true,
            obsolete_v1_7_api: false,
            rpc_scan_and_fix_roots: false,
            ..JsonRpcConfig::default()
        },
        rpc_addrs: Some((
            SocketAddr::new(bind_address, 8899),
            SocketAddr::new(bind_address, 8900),
        )),
        poh_verify: false,
        enforce_ulimit_nofile: false,
        ..ValidatorConfig::default()
    };
    let mut validator = Validator::new(
        node,
        keypair,
        ledger_path,
        &vote_kp.pubkey(),
        authorized_voter_keypair,
        vec![],
        &validator_config,
        true,
        Arc::clone(&start_progress),
        SocketAddrSpace::new(true),
        false,
        4,
    );
    validator.warp_to_slot(1_000_000).unwrap();
    validator.join();
}
