
use std::thread;
use std::thread::JoinHandle;
use std::sync::Arc;
use std::error::Error;

use channel::{Sender, Receiver};

use util::{
    Shared,
    ChainStore,
    IndexedBlock,
    IndexedTransaction,
    InsertionResult,
};
use service::{Request, Service};
use services::notify::{
    NotifyController,
    ForkBlocks,
    TXS_POOL_SUBSCRIBER,
};

pub struct TransactionPoolService<S> {
    shared: Shared<S>,
    notify: NotifyController,
    new_tip_receiver: Receiver<Arc<IndexedBlock>>,
    switch_fork_receiver: Receiver<Arc<ForkBlocks>>
}

impl<S: ChainStore> TransactionPoolService<S> {
    pub fn new(
        shared: Shared<S>,
        notify: NotifyController,
    ) -> Self {
        let new_tip_receiver = notify.subscribe_new_tip(TXS_POOL_SUBSCRIBER);
        let switch_fork_receiver = notify.subscribe_switch_fork(TXS_POOL_SUBSCRIBER);
        TransactionPoolService {
            shared,
            notify,
            new_tip_receiver,
            switch_fork_receiver,
        }
    }
}

#[derive(Clone, Default)]
pub struct TransactionPoolController {
    proposal_commit_txs: Sender<Request<(usize, usize), (Vec<IndexedTransaction>, Vec<IndexedTransaction>)>>,
}

impl<S: ChainStore> Service for TransactionPoolService<S> {
    type Controller = TransactionPoolController;

    fn start<TS: ToString>(self, thread_name: Option<TS>) -> (JoinHandle<()>, Self::Controller) {
        let join_handle = thread::spawn(move || loop {
            select! {
                recv(self.new_tip_receiver, msg) => match msg {
                    Some(block) => self.reconcile_block(&block),
                    None => error!("channel closed")
                }
                recv(self.switch_fork_receiver, msg) => match msg {
                    Some(blocks) => self.switch_fork(&blocks),
                    None => error!("channel closed")
                }
            }
        }).expect("Start transaction pool failed");
        (
            join_handle,
            TransactionPoolController {}
        )
    }
}

pub enum PoolError {
}

impl TransactionPoolController {

    pub fn get_proposal_commit_txs(
        max_prop: usize,
        max_tx: usize
    ) -> (Vec<IndexedTransaction>, Vec<IndexedTransaction>)
    {
        (vec![], vec![])
    }

    pub fn add_transaction(&self, tx: IndexedTransaction) -> Result<InsertionResult, PoolError> {
        Ok(InsertionResult::Unknown)
    }
}
