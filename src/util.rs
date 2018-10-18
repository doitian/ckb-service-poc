
use std::sync::Arc;

pub type Capacity = u64;
pub type BlockNumber = u64;

#[derive(Clone, Default, Debug)]
pub struct Shared<S> {
    pub consensus: Consensus,
    pub store: Arc<S>,
}

impl<S: ChainStore> Shared<S> {
}

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct IndexedTransaction {}

#[derive(Clone, Debug)]
pub enum InsertionResult {
    Unknown,
}

pub struct RocksDBStore {}

pub trait ChainStore {}

impl ChainStore for RocksDBStore {}


#[derive(Clone)]
pub struct Cuckoo {
    max_vertex: usize,
    max_edge: usize,
    cycle_length: usize,
}

#[derive(Clone)]
pub struct CuckooEngine {
    cuckoo: Cuckoo,
}

pub trait PowEngine {}

impl PowEngine for CuckooEngine {}
