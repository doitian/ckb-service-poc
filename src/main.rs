#[macro_use]
extern crate log;
#[macro_use]
extern crate crossbeam_channel as channel;
extern crate parking_lot;
extern crate fnv;

mod util;
mod service;
mod services;

fn main() {
    println!("Hello, world!");
}
