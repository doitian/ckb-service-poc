
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;
use channel::{self, Sender, Receiver};
use fnv::FnvHashMap;

use service::{Request, Service};
use services::notify::{NotifyController, MINER_SUBSCRIBER};
use services::chain::ChainController;
use services::tx_pool::TransactionPoolController;
use util::{
    H256,
    IndexedBlock,
    Shared,
    BlockNumber,
    PowEngine,
    ChainStore,
};

pub struct MinerService<S, P> {
    shared: Shared<S>,
    pow: P,
    chain: ChainController,
    tx_pool: TransactionPoolController,
    new_transaction_receiver: Receiver<()>,
    new_tip_receiver: Receiver<Arc<IndexedBlock>>,
    candidate_uncles: FnvHashMap<H256, IndexedBlock>,
    mining_number: BlockNumber,
}

impl<S, P> MinerService<S, P>
where
    S: ChainStore,
    P: PowEngine,
{
    pub fn new(
        shared: Shared<S>,
        pow: P,
        chain: ChainController,
        tx_pool: TransactionPoolController,
        notify: &NotifyController,
    ) -> MinerService<S, P>
    {
        let mining_number = chain.tip_header().number();
        let new_transaction_receiver = notify.subscribe_new_transaction(MINER_SUBSCRIBER);
        let new_tip_receiver = notify.subscribe_new_tip(MINER_SUBSCRIBER);

        MinerService {
            shared,
            pow,
            chain,
            tx_pool,
            new_transaction_receiver,
            new_tip_receiver,
            mining_number,
            candidate_uncles: FnvHashMap::default(),
        }
    }

    fn commit_new_block(&mut self) {
        unimplemented!();
    }
}

pub struct MinerController {
    uncle_sender: Sender<IndexedBlock>,
}

pub struct MinerReceivers {
    uncle_receiver: Receiver<IndexedBlock>,
}

impl<S, P> MinerService<S, P>
where
    S: ChainStore,
    P: PowEngine,
{
    fn start(mut self, receivers: MinerReceivers) -> JoinHandle<()> {
        thread::spawn(move || {
            self.pow.init(self.mining_number);

            loop {
                select! {
                    recv(receivers.uncle_receiver, msg) => match msg {
                        Some(uncle_block) => {
                            self.candidate_uncles.insert(uncle_block.hash(), uncle_block);
                        }
                        None => error!(target: "miner", "uncle_receiver closed")
                    }
                    default => {
                        self.commit_new_block();
                    }
                }
            }
        })
    }
}

impl MinerController {
    pub fn new() -> (MinerController, MinerReceivers) {
        let (uncle_sender, uncle_receiver) = channel::bounded::<IndexedBlock>(32);
        (
            MinerController { uncle_sender },
            MinerReceivers { uncle_receiver },
        )
    }

    pub fn add_uncle(&self, uncle_block: IndexedBlock) {
        self.uncle_sender.send(uncle_block);
    }
}
