#[macro_use]
extern crate log;
#[macro_use]
extern crate crossbeam_channel as channel;
extern crate parking_lot;
extern crate fnv;

mod util;
mod services;

use util::{
    Shared,
    CuckooEngine,
    Consensus,
};
use services::chain::{
    ChainService,
    ChainController,
    ChainReceivers
};
use services::miner::{
    MinerService,
    MinerController,
    MinerReceivers
};
use services::tx_pool::{
    TransactionPoolService,
    TransactionPoolController,
    TransactionPoolReceivers
};
use services::block_verifier::{
    BlockVerifierService,
    BlockVerifierController,
    BlockVerifierReceivers
};

fn main() {
    let consensus = Consensus::default();
    let pow = CuckooEngine::default();
    let shared = Shared::new(consensus, pow);

    let (_handle, notify_controller) = NotifyService::default().start(Some("notify"));

    let (chain_controller, chain_receivers) = ChainController::new();
    let (miner_controller, miner_receivers) = MinerController::new();
    let (txpool_controller, txpool_receivers) = TransactionPoolController::new();
    let (block_verifier_controller, block_verifier_receivers) = BlockVerifierController::new();

    let chain_handle = ChainService::new(
        shared.clone(),
        miner_controller.clone(),
        notify_controller.clone(),
    ).start(chain_receivers);

    let miner_handle = MinerService::new(
        shared.clone(),
        pow.clone(),
        chain_controller.clone(),
        txpool_controller.clone(),
        notify_controller.clone(),
    ).start(miner_receivers);

    let txpool_handle = TransactionPoolService::new(
        shared.clone(),
        notify_controller.clone()
    ).start(txpool_receivers);
    
    let block_verifier_handle = BlockVerifierService::new(
        shared.clone(),
        pow.clone(),
    ).start(block_verifier_receivers);

    for handle in [
        chain_handle,
        miner_handle,
        txpool_handle,
        block_verifier_handle,
    ]
    {
        let _ = handle.join();
    }
}
