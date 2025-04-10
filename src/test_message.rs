#[cfg(test)]
mod tests {
    use scupt_util::message::{Message, test_check_message};
    use scupt_util::message::MsgTrait;
    use scupt_util::mt_map::MTMap;
    use scupt_util::mt_set::MTSet;

    use crate::conf_value::ConfValue;
    use crate::conf_version::{ConfVersion, ConfVersionPair};
    use crate::log_entry::LogEntry;
    use crate::msg_dtm_testing::{MDTMTesting, MUpdateConf};
    use crate::msg_raft_state::MRaftState;
    use crate::node_addr::NodeAddr;
    use crate::node_info::NodeInfo;
    use crate::raft_message::{MAppendReq, MAppendResp, MApplyReq, MApplyResp, MClientReq, MClientResp, MUpdateConfReq, MUpdateConfResp, MVoteReq, MVoteResp, PreVoteReq, PreVoteResp, RaftMessage};
    use crate::raft_role::RaftRole;
    use crate::snapshot::{Snapshot, SnapshotRange};
    use crate::term_index::TermIndex;

    #[test]
    fn test_raft_config() {
        let vec = vec![
            ConfValue {
                cluster_name: "".to_string(),
                storage_path: "".to_string(),
                node_id: 0,
                bind_address: "".to_string(),
                bind_port: 0,
                timeout_max_tick: 500,
                millisecond_tick: 50,
                max_compact_entries: 1,
                send_value_to_leader: false,
                node_peer: vec![NodeInfo {
                    node_id: 0,

                    can_vote: false,
                }],

            }
        ];
        test_message(vec);
    }

    #[test]
    fn test_term_index() {
        let vec = vec![TermIndex {
            term: 0,
            index: 0,
        }];
        test_message(vec);
    }

    #[test]
    fn test_snapshot_range() {
        let vec = vec![SnapshotRange::<i32> {
            begin_index: 0,
            end_index: 0,
            entries: Default::default(),
        }];
        test_message(vec);
    }

    #[test]
    fn test_addr() {
        let vec = vec![NodeAddr {
            node_id: 1,
            addr: "127.0.0.1".to_string(),
            port: 35,
        }];
        test_message(vec);
    }

    #[test]
    fn test_node_info() {
        let vec = vec![NodeInfo {
            node_id: 0,
            can_vote: false,
        }];
        test_message(vec);
    }


    #[test]
    fn test_raft_message() {
        let mut vec = raft_message();
        let vec2 = dtm_testing_message();
        for m in vec2 {
            vec.push(RaftMessage::DTMTesting(m));
        }
        test_message(vec);
    }

    fn raft_message() -> Vec<RaftMessage<i32>> {
        let vec = vec![
            RaftMessage::VoteReq(MVoteReq {
                term: 0,
                last_log_term: 0,
                last_log_index: 0,
            }),
            RaftMessage::VoteResp(MVoteResp {
                term: 0,
                vote_granted: false,
            }),
            RaftMessage::AppendResp(MAppendResp {
                term: 0,
                append_success: false,
                commit_index: 0,
                match_index: 0,
                next_index: 0,
            }),
            RaftMessage::AppendReq(MAppendReq {
                term: 0,
                prev_log_index: 0,
                prev_log_term: 0,
                log_entries: vec![
                    LogEntry {
                        term: 0,
                        index: 0,
                        value: 1,
                    }
                ],
                commit_index: 0,
            }),
            RaftMessage::ApplyReq(MApplyReq {
                term: 0,
                id: "".to_string(),
                begin_index: 0,
                end_index: 0,
                snapshot: Snapshot::default(),
            }),
            RaftMessage::ApplyResp(MApplyResp {
                term: 0,
                match_index: 0,
                id: "".to_string(),

            }),
            RaftMessage::PreVoteReq(PreVoteReq {
                source_nid: 0,
                request_term: 0,
                last_log_term: 0,
                last_log_index: 0,
            }),
            RaftMessage::PreVoteResp(PreVoteResp {
                source_nid: 0,
                request_term: 0,
                vote_granted: false,
            }),
            RaftMessage::ClientReq(MClientReq {
                id: "".to_string(),
                value: 2,
                source_id: None,
                wait_write_local: false,
                wait_commit: false,
                from_client_request: false,
            }),
            RaftMessage::ClientResp(MClientResp {
                id: "".to_string(),
                source_id: 1,
                index: 1,
                term: 1,
                error: 0,
                info: "".to_string(),
            }),
            RaftMessage::UpdateConfReq(MUpdateConfReq {
                term: 0,
                conf_committed: Default::default(),
                conf_new: Default::default(),
            }),
            RaftMessage::UpdateConfResp(MUpdateConfResp {
                term: 0,
                conf_committed: Default::default(),
                conf_new: Default::default(),
            }),
            RaftMessage::DTMTesting(MDTMTesting::LogCompaction(10)),
            RaftMessage::DTMTesting(MDTMTesting::Restart),
        ];
        vec
    }

    fn dtm_testing_message() -> Vec<MDTMTesting<i32>> {
        let vec = vec![
            MDTMTesting::Check(MRaftState {
                role: RaftRole::Leader,
                current_term: 0,
                log: vec![],
                snapshot: Default::default(),
                voted_for: None,
                follower_vote_granted: Default::default(),
                commit_index: 0,
                follower_next_index: Default::default(),
                follower_match_index: Default::default(),
                follower_conf: MTMap::from_vec(vec![
                    (1, ConfVersionPair {
                        conf_committed: ConfVersion {
                            term: 1,
                            version: 1,
                            index: 1,
                        },
                        conf_new: ConfVersion {
                            term: 1,
                            version: 1,
                            index: 1,
                        },
                    }),
                    (2, ConfVersionPair {
                        conf_committed: ConfVersion {
                            term: 1,
                            version: 1,
                            index: 1,
                        },
                        conf_new: ConfVersion {
                            term: 1,
                            version: 1,
                            index: 1,
                        },
                    }),
                ]),
                conf_new: Default::default(),
                conf_committed: Default::default(),
                follower_term_commit_index: Default::default(),
            }),
            MDTMTesting::Setup(MRaftState {
                role: RaftRole::Follower,
                current_term: 1,
                log: vec![],
                snapshot: Default::default(),
                voted_for: None,
                follower_vote_granted: Default::default(),
                commit_index: 2,
                follower_next_index: Default::default(),
                follower_match_index: Default::default(),
                follower_conf: Default::default(),
                conf_new: Default::default(),
                conf_committed: Default::default(),
                follower_term_commit_index: Default::default(),
            }),

            MDTMTesting::AppendLog,
            MDTMTesting::BecomeLeader,
            MDTMTesting::ClientWriteLog(1),
            MDTMTesting::UpdateConfBegin(MUpdateConf {
                nid_vote: MTSet::from_vec(vec![1, 2]),
                nid_log: MTSet::from_vec(vec![1, 2]),
            }),
            MDTMTesting::UpdateConfBegin(MUpdateConf {
                nid_vote: MTSet::from_vec(vec![1, 2]),
                nid_log: MTSet::from_vec(vec![1, 2]),
            }),
        ];
        vec
    }

    fn test_message<M: MsgTrait + 'static>(vec: Vec<M>) {
        for m in vec {
            let msg = Message::new(m, 1, 2);
            let ok = test_check_message(msg);
            assert!(ok);
        }
    }
}