
use std::sync::Arc;
use channel::{self, Sender, Receiver};

use util::{
    Request,
    IndexedBlock
};

pub struct BlockVerifierService<CS, P> {
    shared: Shared<CS>,
    pow: P,
}


pub struct BlockVerifierController {
    block_sender: Sender<Request<Arc<IndexedBlock>, Result<(), Error>>>,
}

pub struct BlockVerifierReceivers {
    block_receiver: Receiver<Request<Arc<IndexedBlock>, Result<(), Error>>>,
}

impl BlockVerifierController {
    pub fn new() -> (BlockVerifierController, BlockVerifierReceivers) {
        let (block_sender, block_receiver) = channel::bounded(64);
        (
            BlockVerifierController { block_sender },
            BlockVerifierReceivers { block_receiver }
        )
    }

    pub fn verify(&self, block: Arc<IndexedBlock>) -> Result<(), Error> {
        let (responsor, response) = channel::bounded(1);
        self.block_sender.send(Request {
            responsor,
            arguments: block
        });
        response.recv().expect("Verify fialed")
    }
}

impl<CS, P> BlockVerifierService<CS, P>
where
      CS: ChainStore,
      P: PowEngine,
{
    pub fn start(self, receivers: BlockVerifierReceivers) -> JoinHandle<()> {
        thread::spawn(move || loop {
            select! {
                recv(receivers.block_receiver, msg) => match msg {
                    Some(Request { responsor, arguments: block }) => {
                        responsor.send(self.verify(block));
                    },
                    None => error!("channel closed")
                }
            }
        })
    }

    fn verify(&self, block: Arc<IndexedBlock>) -> Result<(), Error> {
        Ok(())
    }
}
