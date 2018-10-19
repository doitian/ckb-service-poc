
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
    proposal_commit_txs_sender: Sender<Request<(usize, usize), (Vec<IndexedTransaction>, Vec<IndexedTransaction>)>>,
    add_transaction_sender: Sender<Request<IndexedTransaction, Result<InsertionResult, PoolError>>>,
}

impl<S: ChainStore> Service for TransactionPoolService<S> {
    type Controller = TransactionPoolController;

    fn start<TS: ToString>(self, thread_name: Option<TS>) -> (JoinHandle<()>, Self::Controller) {
        let (proposal_commit_txs_sender, proposal_commit_txs_receiver) = channel::bounded(32);
        let (add_transaction_sender, add_transaction_receiver) = channel::bounded(64);
        let join_handle = thread::spawn(move || {
            loop {
                select! {
                    recv(self.new_tip_receiver, msg) => match msg {
                        Some(block) => self.reconcile_block(&block),
                        None => error!("channel closed")
                    }
                    recv(self.switch_fork_receiver, msg) => match msg {
                        Some(blocks) => self.switch_fork(&blocks),
                        None => error!("channel closed")
                    }
                    recv(proposal_commit_txs_receiver, msg) => match msg {
                        Some(Request { responsor, arguments: (max_prop, max_tx) }) => {
                            responsor.send(self.get_proposal_commit_txs(max_prop, max_tx));
                        },
                        None => error!("channel closed"),
                    }
                    recv(add_transaction_receiver, msg) => match msg {
                        Some(Request { responsor, arguments: transaction }) => {
                            responsor.send(self.add_transaction(transaction));
                        },
                        None => error!("channel closed"),
                    }
                }
            }
        }).expect("Start transaction pool failed");
        (
            join_handle,
            TransactionPoolController { proposal_commit_txs_sender }
        )
    }
}

#[derive(Debug)]
pub enum PoolError {
    /// An invalid pool entry caused by underlying tx validation error
    InvalidTx(TransactionError),
    /// An entry already in the pool
    AlreadyInPool,
    /// A double spend
    DoubleSpent,
    /// Transaction pool is over capacity, can't accept more transactions
    OverCapacity,
    /// A duplicate output
    DuplicateOutput,
    /// Coinbase transaction
    CellBase,
    /// TimeOut
    TimeOut,
    /// Blocknumber is not right
    InvalidBlockNumber,
}

impl TransactionPoolController {

    pub fn get_proposal_commit_txs(
        max_prop: usize,
        max_tx: usize
    ) -> (Vec<IndexedTransaction>, Vec<IndexedTransaction>)
    {
        let (responsor, response) = channel::bounded(1);
        self.proposal_commit_txs_sender.send(Request {
            responsor,
            arguments: (max_prop, max_tx),
        });
        response.recv().expect("get_proposal_commit_txs failed")
    }

    pub fn add_transaction(&self, tx: IndexedTransaction) -> Result<InsertionResult, PoolError> {
        let (responsor, response) = channel::bounded(1);
        self.proposal_commit_txs_sender.send(Request {
            responsor,
            arguments: tx,
        });
        response.recv().expect("get_proposal_commit_txs failed")
    }
}

impl TransactionPoolService {

    fn get_proposal_commit_txs(
        max_prop: usize,
        max_tx: usize
    ) -> (Vec<IndexedTransaction>, Vec<IndexedTransaction>)
    {
        (vec![], vec![])
    }

    fn add_transaction(&self, tx: IndexedTransaction) -> Result<InsertionResult, PoolError> {
        Ok(InsertionResult::Unknown)
    }
}
