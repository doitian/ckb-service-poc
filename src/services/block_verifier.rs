
use channel::{self, Sender, Receiver};

use service::{Request, Service};
use util::{
    IndexedBlock
}


pub struct BlockVerifierService<CS, P> {
    shared: Shared<CS>,
    pow: P,
}


pub struct BlockVerifierController {
    block_sender: Sender<Request<Arc<IndexedBlock>, Result<(), Error>>>,
}

impl<CS, P> Service for BlockVerifierService<CS, P>
where
      CS: ChainStore,
      P: PowEngine,
{
    type Controller = BlockVerifierController;

    fn start<S: ToString>(self, thread_name: Option<S>) -> (JoinHandle<()>, Self::Controller) {
        let (block_sender, block_receiver) = channel::bounded(64);
        let join_handle = thread::spawn(move || loop {
            select! {
                recv(block_receiver, msg) => {
                    Some(Request { responsor, arguments: block }) => {
                        responsor.send(self.verify(block));
                    }
                    None => error!("channel closed")
                }
            }
        })
    }
}

impl BlockVerifierController {
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
    fn verify(&self, block: Arc<IndexedBlock>) -> Result<(), Error> {
        Ok(())
    }
}
