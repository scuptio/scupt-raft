use std::net::{IpAddr, SocketAddr};
use std::process::exit;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use scupt_net::endpoint_async::EndpointAsync;
use scupt_net::es_option::{ESConnectOpt, ESOption};
use scupt_net::event_sink_async::EventSinkAsync;
use scupt_net::io_service::{IOService, IOServiceOpt};
use scupt_net::io_service_async::IOServiceAsync;
use scupt_net::message_receiver_async::ReceiverAsync;
use scupt_net::message_sender_async::SenderAsync;
use scupt_net::notifier::Notifier;
use scupt_net::task::spawn_local_task;
use scupt_util::message::MsgTrait;
use scupt_util::node_id::NID;
use scupt_util::res::Res;
use scupt_util::res_of::res_parse;
use tokio::runtime::Runtime;
use tokio::task::LocalSet;
use tokio::time::sleep;
use tracing::error;

use crate::node_addr::NodeAddr;
use crate::raft_message::RaftMessage;
use crate::state_machine::StateMachine;
use crate::storage::Storage;

pub struct RaftSetting {
    pub node_id: NID,
    pub node_peer_addr: Vec<NodeAddr>,
    pub enable_testing: bool,
    pub _auto_name: String,
}

pub struct RaftNode<T: MsgTrait + 'static> {
    node_id: NID,
    _dtm_testing: bool,
    node_peer_addrs: Vec<NodeAddr>,
    storage: Storage<T>,
    sender: Arc<dyn SenderAsync<RaftMessage<T>>>,
    receiver: Arc<dyn ReceiverAsync<RaftMessage<T>>>,
    notify: Notifier,
    service: Arc<dyn IOServiceAsync<RaftMessage<T>>>,
    state_machine: Arc<StateMachine<T>>,
}


impl<T: MsgTrait + 'static> RaftNode<T> {
    pub fn new(
        setting: RaftSetting,
        storage: Storage<T>,
        notify: Notifier
    ) -> Res<Self> {
        let opt = IOServiceOpt {
            num_message_receiver: 1,
            testing: setting.enable_testing,
            sync_service: false,
            port_debug: None,
        };
        let service = IOService::<RaftMessage<T>>::new_async_service(
            setting.node_id,
            "RaftService".to_string(),
            opt,
            notify.clone())?;
        let sender = service.default_sender();

        let mut receiver_vec = service.receiver();
        let receiver = receiver_vec.pop().unwrap();
        let state_machine = StateMachine::new(setting.enable_testing, setting._auto_name.clone());

        Ok(RaftNode {
            node_id: setting.node_id,
            _dtm_testing: setting.enable_testing,
            node_peer_addrs: setting.node_peer_addr.clone(),
            storage,
            sender,
            receiver,
            notify,
            service,
            state_machine: Arc::new(state_machine),
        })
    }

    /// async run on a LocalSet
    pub fn local_run(&self, local: &LocalSet) {
        self.start_connect(local);
        self.start_serve(local);
        self.service.local_run(local);
    }

    /// block run on LocalSet
    pub fn block_run(&self, opt_ls: Option<LocalSet>, runtime: Arc<Runtime>) {
        self.service.block_run(opt_ls, runtime);
    }

    fn start_serve(&self, local: &LocalSet) {
        let state_machine = self.state_machine.clone();
        let notify = self.notify.clone();
        let event = self.service.default_sink().clone();

        let sender = self.sender.clone();
        let receiver = self.receiver.clone();
        let storage = self.storage.clone();
        let id = self.node_id.clone();
        let mut opt_local_addr = None;
        for n in self.node_peer_addrs.iter() {
            if id == n.node_id {
                opt_local_addr = Some((*n).clone())
            }
        }
        let local_addr = match opt_local_addr {
            Some(n) => { n }
            None => { panic!("error"); }
        };
        let _ = local.spawn_local(async move {
            spawn_local_task(notify.clone(), format!("state_machine_{}_serve", id).as_str(),
                             async move {
                                 let r = Self::serve(
                                     local_addr, notify, state_machine,
                                     event, storage, sender, receiver,
                                 ).await;
                                 match r {
                                     Ok(_) => {}
                                     Err(e) => {
                                         error!("{}", e.to_string());
                                         exit(-1);
                                     }
                                 }
                             },
            ).unwrap();
        });
    }

    async fn serve(
        local: NodeAddr,
        notifier: Notifier,
        state_machine: Arc<StateMachine<T>>,
        event: Arc<dyn EventSinkAsync<RaftMessage<T>>>,
        storage: Storage<T>,
        sender: Arc<dyn SenderAsync<RaftMessage<T>>>,
        receiver: Arc<dyn ReceiverAsync<RaftMessage<T>>>,
    ) -> Res<()> {
        let ip_addr = res_parse(IpAddr::from_str(local.addr.as_str()))?;
        let sock_addr = SocketAddr::new(ip_addr, local.port);
        event.serve(sock_addr, ESOption::default()).await?;
        state_machine.serve(storage, notifier, sender, receiver).await?;
        Ok(())
    }

    fn start_connect(&self, local: &LocalSet) {
        for node_peer in self.node_peer_addrs.iter() {
            if node_peer.node_id != self.node_id {
                let nid = node_peer.node_id;
                let addr = SocketAddr::new(IpAddr::from_str(node_peer.addr.as_str()).unwrap(), node_peer.port);
                let notify = self.notify.clone();
                let event_sink = self.service.default_sink().clone();
                local.spawn_local(
                    async move {
                        spawn_local_task(notify,
                                         "connect_to_raft_peer",
                                         async move {
                                             let r = Self::start_connect_to(event_sink, nid, addr).await;
                                             match r {
                                                 Ok(_) => {}
                                                 Err(e) => { panic!("{}", e); }
                                             }
                                         },
                        ).unwrap()
                    }
                );
            }
        }
    }

    async fn start_connect_to(event_sink: Arc<dyn EventSinkAsync<RaftMessage<T>>>, nid: NID, addr: SocketAddr) -> Res<Arc<dyn EndpointAsync<RaftMessage<T>>>> {
        loop {
            let r = event_sink.connect(
                nid,
                addr,
                ESConnectOpt::default().enable_return_endpoint(true),
            ).await;
            match r {
                Ok(opt_ep) => {
                    return Ok(opt_ep.unwrap());
                }
                Err(_e) => {
                    sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }
}