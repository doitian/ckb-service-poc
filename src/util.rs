
use std::sync::Arc;
use channel::Sender;

pub type Capacity = u64;
pub type BlockNumber = u64;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct H256 {}

#[derive(Clone, Debug)]
pub struct Shared<S> {
    pub consensus: Consensus,
    pub store: Arc<S>,
}

pub struct Request<A, R> {
    pub responsor: Sender<R>,
    pub arguments: A,
}

impl<S: ChainStore> Shared<S> {
    pub fn new(consensus: Consensus, store: S) {
        Shared {consensus, store}
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Consensus {
    pub genesis_block: IndexedBlock,
    pub initial_block_reward: Capacity,
    pub max_uncles_age: usize,
    pub max_uncles_len: usize,
    pub orphan_rate_target: f32,
    pub pow_time_span: u64,
    pub pow_spacing: u64,
    pub transaction_propagation_time: BlockNumber,
    pub transaction_propagation_timeout: BlockNumber,
}


#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct IndexedBlock {}

impl IndexedBlock {
    pub fn hash(&self) -> H256 {
        H256::default()
    }
}

#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct IndexedTransaction {}

#[derive(Clone, Debug)]
pub enum InsertionResult {
    Unknown,
}

pub struct RocksDBStore {}

pub trait ChainStore {}

impl ChainStore for RocksDBStore {}


#[derive(Clone, Default)]
pub struct Cuckoo {
    max_vertex: usize,
    max_edge: usize,
    cycle_length: usize,
}

#[derive(Clone, Default)]
pub struct CuckooEngine {
    cuckoo: Cuckoo,
}

pub trait PowEngine {
    fn init(&self, number: BlockNumber) {}
}

impl PowEngine for CuckooEngine {}

