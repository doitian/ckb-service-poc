
use channel::Sender;
use std::thread::JoinHandle;

pub struct Request<A, R> {
    pub responsor: Sender<R>,
    pub arguments: A,
}

pub trait Service {
    type Controller;
    // Service's main loop
    fn start<S: ToString>(self, thread_name: Option<S>) -> (JoinHandle<()>, Self::Controller);
}
