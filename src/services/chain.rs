
use std::thread::{self, JoinHandle};

use util::{
    Request,
    BlockNumber,
};

pub struct TipHeader {
    number: BlockNumber,
}

impl TipHeader {
    pub fn number(&self) -> BlockNumber {
        0
    }
}

pub struct ChainService<CS> {
    shared: Shared<CS>,
    miner: MinerController,
    notify: NotifyController,
}

pub struct ChainController {
    process_block_sender: Sender<Request<IndexedBlock, Result<(), Error>>>,
}

pub struct ChainReceivers {
    process_block_receiver: Receiver<Request<IndexedBlock, Result<(), Error>>>,
}

impl ChainService {
    pub fn new(
        shared: Shared<CS>,
        miner: MinerController,
        notify: NotifyController
    ) -> ChainService {
        ChainService { shared, miner, notify }
    }

    pub fn start(mut self, receivers: ChainReceivers) -> JoinHandle<()> {
        thread::spawn(move || loop {
            select! {
                recv(receivers.process_block_receiver, msg) => match msg {
                    Some(Request { responsor, arguments: block }) => {
                        responsor.send(self.process_block(block));
                    },
                    None => error!("channel closed"),
                }
            }
        })
    }

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

    pub fn new() -> (ChainController, ChainReceivers) {
        let (process_block_sender, process_block_receiver) = channel::bounded(32);
        (
            ChainController { process_block_sender },
            ChainReceivers { process_block_receiver }
        )
    }

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
