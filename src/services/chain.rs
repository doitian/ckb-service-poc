
use std::thread::{self, JoinHandle};

use util::{
    BlockNumber,
};
use service::{Request, Service};

pub struct ChainService<CS> {
    shared: Shared<CS>,
    miner: MinerController,
    notify: NotifyController,
}

pub struct ChainController {
    process_block_sender: Sender<Request<IndexedBlock, Result<(), Error>>>,
}

impl Service for ChainService {
    type Controller = ChainController;

    fn start<S: ToString>(mut self, thread_name: Option<S>) -> (JoinHandle<()>, Self::Controller) {

        let (process_block_sender, process_block_receiver) = channel::bounded(32);
        let join_handle = thread::spawn(move || loop {
            select! {
                recv(process_block_receiver, msg) => {
                    Some(Request { responsor, arguments: block }) => {
                        responsor.send(self.process_block(block));
                    }
                }
            }
        }).expect("Start ChainService failed!");
        (join_handle, ChainController{})
    }
}

pub struct TipHeader {
    number: BlockNumber,
}

impl TipHeader {
    pub fn number(&self) -> BlockNumber {
        0
    }
}

impl ChainService {
    pub fn process_block(&self, block: Arc<IndexedBlock>) -> Result<(), Error> {
        let new_best_block = true;
        if new_best_block {
            self.notify.notify_new_tip(Arc::clone(&block));
        } else {
            self.miner.ad_uncle(Arc::clone(&block));
        }
        Ok(())
    }
}

impl ChainController {
    pub fn tip_header(&self) -> TipHeader {
        TipHeader { number: 0 }
    }

    pub fn process_block(&self, block: Arc<IndexedBlock>) -> Result<(), Error> {
        let (responsor, response) = channel::bounded(1);
        self.process_block_sender.send(Request {
            responsor,
            arguments: block
        });
        response.recv().expect("Process block failed")
    }
}
