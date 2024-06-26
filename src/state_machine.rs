use std::sync::Arc;
use std::time::Duration;

use scupt_net::message_receiver_async::ReceiverAsync;
use scupt_net::message_sender_async::SenderAsync;
use scupt_net::notifier::Notifier;
use scupt_net::opt_send::OptSend;
use scupt_net::task::spawn_local_task;
use scupt_util::error_type::ET;
use scupt_util::message::{Message, MsgTrait};
use scupt_util::res::Res;
use sedeve_kit::{input, output};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::debug;
use crate::msg_dtm_testing::MDTMTesting;

use crate::raft_message::{RAFT, RaftMessage};
use crate::state::RaftState;
use crate::state_machine_inner::_StateMachineInner;
use crate::storage::Storage;

pub struct StateMachine<T: MsgTrait + 'static> {
    inner: StateMachineInner<T>,
    dtm_testing: bool,
}

#[derive(Clone)]
struct StateMachineInner<T: MsgTrait + 'static> {
    inner: Arc<Mutex<_StateMachineInner<T>>>,
}


impl<T: MsgTrait + 'static> StateMachineInner<T> {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(_StateMachineInner::new())),
        }
    }
    async fn recovery(&self, storage: &Storage<T>) -> Res<()> {
        let mut inner = self.inner.lock().await;
        inner.recovery(storage).await?;
        Ok(())
    }

    async fn ms_tick(&self) -> u64 {
        let inner = self.inner.lock().await;
        inner.raft_conf().conf_node_value_committed().value.millisecond_tick
    }

    async fn _dtm_step_incoming(
        message: &Message<RaftMessage<T>>
    ) {
        let _m = message.clone();
        input!(RAFT, _m)
    }

    /// only when read snapshot value, the `Inner` struct would read from storage,
    /// except the snapshot values, all other states are cached by the `Inner` struct
    async fn step_incoming(
        &self, message: Message<RaftMessage<T>>,
        state: &mut RaftState<T>,
        storage: &Storage<T>,
    ) -> Res<()> {
        let mut inner = self.inner.lock().await;
        Self::_dtm_step_incoming(&message).await;
        inner.step_incoming(message, state, storage).await?;
        Ok(())
    }

    /// only when read snapshot value, the `Inner` struct would read from storage,
    /// except the snapshot values, all other states are cached by the `Inner` struct
    async fn step_tick_short(&self, state: &mut RaftState<T>, storage: &Storage<T>) -> Res<()> {
        let mut inner = self.inner.lock().await;
        inner.step_tick_short(state, storage).await?;
        Ok(())
    }

    async fn step_tick_long(&self, state: &mut RaftState<T>) -> Res<()> {
        let mut inner = self.inner.lock().await;
        inner.step_tick_long(state).await?;
        Ok(())
    }

    async fn check_storage(&self, storage: &Storage<T>) -> Res<()> {
        let inner = self.inner.lock().await;
        inner.check_storage(storage).await?;
        Ok(())
    }
}


impl<T: MsgTrait + 'static> StateMachine<T> {
    pub fn new(dtm_testing: bool) -> Self {
        Self {
            inner: StateMachineInner::new(),
            dtm_testing,
        }
    }

    pub async fn serve(
        &self,
        storage: Storage<T>,
        notifier: Notifier,
        sender: Arc<dyn SenderAsync<RaftMessage<T>>>,
        receiver: Arc<dyn ReceiverAsync<RaftMessage<T>>>,
    ) -> Res<()> {
        let inner0 = self.inner.clone();
        let storage0 = storage.clone();
        let _j0 = spawn_local_task(notifier.clone(), "recovery", async move {
            inner0.recovery(&storage0).await?;
            Ok::<(), ET>(())
        }).unwrap();
        let opt_r = match _j0.await {
            Ok(_opt) => { _opt }
            Err(e) => { return Err(ET::FatalError(e.to_string())); }
        };
        match opt_r {
            Some(r) => { r? }
            None => { return Ok(()); }
        }
        let mut vec = vec![];
        let inner1 = self.inner.clone();
        let sender1 = sender.clone();
        let storage1 = storage.clone();
        let ms = inner1.ms_tick().await;
        let dtm_testing = self.dtm_testing;
        let _j1 = spawn_local_task(notifier.clone(), "short_latency", async move {
            if !dtm_testing {
                Self::loop_tick_short(ms, inner1, sender1, storage1).await?;
            }
            Ok::<(), ET>(())
        })?;
        vec.push(_j1);

        let inner2 = self.inner.clone();
        let sender2 = sender.clone();
        let storage2 = storage.clone();
        let _j2 = spawn_local_task(notifier.clone(), "long_latency", async move {
            if !dtm_testing {
                Self::loop_tick_long(ms, inner2, sender2, storage2).await?;
            }
            Ok::<(), ET>(())
        })?;
        vec.push(_j2);

        let inner3 = self.inner.clone();
        let receiver3 = receiver.clone();
        let sender3 = sender.clone();
        let storage3 = storage.clone();
        let _j3 = spawn_local_task(notifier.clone(), "incoming_message", async move {
            Self::incoming(inner3, sender3, receiver3, storage3).await?;
            Ok::<(), ET>(())
        })?;
        vec.push(_j3);

        for j in vec {
            let _ = j.await;
        }

        Ok(())
    }

    async fn incoming(
        inner: StateMachineInner<T>,
        sender: Arc<dyn SenderAsync<RaftMessage<T>>>,
        receiver: Arc<dyn ReceiverAsync<RaftMessage<T>>>,
        storage: Storage<T>,
    ) -> Res<()> {
        let mut state = Default::default();
        loop {
            let m = receiver.receive().await?;
            inner.step_incoming(m, &mut state, &storage).await?;
            Self::write_state(&mut state, &storage).await?;
            Self::send_message(&mut state, &*sender).await?;
            inner.check_storage(&storage).await?;
        }
    }

    async fn loop_tick_short(
        ms: u64,
        inner: StateMachineInner<T>,
        sender: Arc<dyn SenderAsync<RaftMessage<T>>>,
        storage: Storage<T>,
    ) -> Res<()> {
        let mut state = Default::default();
        loop {
            sleep(Duration::from_millis(ms)).await;
            inner.step_tick_short(&mut state, &storage).await?;
            Self::write_state(&mut state, &storage).await?;
            Self::send_message(&mut state, &*sender).await?;
        }
    }

    async fn loop_tick_long(
        ms: u64,
        inner: StateMachineInner<T>,
        sender: Arc<dyn SenderAsync<RaftMessage<T>>>,
        storage: Storage<T>,
    ) -> Res<()> {
        let mut state = Default::default();
        loop {
            sleep(Duration::from_millis(ms)).await;
            inner.step_tick_long(&mut state).await?;
            Self::write_state(&mut state, &storage).await?;
            Self::send_message(&mut state, &*sender).await?;
        }
    }

    async fn write_state(state: &mut RaftState<T>, storage: &Storage<T>) -> Res<()> {
        let mut vec = vec![];
        std::mem::swap(&mut state.non_volatile.operation, &mut vec);
        storage.write(vec).await?;
        Ok(())
    }

    async fn send_message(state: &mut RaftState<T>, sender: &dyn SenderAsync<RaftMessage<T>>) -> Res<()> {
        let mut vec = vec![];
        std::mem::swap(&mut state.message, &mut vec);
        for m in vec {
            debug!("send message: {:?}", m);
            let m =m.map(|_m|{
                match _m {
                    RaftMessage::UpdateConfReq(_up) => {
                        RaftMessage::DTMTesting(MDTMTesting::UpdateConfReq(_up.to_dtm_msg()))
                    }
                    _ => { _m }
                }
            });
            output!(RAFT, m.clone());
            sender.send(m, OptSend::new().enable_no_wait(true)).await?;
        }
        Ok(())
    }
}



