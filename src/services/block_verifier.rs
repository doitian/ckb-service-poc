
use channel::{self, Sender, Receiver};

use service::{Request, Service};
use util::{
    IndexedBlock
}


pub struct BlockVerifierService {
    shared: Shared<CS>,
}


pub struct BlockVerifierController {
    block_sender: Sender<Request<IndexedBlock, Result<(), Error>>>,
}

impl Service for BlockVerifierService {
    type Controller = BlockVerifierController;

    fn start<S: ToString>(self, thread_name: Option<S>) -> (JoinHandle<()>, Self::Controller) {
        let (block_sender, block_receiver) = channel::bounded(64);
        let join_handle = thread::spawn(move || loop {
            select! {
                recv(block_receiver, msg) => {
                    Some(Request { responsor, arguments: block })
                }
            }
        })
    }
}

impl BlockVerifierService {
    pub fn verify(&self, )
}
