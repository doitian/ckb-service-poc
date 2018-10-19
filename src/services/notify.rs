
use channel;
use channel::{Receiver, Sender};
use fnv::FnvHashMap;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use util::{
    Request,
    IndexedBlock,
};

pub const MINER_SUBSCRIBER: &str = "miner";
pub const TXS_POOL_SUBSCRIBER: &str = "txs_pool";

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ForkBlocks{}

type StopSignal = ();
pub type MsgNewTransaction = ();
pub type MsgNewTip = Arc<IndexedBlock>;
pub type MsgSwitchFork = Arc<ForkBlocks>;
pub type NotifyRegister<M> = Sender<Request<(String, usize), Receiver<M>>>;

#[derive(Default)]
pub struct NotifyService {}

#[derive(Clone)]
pub struct NotifyController {
    signal: Sender<StopSignal>,
    new_transaction_register: NotifyRegister<MsgNewTransaction>,
    new_tip_register: NotifyRegister<MsgNewTip>,
    switch_fork_register: NotifyRegister<MsgSwitchFork>,
    new_transaction_notifier: Sender<MsgNewTransaction>,
    new_tip_notifier: Sender<MsgNewTip>,
    switch_fork_notifier: Sender<MsgSwitchFork>,
}

impl NotifyService {
    pub fn start<S: ToString>(self, thread_name: Option<S>) -> (JoinHandle<()>, NotifyController) {
        let (signal_sender, signal_receiver) = channel::bounded::<()>(1);
        let (new_transaction_register, new_transaction_register_receiver) = channel::bounded(2);
        let (new_tip_register, new_tip_register_receiver) = channel::bounded(2);
        let (switch_fork_register, switch_fork_register_receiver) = channel::bounded(2);
        let (new_transaction_sender, new_transaction_receiver) =
            channel::bounded::<MsgNewTransaction>(128);
        let (new_tip_sender, new_tip_receiver) = channel::bounded::<MsgNewTip>(128);
        let (switch_fork_sender, switch_fork_receiver) = channel::bounded::<MsgSwitchFork>(128);

        let mut new_transaction_subscribers = FnvHashMap::default();
        let mut new_tip_subscribers = FnvHashMap::default();
        let mut switch_fork_subscribers = FnvHashMap::default();

        let mut thread_builder = thread::Builder::new();
        // Mainly for test: give a empty thread_name
        if let Some(name) = thread_name {
            thread_builder = thread_builder.name(name.to_string());
        }
        let join_handle = thread_builder
            .spawn(move || loop {
                select! {
                    recv(signal_receiver, _) => {
                        break;
                    }

                    recv(new_transaction_register_receiver, msg) => Self::handle_register_new_transaction(
                        &mut new_transaction_subscribers, msg
                    ),
                    recv(new_tip_register_receiver, msg) => Self::handle_register_new_tip(
                        &mut new_tip_subscribers, msg
                    ),
                    recv(switch_fork_register_receiver, msg) => Self::handle_register_switch_fork(
                        &mut switch_fork_subscribers, msg
                    ),

                    recv(new_transaction_receiver, msg) => Self::handle_notify_new_transaction(
                        &new_transaction_subscribers, msg
                    ),
                    recv(new_tip_receiver, msg) => Self::handle_notify_new_tip(
                        &new_tip_subscribers, msg
                    ),
                    recv(switch_fork_receiver, msg) => Self::handle_notify_switch_fork(
                        &switch_fork_subscribers, msg
                    )
                }
            }).expect("Start notify service failed");

        (
            join_handle,
            NotifyController {
                new_transaction_register,
                new_tip_register,
                switch_fork_register,
                new_transaction_notifier: new_transaction_sender,
                new_tip_notifier: new_tip_sender,
                switch_fork_notifier: switch_fork_sender,
                signal: signal_sender,
            },
        )
    }

    fn handle_register_new_transaction(
        subscribers: &mut FnvHashMap<String, Sender<MsgNewTransaction>>,
        msg: Option<Request<(String, usize), Receiver<MsgNewTransaction>>>,
    ) {
        match msg {
            Some(Request {
                responsor,
                arguments: (name, capacity),
            }) => {
                debug!(target: "notify", "Register new_transaction {:?}", name);
                let (sender, receiver) = channel::bounded::<MsgNewTransaction>(capacity);
                subscribers.insert(name, sender);
                responsor.send(receiver);
            }
            None => warn!(target: "notify", "Register new_transaction channel is closed"),
        }
    }

    fn handle_register_new_tip(
        subscribers: &mut FnvHashMap<String, Sender<MsgNewTip>>,
        msg: Option<Request<(String, usize), Receiver<MsgNewTip>>>,
    ) {
        match msg {
            Some(Request {
                responsor,
                arguments: (name, capacity),
            }) => {
                debug!(target: "notify", "Register new_tip {:?}", name);
                let (sender, receiver) = channel::bounded::<MsgNewTip>(capacity);
                subscribers.insert(name, sender);
                responsor.send(receiver);
            }
            None => warn!(target: "notify", "Register new_tip channel is closed"),
        }
    }

    fn handle_register_switch_fork(
        subscribers: &mut FnvHashMap<String, Sender<MsgSwitchFork>>,
        msg: Option<Request<(String, usize), Receiver<MsgSwitchFork>>>,
    ) {
        match msg {
            Some(Request {
                responsor,
                arguments: (name, capacity),
            }) => {
                debug!(target: "notify", "Register switch_fork {:?}", name);
                let (sender, receiver) = channel::bounded::<MsgSwitchFork>(capacity);
                subscribers.insert(name, sender);
                responsor.send(receiver);
            }
            None => warn!(target: "notify", "Register switch_fork channel is closed"),
        }
    }

    fn handle_notify_new_transaction(
        subscribers: &FnvHashMap<String, Sender<MsgNewTransaction>>,
        msg: Option<MsgNewTransaction>,
    ) {
        match msg {
            Some(()) => {
                trace!(target: "notify", "event new transaction {:?}", msg);
                for subscriber in subscribers.values() {
                    subscriber.send(());
                }
            }
            None => warn!(target: "notify", "new transaction channel is closed"),
        }
    }

    fn handle_notify_new_tip(
        subscribers: &FnvHashMap<String, Sender<MsgNewTip>>,
        msg: Option<MsgNewTip>,
    ) {
        match msg {
            Some(msg) => {
                trace!(target: "notify", "event new tip {:?}", msg);
                for subscriber in subscribers.values() {
                    subscriber.send(Arc::clone(&msg));
                }
            }
            None => warn!(target: "notify", "new tip channel is closed"),
        }
    }

    fn handle_notify_switch_fork(
        subscribers: &FnvHashMap<String, Sender<MsgSwitchFork>>,
        msg: Option<MsgSwitchFork>,
    ) {
        match msg {
            Some(msg) => {
                trace!(target: "notify", "event switch fork {:?}", msg);
                for subscriber in subscribers.values() {
                    subscriber.send(Arc::clone(&msg));
                }
            }
            None => warn!(target: "notify", "event 3 channel is closed"),
        }
    }
}

impl NotifyController {
    pub fn stop(self) {
        self.signal.send(());
    }

    pub fn subscribe_new_transaction<S: ToString>(&self, name: S) -> Receiver<MsgNewTransaction> {
        let (responsor, response) = channel::bounded(1);
        self.new_transaction_register.send(Request {
            responsor,
            arguments: (name.to_string(), 128),
        });
        // Ensure the subscriber is registered.
        response.recv().expect("Subscribe new transaction failed")
    }
    pub fn subscribe_new_tip<S: ToString>(&self, name: S) -> Receiver<MsgNewTip> {
        let (responsor, response) = channel::bounded(1);
        self.new_tip_register.send(Request {
            responsor,
            arguments: (name.to_string(), 128),
        });
        // Ensure the subscriber is registered.
        response.recv().expect("Subscribe new tip failed")
    }
    pub fn subscribe_switch_fork<S: ToString>(&self, name: S) -> Receiver<MsgSwitchFork> {
        let (responsor, response) = channel::bounded(1);
        self.switch_fork_register.send(Request {
            responsor,
            arguments: (name.to_string(), 128),
        });
        // Ensure the subscriber is registered.
        response.recv().expect("Subscribe switch fork failed")
    }

    pub fn notify_new_transaction(&self) {
        self.new_transaction_notifier.send(());
    }
    pub fn notify_new_tip(&self, block: MsgNewTip) {
        self.new_tip_notifier.send(block);
    }
    pub fn notify_switch_fork(&self, txs: MsgSwitchFork) {
        self.switch_fork_notifier.send(txs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_transaction() {
        let (handle, notify) = NotifyService::default().start::<&str>(None);
        let receiver1 = notify.subscribe_new_transaction("miner1");
        let receiver2 = notify.subscribe_new_transaction("miner2");
        notify.notify_new_transaction();
        assert_eq!(receiver1.recv(), Some(()));
        assert_eq!(receiver2.recv(), Some(()));
        notify.stop();
        handle.join().expect("join failed");
    }

    #[test]
    fn test_new_tip() {
        let tip = Arc::new(IndexedBlock::default());

        let (handle, notify) = NotifyService::default().start::<&str>(None);
        let receiver1 = notify.subscribe_new_tip("miner1");
        let receiver2 = notify.subscribe_new_tip("miner2");
        notify.notify_new_tip(Arc::clone(&tip));
        assert_eq!(receiver1.recv(), Some(Arc::clone(&tip)));
        assert_eq!(receiver2.recv(), Some(tip));
        notify.stop();
        handle.join().expect("join failed");
    }

    #[test]
    fn test_switch_fork() {
        let blks = Arc::new(ForkBlocks::default());

        let (handle, notify) = NotifyService::default().start::<&str>(None);
        let receiver1 = notify.subscribe_switch_fork("miner1");
        let receiver2 = notify.subscribe_switch_fork("miner2");
        notify.notify_switch_fork(Arc::clone(&blks));
        assert_eq!(receiver1.recv(), Some(Arc::clone(&blks)));
        assert_eq!(receiver2.recv(), Some(blks));
        notify.stop();
        handle.join().expect("join failed");
    }
}
