
use std::thread::{self, JoinHandle};

use util::{
    BlockNumber,
};
use service::{Request, Service};

pub struct ChainService {
}

pub struct ChainController {
}

impl Service for ChainService {
    type Controller = ChainController;

    fn start<S: ToString>(mut self, thread_name: Option<S>) -> (JoinHandle<()>, Self::Controller) {
        let mut thread_builder = thread::Builder::new();
        // Mainly for test: give a empty thread_name
        if let Some(name) = thread_name {
            thread_builder = thread_builder.name(name.to_string());
        }

        let join_handle = thread_builder
            .spawn(move || {
                loop {
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

impl ChainController {
    pub fn tip_header(&self) -> TipHeader {
        TipHeader { number: 0 }
    }
}
