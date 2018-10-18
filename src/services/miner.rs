
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;
use channel::{self, Sender, Receiver};
use fnv::FnvHashMap;
use services::notify::{NotifyController, MINER_SUBSCRIBER};
use services::chain::ChainController;
use services::tx_pool::TransactionPoolController;
use util::{
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
    )
    {
        let mining_number = chain.tip_header().header.number;
        let new_transaction_receiver = notify.subscribe_new_transaction(MINER_SUBSCRIBER);
        let new_tip_receiver = notify.subscribe_new_tip(MINER_SUBSCRIBER);

        MinerService {
            shared,
            pow,
            chain,
            tx_pool,
            notify,
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

impl<P> Service for MinerService<P>
where
    P: PowEngine + 'static,
{
    type Controller = MinerController;

    fn start<S: ToString>(mut self, thread_name: Option<S>) -> (JoinHandle<()>, Self::Controller) {
        let mut thread_builder = thread::Builder::new();
        // Mainly for test: give a empty thread_name
        if let Some(name) = thread_name {
            thread_builder = thread_builder.name(name.to_string());
        }

        let (uncle_sender, uncle_receiver) = channel::bounded::<IndexedBlock>(32);
        let join_handle = thread_builder
            .spawn(move || {
                self.pow.init(self.mining_number);

                loop {
                    select! {
                        recv(uncle_receiver, msg) => match msg {
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
            }).expect("Start miner service fialed");
        (join_handle, MinerController { uncle_sender })
    }
}

impl MinerController {
    pub fn add_uncle(&self, uncle_block: IndexedBlock) {
        self.uncle_sender.send(uncle_block);
    }
}
