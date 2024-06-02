#[cfg(test)]
pub mod tests {
    use std::process::exit;
    use std::sync::Arc;
    use std::thread;

    use scupt_net::notifier::Notifier;
    use scupt_util::error_type::ET;
    use scupt_util::message::MsgTrait;
    use scupt_util::res::Res;
    use tokio::runtime::Builder;
    use tokio::task::LocalSet;
    use tracing::{error, trace};

    use crate::conf_value::ConfValue;
    use crate::node_addr::NodeAddr;
    use crate::node_info::NodeInfo;
    use crate::raft_node::RaftNode;
    use crate::storage::Storage;
    use crate::test_store_simple::tests::SMStoreSimple;

    /// portal of raft testing
    /// including specification-driven testing and fuzzy testing
    pub struct TestRaftNode {
        join_handle: thread::JoinHandle<()>,
    }

    impl TestRaftNode {
        pub fn start_node<M: MsgTrait + 'static>(
            cluster_name: String,
            this_peer: (NodeAddr, NodeInfo),
            peers: Vec<(NodeAddr, NodeInfo)>,
            notifier: Notifier,
            storage_path: String,
            ms_tick: u64,
            timeout_max_tick: u64,
            max_compact_entries: u64,
            enable_check_invariants: bool,
            send_to_leader: bool,
        ) -> Res<TestRaftNode> {
            let mut conf = ConfValue::new(
                cluster_name,
                this_peer.0.node_id,
                storage_path.clone(),
                this_peer.0.addr.clone(), this_peer.0.port);
            conf.millisecond_tick = ms_tick;
            conf.timeout_max_tick = timeout_max_tick;
            conf.max_compact_entries = max_compact_entries;
            conf.send_value_to_leader = send_to_leader;
            let mut node_peer_addr = vec![];
            for (_addr, _info) in peers.iter() {
                conf.add_peers(_info.node_id, _info.can_vote);
                node_peer_addr.push(_addr.clone());
            }

            let r_join_handle = thread::Builder::new()
                .name(format!("node_raft_{}", this_peer.0.node_id))
                .spawn(move || {
                    let r = Self::thread_run::<M>(conf, node_peer_addr, notifier, enable_check_invariants);
                    match r {
                        Ok(()) => {}
                        Err(e) => {
                            error!("{}", e.to_string());
                            exit(-1);
                        }
                    }
                    trace!("node {} stop", this_peer.0.node_id);
                });
            let join_handle = match r_join_handle {
                Ok(join_handle) => { join_handle }
                Err(e) => {
                    error!("spawn thread error, {}", e);
                    return Err(ET::IOError(e.to_string()));
                }
            };
            let n = TestRaftNode {
                join_handle,
            };
            Ok(n)
        }

        pub fn thread_run<M: MsgTrait + 'static>(
            conf: ConfValue,
            node_peer_addr: Vec<NodeAddr>,
            notifier: Notifier,
            _enable_check_invariants: bool,
        ) -> Res<()> {
            let s = Arc::new(SMStoreSimple::<M>::create(
                conf.clone()
            )?);
            let storage = Storage::Async(s);
            let node = RaftNode::<M>::new(conf.node_id, node_peer_addr, storage, notifier, true)?;
            let ls = LocalSet::new();
            let r = Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let runtime = Arc::new(r);
            node.local_run(&ls);
            node.block_run(Some(ls), runtime);
            Ok(())
        }
        pub fn join(self) {
            self.join_handle.join().unwrap();
        }
    }
}

