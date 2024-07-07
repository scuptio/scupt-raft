#[cfg(test)]
mod tests {
    use scupt_util::message::{Message, test_check_message};
    use scupt_util::message::MsgTrait;
    use crate::msg_dtm_testing::MDTMTesting;

    use crate::msg_raft_state::MRaftState;
    use crate::node_info::NodeInfo;
    use crate::conf_value::ConfValue;

    use crate::raft_message::{LogEntry, MAppendReq, MAppendResp, MApplyReq, MApplyResp, MClientReq, MClientResp, MUpdateConfReq, MUpdateConfResp, MVoteReq, MVoteResp, PreVoteReq, PreVoteResp, RaftMessage};
    use crate::raft_role::RaftRole;
    use crate::snapshot::Snapshot;

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
    fn test_raft_message() {
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
            RaftMessage::DTMTesting(MDTMTesting::Check(MRaftState {
                role: RaftRole::Leader,
                current_term: 0,
                log: vec![],
                snapshot: Default::default(),
                voted_for: None,
                follower_vote_granted: Default::default(),
                commit_index: 0,
                follower_next_index: Default::default(),
                follower_match_index: Default::default(),
                follower_conf: Default::default(),
                conf_new: Default::default(),
                conf_committed: Default::default(),
                follower_term_commit_index: Default::default(),
            })),
            RaftMessage::DTMTesting(MDTMTesting::Setup(MRaftState {
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
            })),
            RaftMessage::DTMTesting(MDTMTesting::AdvanceCommitIndex(1)),
            RaftMessage::DTMTesting(MDTMTesting::AppendLog),
            RaftMessage::DTMTesting(MDTMTesting::BecomeLeader),
            RaftMessage::DTMTesting(MDTMTesting::ClientWriteLog(1)),
            RaftMessage::DTMTesting(MDTMTesting::HandleAppendReq(MAppendReq {
                term: 0,
                prev_log_index: 0,
                prev_log_term: 0,
                log_entries: vec![],
                commit_index: 0,
            })),
            RaftMessage::DTMTesting(MDTMTesting::HandleAppendResp(MAppendResp {
                term: 0,
                append_success: false,
                commit_index: 0,
                match_index: 0,
                next_index: 0,
            })),
            RaftMessage::DTMTesting(MDTMTesting::HandleVoteReq(MVoteReq {
                term: 0,
                last_log_term: 0,
                last_log_index: 0,
            })),
            RaftMessage::DTMTesting(MDTMTesting::HandleVoteResp(MVoteResp {
                term: 0,
                vote_granted: false,
            })),
            RaftMessage::DTMTesting(MDTMTesting::HandleAppendReq(MAppendReq {
                term: 0,
                prev_log_index: 0,
                prev_log_term: 0,
                log_entries: vec![],
                commit_index: 0,
            })),
            RaftMessage::DTMTesting(MDTMTesting::HandleApplyResp(MApplyResp {
                term: 0,
                match_index: 0,
                id: "".to_string(),
            })),
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
        test_message(vec);
    }

    fn test_message<M: MsgTrait + 'static>(vec: Vec<M>) {
        for m in vec {
            let msg = Message::new(m, 1, 2);
            let ok = test_check_message(msg);
            assert!(ok);
        }
    }
}