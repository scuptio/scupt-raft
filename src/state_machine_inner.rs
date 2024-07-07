use std::cmp::min;
use std::collections::{Bound, HashMap, HashSet};

use scupt_util::message::{Message, MsgTrait};
use scupt_util::mt_set::MTSet;
use scupt_util::node_id::NID;
use scupt_util::res::Res;
use tracing::{debug, error, trace};

use crate::conf_node::ConfNode;
use crate::conf_value::ConfValue;
use crate::conf_version::ConfVersion;
use crate::msg_dtm_testing::MDTMTesting;
use crate::msg_raft_state::MRaftState;
use crate::node_info::NodeInfo;
use crate::non_volatile_write::{NonVolatileWrite, WriteEntriesOpt};
use crate::quorum::{
    quorum_agree_match_index,
    quorum_agree_vote,
    quorum_check_conf_term_version,
    quorum_check_term_commit_index,
};
use crate::raft_conf::{ConfNodeValue, RaftConf};
use crate::raft_message::{LogEntry, MAppendReq, MAppendResp, MApplyReq, MApplyResp, MClientReq, MClientResp, MDTMUpdateConfReq, MUpdateConfReq, MUpdateConfResp, MVoteReq, MVoteResp, PreVoteReq, PreVoteResp, RaftMessage, RCR_ERR_RESP, RCR_OK};
use crate::raft_role::RaftRole;
use crate::snapshot::{Snapshot, SnapshotRange};
use crate::state::RaftState;
use crate::storage::Storage;
use crate::term_index::TermIndex;

struct NodeSet {
    nid_log_all: Vec<NID>,
    nid_vote_all: Vec<NID>,
    committed_log:HashSet<NID>,
    new_log:HashSet<NID>,
    committed_vote:HashSet<NID>,
    new_vote:HashSet<NID>,
}


pub struct _StateMachineInner<T: MsgTrait + 'static> {
    role: RaftRole,

    // the following 5 variable need to be persisted
    // term index started form 1, 0 as invalid
    current_term: u64,
    commit_index: u64,
    log: Vec<LogEntry<T>>,
    voted_for: Option<NID>,
    conf: RaftConf,

    // we only cached max index and term of the snapshot
    snapshot_max_index_term: TermIndex,

    max_append_entries: u64,
    tick: u64,
    testing_is_crash: bool,

    // log entry index started from 1, 0 as an invalid index
    // to map the index value to vector's offset, the index value should minus 1
    follower_next_index: HashMap<NID, u64>,
    follower_match_index: HashMap<NID, u64>,

    // `follower_pre_vote_granted` and `follower_vote_granted` keep
    // [N, T] pair, follower node N vote this leader node at term T
    follower_pre_vote_granted: HashMap<NID, u64>,
    follower_vote_granted: HashMap<NID, u64>,

    follower_conf_committed: HashMap<NID, ConfVersion>,
    follower_conf_new: HashMap<NID, ConfVersion>,
    // [N, (T, C)] pair, node N's current term is T, and commit index is C
    follower_term_committed_index: HashMap<NID, TermIndex>,
    node_set: NodeSet,
}

const MAX_APPEND_ENTRIES: u64 = 2;

impl<T: MsgTrait + 'static> _StateMachineInner<T> {
    pub fn new() -> Self {
        Self {
            role: RaftRole::Follower,
            current_term: Default::default(),
            follower_next_index: Default::default(),
            follower_match_index: Default::default(),
            follower_pre_vote_granted: Default::default(),
            follower_vote_granted: Default::default(),
            commit_index: Default::default(),
            log: Default::default(),
            voted_for: None,
            snapshot_max_index_term: Default::default(),
            conf: Default::default(),
            max_append_entries: MAX_APPEND_ENTRIES,
            tick: 0,
            testing_is_crash: false,

            follower_conf_new: Default::default(),
            follower_conf_committed: Default::default(),
            follower_term_committed_index: Default::default(),
            node_set: NodeSet {
                nid_log_all: vec![],
                nid_vote_all: vec![],
                committed_log: Default::default(),
                new_log: Default::default(),
                committed_vote: Default::default(),
                new_vote: Default::default(),
            },
        }
    }


    /// When recovery, the `Inner` struct would cache all states except snapshot values
    pub async fn recovery(
        &mut self,
        storage: &Storage<T>,
    ) -> Res<()> {
        self.recovery_state(storage).await
    }


    pub fn raft_conf(&self) -> &RaftConf {
        &self.conf
    }

    /// only when read snapshot value, the `Inner` struct would read from storage,
    /// except the snapshot values, all other states are cached by the `Inner` struct
    pub async fn step_incoming(
        &mut self, message: Message<RaftMessage<T>>,
        state: &mut RaftState<T>,
        storage: &Storage<T>,
    ) -> Res<()> {
        self.handle_message(message, state, storage).await
    }

    /// only when read snapshot value, the `Inner` struct would read from storage,
    /// except the snapshot values, all other states are cached by the `Inner` struct
    pub async fn step_tick_short(
        &mut self,
        state: &mut RaftState<T>,
        storage: &Storage<T>,
    ) -> Res<()> {
        self.step_tick_inner_1(state, storage).await
    }

    pub async fn step_tick_long(&mut self, state: &mut RaftState<T>) -> Res<()> {
        self.step_tick_inner_2(state).await
    }

    async fn step_tick_inner_1(
        &mut self, state:
        &mut RaftState<T>,
        storage: &Storage<T>,
    ) -> Res<()> {
        match self.role {
            RaftRole::Leader => {
                self.step_tick_leader(state, storage).await?;
            }
            RaftRole::Follower | RaftRole::Candidate => {
                self.step_tick_follower(state)?;
            }
            RaftRole::Learner => {
                self.step_tick_learner()?;
            }
        }

        Ok(())
    }

    async fn step_tick_inner_2(&mut self, state: &mut RaftState<T>) -> Res<()> {
        self.compact_log(state)?;
        self.send_update_conf_to_follower(state)?;
        self.leader_re_conf_commit(state)?;
        Ok(())
    }

    fn compact_log(&mut self, state: &mut RaftState<T>) -> Res<()> {
        let last_log_index = self.last_log_index();
        let index = min(last_log_index, self.commit_index);
        if self.snapshot_max_index_term.index < index {
            self.compact_log_inner(index, state)?;
        }
        Ok(())
    }
    fn conf_nid_log(&self) -> (HashSet<NID>, HashSet<NID>) {
        (self.conf.conf_committed_nid_log().iter().cloned().collect(),
         self.conf.conf_new_nid_log().iter().cloned().collect())
    }

    fn conf_nid_vote(&self) -> (HashSet<NID>, HashSet<NID>) {
        (self.conf.conf_committed_nid_log().iter().cloned().collect(),
         self.conf.conf_new_nid_log().iter().cloned().collect())
    }

    fn only_one_node_can_vote(&self) -> bool {
        if self.conf.conf_node_committed().conf_version().eq(
            self.conf.conf_node_new().conf_version()) {
            self.conf.conf_committed_nid_vote().len() == 1
        } else {
            self.conf.conf_committed_nid_vote().len() == 1 &&
                self.conf.conf_new_nid_vote().len() == 1 &&
                self.conf.conf_committed_nid_vote().eq(self.conf.conf_new_nid_vote())
        }
    }
    async fn step_tick_leader(&mut self, state: &mut RaftState<T>, storage: &Storage<T>) -> Res<()> {
        assert_eq!(self.role, RaftRole::Leader);
        self.append_entries(state, storage).await?;
        Ok(())
    }

    fn step_tick_follower(&mut self, state: &mut RaftState<T>) -> Res<()> {
        if self.tick > self.conf.conf_committed_value().timeout_max_tick {
            self.pre_vote_request(state)?;
        }
        self.tick += 1;
        Ok(())
    }

    async fn handle_raft_message(
        &mut self,
        source: NID,
        dest: NID,
        msg: RaftMessage<T>,
        state: &mut RaftState<T>,
        storage: &Storage<T>,
    ) -> Res<()> {
        assert_eq!(self.node_id(), dest);
        let _a = Message::new(msg.clone(), source, dest);
        debug!("handle message {:?}", _a);
        match msg {
            RaftMessage::VoteReq(m) => {
                //event_add!(RAFT_FUZZY, Message::new(RaftEvent::VoteReq, source, dest));
                //input!(RAFT, _a);
                self.handle_vote_req(source, m, state)?;
            }
            RaftMessage::VoteResp(m) => {
                //event_add!(RAFT_FUZZY, Message::new(RaftEvent::VoteResp, source, dest));
                //input!(RAFT, _a);
                self.handle_vote_resp(source, m, state)?;
            }
            RaftMessage::AppendReq(m) => {
                //event_add!(RAFT_FUZZY, Message::new(RaftEvent::AppendReq, source, dest));
                //input!(RAFT, _a);
                self.handle_append_req(source, m, state).await?;
            }
            RaftMessage::AppendResp(m) => {
                //event_add!(RAFT_FUZZY, Message::new(RaftEvent::AppendResp, source, dest));
                //input!(RAFT, _a);
                self.handle_append_resp(source, m, state)?;
            }
            RaftMessage::PreVoteReq(_m) => {
                //event_add!(RAFT_FUZZY, Message::new(RaftEvent::PreVoteReq, source, dest)); //FUZZY
                self.handle_pre_vote_request(source, _m, state)?;
            }
            RaftMessage::PreVoteResp(_m) => {
                //event_add!(RAFT_FUZZY, Message::new(RaftEvent::PreVoteResp, source, dest)); //FUZZY
                self.handle_pre_vote_response(source, _m, state)?;
            }
            RaftMessage::ApplyReq(_m) => {
                //event_add!(RAFT_FUZZY, Message::new(RaftEvent::ApplyReq, source, dest)); //FUZZY
                //input!(RAFT, _a); //DTM
                self.handle_apply_snapshot_req(source, dest, _m, state)?;
            }
            RaftMessage::ApplyResp(_m) => {
                //event_add!(RAFT_FUZZY, Message::new(RaftEvent::ApplyResp, source, dest)); //FUZZY
                //input!(RAFT, _a); //DTM
                self.handle_apply_snapshot_resp(source, _m)?;
            }
            RaftMessage::ClientReq(m) => {
                self.handle_client_value_req(source, m, state, storage).await?;
            }
            RaftMessage::ClientResp(_) => {}
            RaftMessage::UpdateConfReq(_m) => {
                self.handle_update_conf_req(source, _m, state)?;
            }
            RaftMessage::UpdateConfResp(m) => {
                self.handle_update_conf_resp(source, m)?;
            }
            RaftMessage::DTMTesting(_m) => {
                self.handle_dtm(source, _m, state, storage).await?; //DTM
            }
        }
        Ok(())
    }

    async fn handle_message(
        &mut self, m: Message<RaftMessage<T>>,
        state: &mut RaftState<T>,
        storage: &Storage<T>,
    ) -> Res<()> {
        let source = m.source();
        let dest = m.dest();
        let _m = m.clone();
        self.handle_raft_message(source, dest, m.payload(), state, storage).await?;
        Ok(())
    }

    fn __update_conf_value(value:&mut ConfValue, can_vote_nid:&HashSet<NID>, log_nid:&HashSet<NID>){
        value.node_peer.clear();
        for i in log_nid {
            let peer = NodeInfo {
                node_id: *i,
                can_vote: can_vote_nid.contains(&i),
            };
            value.node_peer.push(peer);
        }
    }

    fn handle_dtm_update_conf_req(&mut self, source:NID, _m:MDTMUpdateConfReq, state:&mut RaftState<T>) -> Res<()> {
        let mut committed_conf_value = self.conf.conf_committed_value().clone();
        let mut new_conf_value = self.conf.conf_new_value().clone();
        Self::__update_conf_value(&mut committed_conf_value, &_m.conf_committed.nid_vote.to_set(), &_m.conf_committed.nid_log.to_set());
        Self::__update_conf_value(&mut new_conf_value, &_m.conf_new.nid_vote.to_set(), &_m.conf_new.nid_log.to_set());
        let m = MUpdateConfReq {
            term: _m.term,
            conf_committed: ConfNodeValue {
                node: _m.conf_committed,
                value: committed_conf_value,
            },
            conf_new: ConfNodeValue {
                node: _m.conf_new,
                value: new_conf_value,
            },
        };
        self.handle_update_conf_req(source, m, state)?;
        Ok(())
    }

    async fn handle_dtm(
        &mut self,
        source:NID,
        m: MDTMTesting<T>,
        state: &mut RaftState<T>,
        storage: &Storage<T>,
    ) -> Res<()> {
        match m {
            MDTMTesting::Check(m) => {
                self.state_check(m)?;
            }
            MDTMTesting::Setup(m) => {
                self.state_setup(m, state)?;
            }
            MDTMTesting::UpdateConfBegin(m) => {
                let mut value = self.conf.conf_node_value_committed().value.clone();
                Self::__update_conf_value(&mut value, &m.nid_vote.to_set(), &m.nid_log.to_set());
                self.leader_re_conf_begin(state, value,  m.nid_vote.vec().clone(), m.nid_log.vec().clone())?;
            }
            MDTMTesting::RequestVote => {
                self.start_request_vote(state)?;
            }
            MDTMTesting::ClientWriteLog(v) => {
                self.client_request_write_value_gut(v, state, storage).await?;
            }
            MDTMTesting::UpdateConfCommit => {
                self.leader_re_conf_commit(state)?;
            }
            MDTMTesting::Restart => {
                self.restart_for_testing(storage).await?;
            }
            MDTMTesting::AppendLog => {
                self.append_entries(state, storage).await?;
            }
            MDTMTesting::LogCompaction(_n) => {
                self.compact_log(state)?;
            }
            MDTMTesting::SendUpdateConf => {
                self.send_update_conf_to_follower(state)?
            }
            MDTMTesting::UpdateConfReq(_m) => {
                self.handle_dtm_update_conf_req(source, _m, state)?
            }
            _ => {
                panic!("error {:?}", m)
            }
        }
        Ok(())
    }

    fn step_tick_learner(&self) -> Res<()> {
        return Ok(());
    }

    fn pre_vote_request(&mut self, state: &mut RaftState<T>) -> Res<()> {
        self.tick = 0;
        self.role = RaftRole::Follower;
        if !self.only_one_node_can_vote() {
            self.send_pre_vote_request(state)?;
        } else {
            self.start_request_vote(state)?;
        }
        Ok(())
    }

    fn pre_vote_req_message(
        source: NID,
        dest: NID,
        term: u64,
        last_log_term: u64,
        last_log_index: u64,
    ) -> Message<RaftMessage<T>> {
        let m = Message::new(
            RaftMessage::PreVoteReq(
                PreVoteReq {
                    source_nid: source,
                    request_term: term,
                    last_log_term,
                    last_log_index,
                }
            ),
            source,
            dest,
        );
        m
    }
    fn send_pre_vote_request(&self, state: &mut RaftState<T>) -> Res<()> {
        let last_index = self.last_log_index();
        let last_term = self.last_log_term();
        let term = self.current_term;
        let from = self.node_id();
        let id_set = &self.node_set.nid_vote_all;
        for id in id_set {
            if from != *id {
                let m = Self::pre_vote_req_message(
                    from, *id, term, last_term, last_index);
                state.message.push(m);
            }
        }

        Ok(())
    }

    fn handle_pre_vote_request(
        &mut self,
        source:NID,
        m: PreVoteReq,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        if !self.node_set.can_vote(source) ||
            !self.node_set.can_vote(self.node_id()) {
            return Ok(())
        }
        let grant = self.can_grant_vote(
            m.request_term + 1,
            m.last_log_index,
            m.last_log_term,
            m.source_nid);

        let resp = Message::new(
            RaftMessage::PreVoteResp(
                PreVoteResp {
                    source_nid: self.node_id(),
                    request_term: m.request_term,
                    vote_granted: grant,
                }
            ),
            self.node_id(),
            m.source_nid,
        );
        state.message.push(resp);
        //self.send(resp).await?;
        Ok(())
    }

    fn handle_pre_vote_response(
        &mut self,
        source:NID,
        m: PreVoteResp,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        if !self.node_set.can_vote(source) ||
            !self.node_set.can_vote(self.node_id()) {
            return Ok(())
        }
        if self.current_term == m.request_term && self.role == RaftRole::Follower {
            if m.vote_granted {
                let _ = self.follower_pre_vote_granted.insert(m.source_nid, m.request_term);
            }
            self.try_become_candidate(state);
        }
        Ok(())
    }

    fn try_become_candidate(&mut self, state: &mut RaftState<T>) {
        if quorum_agree_vote(
            self.node_id(),
            self.current_term,
            &self.follower_vote_granted,
            self.conf.conf_node_committed(),
            self.conf.conf_node_new(),
        ) {
            if self.role == RaftRole::Follower {
                let _ = self.start_request_vote(state);
            }
        }
    }

    // vote_begin
    fn try_become_leader(&mut self, state: &mut RaftState<T>) -> Res<()> {
        if self.role == RaftRole::Candidate {
            if quorum_agree_vote(
                self.node_id(),
                self.current_term,
                &self.follower_vote_granted,
                self.conf.conf_node_committed(),
                self.conf.conf_node_new(),
            ) {
                self.become_leader(state);
            }
        }
        Ok(())
    }

    fn become_leader(&mut self, state: &mut RaftState<T>) {
        //event_add!(RAFT_FUZZY, Message::new(
        // RaftEvent::BecomeLeader, self.node_id(), self.node_id())); //FUZZY
        self.follower_next_index.clear();
        self.follower_match_index.clear();
        self.tick = 0;
        self.role = RaftRole::Leader;
        self.commit_index = self.snapshot_max_index_term.index;

        state.volatile.role = Some(RaftRole::Leader);

        let last_index = self.last_log_index();
        let nid_set = &self.node_set.nid_log_all;
        for i in nid_set {
            if *i != self.node_id() {
                self.follower_next_index.insert(*i, last_index + 1);
                self.follower_match_index.insert(*i, self.snapshot_max_index_term.index);
            }
        }
    }

    fn last_log_term(&self) -> u64 {
        if self.log.is_empty() {
            self.snapshot_max_index_term.term
        } else {
            self.log[self.log.len() - 1].term
        }
    }

    fn last_log_index(&self) -> u64 {
        if self.log.is_empty() {
            self.snapshot_max_index_term.index
        } else {
            self.log[self.log.len() - 1].index
        }
    }

    fn start_request_vote(&mut self, state: &mut RaftState<T>) -> Res<()> {
        // update current term
        self.current_term += 1;

        self.voted_for = Some(self.node_id());

        // become candidate
        self.role = RaftRole::Candidate;

        // save voted_for and current_term
        state.non_volatile.operation.push(
            NonVolatileWrite::OpUpTermVotedFor {
                term: self.current_term,
                voted_for: self.voted_for,
            });
        state.volatile.role = Some(self.role.clone());

        // vote for itself node
        self.follower_vote_granted.insert(self.node_id(), self.current_term);

        if !self.only_one_node_can_vote() {
            self.send_vote_request(state)?;
        } else {
            self.become_leader(state);
        }
        Ok(())
    }

    fn vote_request_message(
        source: NID,
        dest: NID,
        current_term: u64,
        last_log_term: u64,
        last_log_index: u64,
    ) -> Message<RaftMessage<T>> {
        let m = Message::new(
            RaftMessage::VoteReq(
                MVoteReq {
                    term: current_term,
                    last_log_term,
                    last_log_index,
                }
            ),
            source,
            dest,
        );
        m
    }

    fn send_vote_request(&self, state: &mut RaftState<T>) -> Res<()> {
        let last_log_index = self.last_log_index();
        let last_log_term = self.last_log_term();
        let current_term = self.current_term;
        let from = self.node_id();
        if !self.node_set.can_vote(from) {
            return Ok(())
        }
        // send VoteRequest message to both current configurate node and new configurate node
        let id_set = &self.node_set.nid_vote_all;
        for id in id_set {
            if from != *id {
                let m = Self::vote_request_message(
                    from, *id, current_term, last_log_term, last_log_index,
                );
                state.message.push(m);
            }
        }

        Ok(())
    }

    fn handle_vote_req(
        &mut self,
        source: NID,
        m: MVoteReq,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        if !self.node_set.can_vote(source) ||
            !self.node_set.can_vote(self.node_id()) {
            return Ok(())
        }
        self.update_term(m.term, state)?;
        self.handle_vote_req_gut(source, m, state)?;
        Ok(())
    }

    fn handle_vote_req_gut(
        &mut self,
        source: NID,
        m: MVoteReq,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        if !self.node_set.can_vote(source) ||
            !self.node_set.can_vote(self.node_id()) {
            return Ok(())
        }
        self.vote_req_resp(source, m, state)?;
        Ok(())
    }

    fn vote_req_resp(
        &mut self,
        source: NID,
        m: MVoteReq,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        assert!(self.current_term >= m.term);

        let grant = self.can_grant_vote(m.term,
                                        m.last_log_index,
                                        m.last_log_term,
                                        source);
        if grant {
            self.voted_for = Some(source);
            state.non_volatile.operation.push(
                NonVolatileWrite::OpUpTermVotedFor {
                    term: self.current_term,
                    voted_for: self.voted_for,
                });
            //self.store.set_term_voted(self.current_term, self.voted_for).await?;
        }
        assert!(grant && self.current_term == m.term || !grant);
        let resp = Message::new(
            RaftMessage::VoteResp(
                MVoteResp {
                    term: self.current_term,
                    vote_granted: grant,
                }
            ),
            self.node_id(),
            source,
        );
        state.message.push(resp);
        Ok(())
    }

    fn handle_vote_resp(
        &mut self,
        source: NID,
        m: MVoteResp,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        if !self.node_set.can_vote(source) ||
            !self.node_set.can_vote(self.node_id()) {
            return Ok(())
        }
        self.update_term(m.term, state)?;
        self.handle_vote_resp_gut(source, m, state)?;
        Ok(())
    }

    fn handle_vote_resp_gut(
        &mut self,
        source: NID,
        m: MVoteResp,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        if m.term == self.current_term && (
            self.role == RaftRole::Candidate ||
                self.role == RaftRole::Leader) {
            if m.vote_granted {
                self.follower_vote_granted.insert(source, m.term);
            }
            self.try_become_leader(state)?;
        }

        Ok(())
    }


    fn is_last_log_term_index_ok(&self, last_index: u64, last_term: u64) -> bool {
        if self.log.len() == 0 {
            let term = self.snapshot_max_index_term.term;
            let index = self.snapshot_max_index_term.index;
            last_term > term || (last_term == term && last_index >= index)
        } else {
            let log_off = self.log.len() - 1;
            let term = self.log[log_off].term;
            let index = self.log[log_off].index;
            last_term > term || (last_term == term && last_index >= index)
        }
    }

    fn can_grant_vote(&self, term: u64, last_index: u64, last_term: u64, vote_for: NID) -> bool {
        return self.is_last_log_term_index_ok(last_index, last_term) &&
            (term > self.current_term ||
                (term == self.current_term &&
                    (match self.voted_for {
                        Some(n) => { n == vote_for }
                        None => { true }
                    })
                )
            );
    }

    // vote_end
    async fn restart_for_testing(
        &mut self,
        storage: &Storage<T>,
    ) -> Res<()> {
        self.testing_is_crash = false;
        self.role = RaftRole::Follower;
        self.tick = 0;
        self.follower_match_index.clear();
        self.follower_next_index.clear();
        self.follower_vote_granted.clear();
        self.follower_pre_vote_granted.clear();
        self.follower_term_committed_index.clear();
        self.follower_conf_committed.clear();
        self.follower_conf_new.clear();
        self.recovery_state(storage).await?;

        Ok(())
    }

    fn compact_log_inner(&mut self, index: u64, state: &mut RaftState<T>) -> Res<()> {
        assert!(index > self.snapshot_max_index_term.index);
        let n = (index - self.snapshot_max_index_term.index) as usize;
        assert!(n >= 1);

        let log_entries: Vec<_> = self.log.splice(0..=n - 1, vec![]).collect();
        if !log_entries.is_empty() {
            let entry: &LogEntry<T> = log_entries.last().unwrap();
            self.snapshot_max_index_term.index = entry.index;
            self.snapshot_max_index_term.term = entry.term;
            state.non_volatile.operation.push(
                NonVolatileWrite::OpCompactLog(
                    log_entries));
        }
        Ok(())
    }


    async fn append_entries(&self, state: &mut RaftState<T>, storage: &Storage<T>) -> Res<()> {
        let nid_set = &self.node_set.nid_log_all;
        for node_id in nid_set {
            if *node_id != self.node_id() {
                self.append_entries_to_node(self.node_id(), node_id.clone(), state, storage).await?;
            }
        }

        Ok(())
    }

    async fn append_entries_to_node(&self, source: NID, dest: NID, state: &mut RaftState<T>, storage: &Storage<T>) -> Res<()> {
        let rm = self.make_append_req_message(source, dest, storage).await?;
        let m = Message::new(
            rm,
            self.node_id(),
            dest,
        );
        state.message.push(m);
        Ok(())
    }

    fn prev_log_index_of_node(&self, id: NID) -> u64 {
        let opt_i = self.follower_next_index.get(&id);
        let i = match opt_i {
            Some(i) => {
                i.clone()
            }
            None => {
                // the default value of next_index is last_log_index + 1
                self.last_log_index() + 1
            }
        };
        if i >= 1 {
            i - 1
        } else {
            error!("next index must >= 1");
            0
        }
    }

    fn log_term(&self, index: u64) -> u64 {
        if index <= self.snapshot_max_index_term.index {
            return self.snapshot_max_index_term.term;
        } else {
            let i = (index - self.snapshot_max_index_term.index - 1) as usize;
            if i < self.log.len() {
                return self.log[i].term;
            } else {
                panic!("log index error");
            }
        }
    }

    // select log entries started  from index with max entries
    fn select_log_entries(&self, index: u64, max: u64) -> Vec<LogEntry<T>> {
        let mut vec = vec![];
        if index <= self.snapshot_max_index_term.index {
            panic!("error log index, must >= 1");
        }
        let start = (index - self.snapshot_max_index_term.index - 1) as usize;
        let end = min(start + max as usize, self.log.len());
        for i in start..end {
            vec.push(self.log[i].clone())
        }
        return vec;
    }


    async fn make_apply_snapshot_message(&self, _source: NID, index: u64, storage: &Storage<T>) -> Res<RaftMessage<T>> {
        let id = "".to_string();
        let term = self.current_term;
        let values = storage.read_snapshot_value(index, None).await?;
        let (begin_index, end_index) = if values.is_empty() {
            (0, 0)
        } else {
            let mut begin_index = u64::MAX;
            let mut end_index = u64::MIN;
            for i in values.iter() {
                if begin_index > i.index {
                    begin_index = i.index
                }
                if end_index < i.index {
                    end_index = i.index
                }
            }
            (begin_index, end_index)
        };
        assert_eq!(end_index, self.snapshot_max_index_term.index);
        Ok(RaftMessage::ApplyReq(
            MApplyReq {
                term,
                id,
                begin_index,
                end_index,
                snapshot: Snapshot {
                    index: self.snapshot_max_index_term.index,
                    term: self.snapshot_max_index_term.term,
                    entries: MTSet::from_vec(values),
                },
            }))
    }

    /// TLA+ {D240504N-MsgAppendRequest}
    async fn make_append_req_message(&self, source: NID, dest: NID, storage: &Storage<T>) -> Res<RaftMessage<T>> {
        let prev_log_index = self.prev_log_index_of_node(dest);
        if prev_log_index < self.snapshot_max_index_term.index {
            Ok(self.make_apply_snapshot_message(source, prev_log_index + 1, storage).await?)
        } else {
            let term = self.current_term;
            let prev_log_term = self.log_term(prev_log_index);
            let commit_index = self.commit_index;

            let log_entries = self.select_log_entries(
                prev_log_index + 1, self.max_append_entries);
            if let Some(l) = log_entries.first() {
                assert_eq!(l.index, prev_log_index + 1);
            }
            let m = MAppendReq {
                term,
                prev_log_index,
                prev_log_term,
                log_entries,
                commit_index,
            };
            Ok(RaftMessage::AppendReq(m))
        }
    }

    fn is_prev_log_entry_ok(&self, prev_log_index: u64, prev_log_term: u64) -> bool {
        if prev_log_index == 0 &&
            self.snapshot_max_index_term.index == 0 &&
            self.log.len() == 0 {
            return true;
        }

        if self.snapshot_max_index_term.index >= prev_log_index {
            self.snapshot_max_index_term.index == prev_log_index
                && self.snapshot_max_index_term.term == prev_log_term
        } else {
            let i = (prev_log_index - self.snapshot_max_index_term.index) as usize;
            assert!(i > 0);
            let ret = i <= self.log.len()
                && self.log[i - 1].term == prev_log_term
                && self.log[i - 1].index == prev_log_index;
            ret
        }
    }


    async fn follower_append_entries(
        &mut self, prev_index: u64,
        log_entries: Vec<LogEntry<T>>,
        state: &mut RaftState<T>,
    ) -> Res<u64> {
        let mut _prev_index = prev_index;
        if log_entries.len() == 0 {
            return Ok(_prev_index);
        }

        let offset_start = (_prev_index - self.snapshot_max_index_term.index) as usize;
        if self.log.len() < offset_start {
            panic!("log index error");
        }
        let (to_append, offset_write) = if self.log.len() == offset_start {
            // *prev_index* is exactly equal to current log
            // append all log entries, start at offset *offset_start*
            (log_entries, offset_start as u64)
        } else {
            assert!(self.log.len() > offset_start);
            let mut entries_inconsistency_index = None;
            for i in 0..log_entries.len() {
                if self.log.len() == offset_start + i {
                    break;
                }
                let e1 = &self.log[offset_start + i];
                let e2 = &log_entries[i];
                if e1.term != e2.term {
                    // the first inconsistency index
                    // the later log would be truncated
                    entries_inconsistency_index = Some(i);
                    break;
                } else {
                    _prev_index += 1;
                    assert_eq!(_prev_index, e1.index);
                }
                assert_eq!(e1.index, e2.index);
            }
            match entries_inconsistency_index {
                Some(i) => {
                    // *self.log* and *[0..i)* entries of *log_entries* are consistent, and they
                    // not need to be appended; log append start at offset *offset_start + i*
                    let mut to_append = log_entries;
                    // to_append, remove index range 0..i, and left index range i..
                    to_append = to_append.drain(i..).collect();
                    (to_append, (offset_start + i) as u64)
                }
                None => {
                    // *self.log* and *log_entries* received are consistent, and the equal prefix entries
                    // are not need to be appended;
                    let pos = self.log.len() - offset_start;
                    let mut to_append = log_entries;
                    let pos = min(pos, to_append.len());
                    let to_append1 = to_append.drain(pos..).collect();
                    (to_append1, (offset_start + pos) as u64)
                }
            }
        };

        let match_index = self.write_log_entries(_prev_index, offset_write, to_append, state).await?;

        Ok(match_index)
    }

    async fn write_log_entries(
        &mut self,
        prev_index: u64,
        offset_write_pos: u64,
        log_entries: Vec<LogEntry<T>>,
        state: &mut RaftState<T>,
    ) -> Res<u64> {
        let pos = offset_write_pos as usize;
        return if self.log.len() < pos || log_entries.is_empty() {
            // match index is prev index
            Ok(prev_index)
        } else {
            let mut to_append = log_entries.clone();
            self.log.drain(pos..);
            self.log.append(&mut to_append);

            let len = log_entries.len() as u64;
            let op = NonVolatileWrite::OpWriteLog {
                prev_index,
                entries: log_entries,
                opt: WriteEntriesOpt {
                    truncate_left: false,
                    truncate_right: true,
                },
            };
            state.non_volatile.operation.push(op);
            // self.store.write_log_entries(prev_index,
            // log_entries, WriteEntriesOpt::default()).await?;
            Ok(prev_index + len)
        };
    }

    async fn handle_append_req(
        &mut self,
        source: NID,
        m: MAppendReq<T>,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        if !self.node_set.can_log(source) ||
            !self.node_set.can_log(self.node_id()) {
            return Ok(())
        }
        self.update_term(m.term, state)?;
        self.handle_append_req_gut(source, m, state).await?;
        Ok(())
    }

    async fn handle_append_req_gut(
        &mut self,
        source: NID,
        m: MAppendReq<T>,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        self.tick = 0;
        self.append_req_resp(source, m, state).await?;
        Ok(())
    }

    async fn append_req_resp(
        &mut self,
        source: NID,
        m: MAppendReq<T>,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        let log_ok = self.is_prev_log_entry_ok(m.prev_log_index, m.prev_log_term);
        if self.current_term > m.term || (
            self.current_term == m.term &&
                self.role == RaftRole::Follower &&
                !log_ok
        ) {
            // reject request
            let next_index = if self.snapshot_max_index_term.index > m.prev_log_index {
                self.snapshot_max_index_term.index  //FUZZY
            } else {
                0
            };

            // when reject the vote, setting commit_index = 0, match_index = 0
            let resp = MAppendResp {
                term: self.current_term,
                append_success: false,
                commit_index: 0,
                match_index: 0,
                next_index,
            };
            let m = Message::new(
                RaftMessage::AppendResp(resp),
                self.node_id(),
                source,
            );
            state.message.push(m);
        } else if self.current_term == m.term &&
            self.role == RaftRole::Candidate {
            // *become candidate state
            self.become_follower(state)?;
        } else if self.current_term == m.term &&
            self.role == RaftRole::Follower &&
            log_ok {
            assert!(self.snapshot_max_index_term.index <= m.prev_log_index);
            // append ok

            let match_index = self.follower_append_entries(
                m.prev_log_index, m.log_entries, state).await?;

            // TLA+ {D240524N- update commit index}
            let mut commit_index = m.commit_index;
            if commit_index > self.commit_index {
                if !commit_index <= match_index {
                    commit_index = match_index;
                }
                self.set_commit_index(m.commit_index, state)?;
            } else {
                commit_index = self.commit_index;
            }
            let resp = MAppendResp {
                term: self.current_term,
                append_success: true,
                commit_index,
                match_index,
                next_index: 0,
            };

            let m = Message::new(
                RaftMessage::AppendResp(resp),
                self.node_id(),
                source,
            );
            state.message.push(m);
        } else {};
        Ok(())
    }

    fn handle_append_resp(
        &mut self,
        source: NID,
        m: MAppendResp,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        if !self.node_set.can_log(source) ||
            !self.node_set.can_vote(self.node_id()) {
            return Ok(())
        }
        self.update_term(m.term, state)?;
        self.handle_append_resp_gut(source, m, state)?;
        Ok(())
    }

    fn update_follower_term_commit_index(&mut self, source:NID, term:u64, commit_index:u64) {
        let opt = self.follower_term_committed_index.get_mut(&source);
        if let Some(ti) = opt {
            if ti.term != term || ti.index != commit_index {
                ti.term = term;
                ti.index = commit_index;
            }
        } else {
            self.follower_term_committed_index.insert(source, TermIndex {term, index:commit_index});
        }
    }

    fn handle_append_resp_gut(
        &mut self,
        source: NID,
        m: MAppendResp,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        if !(self.current_term == m.term && self.role == RaftRole::Leader) {
            return Ok(());
        }

        if m.append_success {
            self.follower_next_index.insert(source, m.match_index + 1);
            assert!(self.last_log_index() >= m.match_index);
            let opt = self.follower_match_index.insert(source, m.match_index);
            let _advance_commit = match opt {
                Some(v) => { m.match_index > v }
                None => { true }
            };

            self.advance_commit_index(state)?;
            self.update_follower_term_commit_index(source, m.term, m.commit_index);
        } else {
            let last_log_index = self.last_log_index();
            let opt = self.follower_next_index.get_mut(&source);
            match opt {
                Some(next_index) => {
                    if m.next_index > 0 {
                        if last_log_index + 1 < m.next_index {
                            error!("may be this is a stale leader?");
                        }
                        let new_index = min(m.next_index, last_log_index + 1);
                        *next_index = new_index;
                    } else {
                        if *next_index > 1 {
                            *next_index -= 1;
                        }
                    }
                }
                None => {
                    if m.next_index > 0 {
                        self.follower_next_index.insert(source, m.next_index);
                    }
                }
            }
        }
        Ok(())
    }

    fn advance_commit_index(&mut self, state: &mut RaftState<T>) -> Res<()> {
        assert_eq!(self.role, RaftRole::Leader);
        // add last index of this nodes
        let leader_last_index = self.last_log_index();

        let new_commit_index = quorum_agree_match_index(
            self.node_id(),
            leader_last_index,
            &self.follower_match_index,
            self.conf.conf_node_committed(),
            self.conf.conf_node_new(),
        );
        self.set_commit_index(new_commit_index, state)?;
        Ok(())
    }

    fn set_commit_index(&mut self, new_commit_index: u64, state: &mut RaftState<T>) -> Res<()> {
        if self.commit_index < new_commit_index {
            self.commit_index = new_commit_index;
            state.non_volatile.operation.push(NonVolatileWrite::OpUpCommitIndex(new_commit_index));
        }
        Ok(())
    }

    //  TLA+ {D240524N- HandleApplySnapshotReq}
    fn handle_apply_snapshot_req(
        &mut self,
        source: NID,
        _dest: NID,
        m: MApplyReq<T>,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        if !self.node_set.can_log(source) ||
            !self.node_set.can_log(self.node_id()) {
            return Ok(())
        }
        self.update_term(m.term, state)?;
        self.apply_snapshot_gut(source, m, state)?;
        Ok(())
    }

    fn apply_snapshot_gut(
        &mut self,
        source: NID,
        m: MApplyReq<T>,
        state: &mut RaftState<T>,
    ) -> Res<()> {
        if self.current_term != m.term {
            return Ok(());
        }
        if m.begin_index >= m.end_index {
            return Ok(());
        }
        if m.end_index < 1 {
            return Ok(());
        }

        let end_index = m.end_index;

        let match_index = if end_index >= self.last_log_index()
            && end_index > self.commit_index {
            let match_index = end_index;
            match_index
        } else {
            return Ok(());
        };
        self.snapshot_max_index_term.index = m.snapshot.index;
        self.snapshot_max_index_term.term = m.snapshot.term;
        state.non_volatile.operation.push(
            NonVolatileWrite::OpApplySnapshot(SnapshotRange {
                begin_index: m.begin_index,
                end_index: m.end_index,
                entries: m.snapshot.entries,
            })
        );
        state.non_volatile.operation.push(
            NonVolatileWrite::OpWriteLog {
                prev_index: m.end_index,
                entries: vec![],
                opt: WriteEntriesOpt {
                    truncate_left: true,
                    truncate_right: true,
                },
            }
        );
        self.log.clear();
        let resp = Message::new(
            RaftMessage::ApplyResp(
                MApplyResp {
                    term: m.term,
                    match_index,
                    id: m.id,
                }
            ),
            self.node_id(),
            source,
        );
        state.message.push(resp);

        Ok(())
    }

    fn handle_apply_snapshot_resp(
        &mut self,
        source: NID,
        m: MApplyResp,
    ) -> Res<()> {
        if !self.node_set.can_log(source) ||
            !self.node_set.can_vote(self.node_id()) {
            return Ok(())
        }
        if self.current_term != m.term {
            return Ok(())
        }

        if let Some(index) = self.follower_next_index.get(&source) {
            if *index < m.match_index + 1 {
                self.follower_next_index.insert(source, m.match_index + 1);
            }
        } else {
            self.follower_next_index.insert(source, m.match_index + 1);
        }

        if let Some(index) = self.follower_match_index.get(&source) {
            if *index < m.match_index {
                self.follower_match_index.insert(source, m.match_index);
            }
        } else {
            self.follower_match_index.insert(source, m.match_index);
        }

        return Ok(());
    }

    fn update_term(&mut self, term: u64, state: &mut RaftState<T>) -> Res<()> {
        if self.current_term < term {
            self.current_term = term;
            self.become_follower(state)?;
        }

        Ok(())
    }

    fn become_follower(&mut self, state: &mut RaftState<T>) -> Res<()> {
        if self.role != RaftRole::Learner {
            self.role = RaftRole::Follower;
            state.volatile.role = Some(RaftRole::Leader);
        }
        self.tick = 0;
        self.voted_for = None;

        state.non_volatile.operation.push(
            NonVolatileWrite::OpUpTermVotedFor {
                term: self.current_term,
                voted_for: self.voted_for,
            });

        self.follower_match_index.clear();
        self.follower_next_index.clear();
        self.follower_vote_granted.clear();
        Ok(())
    }

    async fn recovery_state_inner(
        storage: &Storage<T>
    ) -> Res<(ConfNodeValue, ConfNodeValue, Vec<LogEntry<T>>, TermIndex, u64, Option<u64>)>
    {
        let ((conf_committed, version_committed),
            (conf_new, version_new)) = storage.conf().await?;
        let c_committed = ConfNodeValue::new(
            conf_committed, version_committed.term,
            version_committed.version, version_committed.index);
        let c_new = ConfNodeValue::new(
            conf_new, version_new.term,
            version_new.version, version_new.index);

        let index_term = storage.snapshot_index_term().await?;
        let log = storage.read_log_entries(
            Bound::Unbounded, Bound::Unbounded).await?;
        let (term, voted_for) = storage.term_and_voted_for().await?;
        Ok((c_committed, c_new, log, index_term, term, voted_for))
    }

    fn update_conf_local(& mut self) {
        (self.node_set.committed_log, self.node_set.new_log) = self.conf_nid_log();
        (self.node_set.committed_vote, self.node_set.new_vote) = self.conf_nid_vote();
        self.node_set.nid_log_all = self.node_set.committed_log.union(&self.node_set.new_log).cloned().collect();
        self.node_set.nid_vote_all = self.node_set.committed_vote.union(&self.node_set.new_vote).cloned().collect();
    }

    async fn recovery_state(&mut self, storage: &Storage<T>) -> Res<()> {
        let (conf_committed, conf_new,
            log, snapshot,
            term, voted_for) =
            Self::recovery_state_inner(storage).await?;

        trace!("state machine node:{} recovery success", self.node_id());
        self.log = log;
        self.snapshot_max_index_term = snapshot;
        self.current_term = term;
        self.voted_for = voted_for;
        self.conf.update_conf_committed(conf_committed);
        self.conf.update_conf_new(conf_new);
        self.update_conf_local();
        self.commit_index = self.snapshot_max_index_term.index;
        Ok(())
    }

    fn state_setup(&mut self, hs: MRaftState<T>, state: &mut RaftState<T>) -> Res<()> { //DTM
        self.current_term = hs.current_term; //DTM
        self.voted_for = hs.voted_for; //DTM
        state.non_volatile.operation.push(
            NonVolatileWrite::OpUpTermVotedFor {
                term: self.current_term,
                voted_for: self.voted_for,
            });

        self.log = hs.log.clone(); //DTM
        state.non_volatile.operation.push(
            NonVolatileWrite::OpWriteLog {
                prev_index: 0,
                entries: self.log.clone(),
                opt: WriteEntriesOpt {
                    truncate_left: true,
                    truncate_right: true,
                },
            }
        );

        self.snapshot_max_index_term = TermIndex { index: hs.snapshot.index, term: hs.snapshot.term }; //DTM
        let end_index = if hs.snapshot.index == 0 {
            0
        } else {
            hs.snapshot.index
        };
        state.non_volatile.operation.push(
            NonVolatileWrite::OpApplySnapshot(
                SnapshotRange {
                    begin_index: 0,
                    end_index,
                    entries: hs.snapshot.entries,
                }
            )
        );

        self.role = hs.role; //DTM
        state.volatile.role = Some(self.role);

        match hs.log.first() {  //DTM
            Some(l) => { //DTM
                assert_eq!(hs.snapshot.index + 1, l.index); //DTM
            } //DTM
            None => {} //DTM
        }
        self.commit_index = hs.commit_index; //DTM
        state.non_volatile.operation.push(
            NonVolatileWrite::OpUpCommitIndex(self.commit_index)
        );

        self.follower_match_index = hs.follower_match_index.to_map(); //DTM
        self.follower_next_index = hs.follower_next_index.to_map(); //DTM
        self.follower_vote_granted = hs.follower_vote_granted.to_map();
        let old = hs.follower_term_commit_index.to_map();
        let m =
            old.into_iter().map(|kv| {
                return (kv.0.clone(), TermIndex {
                    index: kv.1.index,
                    term: kv.1.term,
                });
            }).collect();

        self.follower_term_committed_index = m;

        for (nid, pair) in hs.follower_conf.to_map().into_iter() {
            self.follower_conf_committed.insert(nid, pair.conf_committed);
            self.follower_conf_new.insert(nid, pair.conf_new);
        }

        let value_committed = self.conf.conf_node_value_committed().value.clone();
        let conf_committed = ConfNodeValue::from_conf_node_value(
            hs.conf_committed, value_committed);
        let value_new = self.conf.conf_node_value_new().value.clone();
        let conf_new = ConfNodeValue::from_conf_node_value(
            hs.conf_new, value_new);
        self.conf.update_conf_committed(conf_committed);
        self.conf.update_conf_new(conf_new);
        self.update_conf_local();
        state.non_volatile.operation.push(
            NonVolatileWrite::OpUpConfCommitted {
                value: self.conf.conf_node_value_committed().value.clone(),
                version: self.conf.conf_node_value_committed().node.conf_version().clone(),
            }
        );
        state.non_volatile.operation.push(
            NonVolatileWrite::OpUpConfNew {
                value: self.conf.conf_node_value_new().value.clone(),
                version: self.conf.conf_node_value_new().node.conf_version().clone(),
            }
        );
        //self.follower_vote_granted = hs.vote_granted.to_set(); //DTM
        Ok(()) //DTM
    }

    fn state_check(&self, hs: MRaftState<T>) -> Res<()> { //DTM
        assert_eq!(self.role, hs.role);
        assert_eq!(self.current_term, hs.current_term); //DTM
        assert_eq!(self.voted_for, hs.voted_for); //DTM
        assert_eq!(self.log, hs.log); //DTM
        assert_eq!(self.snapshot_max_index_term, TermIndex { index: hs.snapshot.index, term: hs.snapshot.term }); //DTM
        assert_eq!(self.commit_index, hs.commit_index); //DTM
        assert_eq!(self.conf.conf_node_value_committed().node, hs.conf_committed);
        assert_eq!(self.conf.conf_node_value_new().node, hs.conf_new);
        for (k, v) in hs.follower_next_index.to_map() {
            if let Some(_v)  = self.follower_next_index.get(&k) {
                assert_eq!(v, v);
            }
        }

        for (k, v) in hs.follower_match_index.to_map() {
            if let Some(_v)  = self.follower_match_index.get(&k) {
                assert_eq!(v, v);
            }
        }

        Ok(()) //DTM
    }

    async fn handle_client_value_req(
        &mut self,
        source_id: NID, m: MClientReq<T>,
        state: &mut RaftState<T>,
        storage: &Storage<T>,
    ) -> Res<()> {
        if !self.node_set.can_vote(self.node_id()) {
            return Ok(())
        }
        let need_resp = m.wait_commit || m.wait_write_local;
        let id = m.id.clone();
        let from_client_request = m.from_client_request;
        let r = self.client_req_resp(source_id, m, state, storage).await;
        if need_resp {
            let resp = match r {
                Ok(opt) => {
                    match opt {
                        Some(e) => { e }
                        None => {
                            MClientResp {
                                id,
                                source_id: self.node_id(),
                                index: 0,
                                term: 0,
                                error: RCR_ERR_RESP,
                                info: "".to_string(),
                            }
                        }
                    }
                }
                Err(e) => {
                    MClientResp {
                        id,
                        source_id: self.node_id(),
                        index: 0,
                        term: 0,
                        error: RCR_ERR_RESP,
                        info: e.to_string(),
                    }
                }
            };
            let msg = Message::new(
                RaftMessage::ClientResp(resp),
                self.node_id(),
                source_id);

            if from_client_request {
                state.message.push(msg);
            } else {
                state.message.push(msg);
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    async fn client_req_resp(
        &mut self,
        _source_nid: NID,
        m: MClientReq<T>,
        state: &mut RaftState<T>,
        storage: &Storage<T>,
    ) -> Res<Option<MClientResp>> {
        if self.role != RaftRole::Leader {
            return Ok(None);
        }

        let (index, term) = self.client_request_write_value_gut(m.value, state, storage).await?;
        if m.wait_write_local || m.wait_commit {
            let _m = MClientResp {
                id: m.id,
                source_id: self.node_id(),
                index,
                term,
                error: RCR_OK,
                info: "".to_string(),
            };
            //state.message.push(_m.clone());
            return Ok(Some(_m));
        }

        Ok(None)
    }

    async fn client_request_write_value_gut(
        &mut self, value: T,
        state: &mut RaftState<T>,
        storage: &Storage<T>,
    ) -> Res<(u64, u64)> {
        assert_eq!(self.role, RaftRole::Leader);
        let last_index = self.last_log_index();
        let index = last_index + 1;
        let term = self.current_term;
        let entry = LogEntry {
            term,
            index,
            value,
        };
        self.write_log_entries(last_index, self.log.len() as u64, vec![entry], state).await?;
        if self.conf.conf_committed_value().millisecond_tick > 0 {
            self.append_entries(state, storage).await?;
        }
        if self.only_one_node_can_vote() {
            self.advance_commit_index(state)?;
        }
        Ok((index, term))
    }


    /// TLA+ {D240422N-MsgUpdateConfReq}
    fn msg_update_conf_req(
        source: NID,
        dest: NID,
        term: u64,
        conf: &RaftConf,
    ) -> Message<RaftMessage<T>> {
        Message::<_>::new(
            RaftMessage::UpdateConfReq(MUpdateConfReq {
                term,
                conf_committed: conf.conf_node_value_committed().clone(),
                conf_new: conf.conf_node_value_new().clone(),
            }),
            source, dest)
    }

    /// TLA+ {D240422N-_MsgUpdateConfResp}
    fn msg_update_conf_resp(
        source: NID,
        dest: NID,
        term: u64,
        ctv_committed: ConfVersion,
        ctv_new: ConfVersion,
    ) -> Message<RaftMessage<T>> {
        Message::new(
            RaftMessage::UpdateConfResp(MUpdateConfResp {
                term,
                conf_committed: ctv_committed,
                conf_new: ctv_new,
            }),
            source, dest)
    }

    /// TLA+ {D240422N-LeaderSendUpdateConfToFollower}
    fn send_update_conf_to_follower(&self, state: &mut RaftState<T>) -> Res<()> {
        if self.role != RaftRole::Leader {
            return Ok(());
        }

        let nid_set = &self.node_set.nid_log_all;
        let this_nid = self.node_id();
        for nid in nid_set {
            if *nid != this_nid {
                let m = Self::msg_update_conf_req(
                    this_nid,
                    *nid,
                    self.current_term,
                    &self.conf,
                );
                state.message.push(m);
            }
        }
        Ok(())
    }

    /// TLA+ {D240422N-HandleUpdateConfReq}
    fn handle_update_conf_req(
        &mut self, source: NID, msg: MUpdateConfReq, state: &mut RaftState<T>) -> Res<()> {
        if !self.node_set.can_log(self.node_id()) ||
            !self.node_set.can_log(source)
        {
            return Ok(())
        }

        if self.current_term != msg.term {
            return Ok(());
        }

        // only can update config when the term is equal to current term
        assert_eq!(self.current_term, msg.term);
        let update = Self::_update_conf(&mut self.conf, msg, state)?;
        if update {
            self.update_conf_local();
        }
        let resp = Self::msg_update_conf_resp(
            self.node_id(), source,
            self.current_term,
            self.conf.conf_node_value_committed().term_version().clone(),
            self.conf.conf_node_new().clone().conf_version().clone());
        state.message.push(resp);
        Ok(())
    }

    fn __update_conf<const COMMITTED:bool>(
            conf:&mut RaftConf,
            state:&mut RaftState<T>,
            conf_node_value:ConfNodeValue) {
        if COMMITTED {
            let op =  NonVolatileWrite::OpUpConfCommitted {
                value:conf_node_value.value.clone(),
                version:conf_node_value.node.conf_version().clone(),
            };
            state.non_volatile.operation.push(op);
            conf.update_conf_committed(conf_node_value);
        } else {
            let op = NonVolatileWrite::OpUpConfNew {
                value:conf_node_value.value.clone(),
                version:conf_node_value.node.conf_version().clone(),
            };
            state.non_volatile.operation.push(op);
            conf.update_conf_new(conf_node_value);
        };

    }

    /// TLA+ {D240422N-_UpdateConf}
    fn _update_conf(conf: &mut RaftConf, msg: MUpdateConfReq, state:&mut RaftState<T>) -> Res<bool> {
        let mut update = false;
        if conf.conf_node_value_committed().node.conf_version().lt(msg.conf_committed.node.conf_version()) {
            Self::__update_conf::<true>(conf, state, msg.conf_committed);
            update = true;
        }
        if conf.conf_node_value_new().node.conf_version().lt(msg.conf_new.node.conf_version()) {
            Self::__update_conf::<false>(conf, state, msg.conf_new);
            update = true;
        }
        Ok(update)
    }

    /// TLA+ {D240422N-LeaderReConfBegin}
    fn leader_re_conf_begin(&mut self, state:&mut RaftState<T>, conf_value: ConfValue, nid_vote: Vec<NID>, nid_log: Vec<NID>) -> Res<()> {
        if self.role != RaftRole::Leader {
            //  only leader can request update conf
            return Ok(());
        }

        if self.conf.conf_node_value_committed().term_version().ne(self.conf.conf_node_new().conf_version()) {
            // updating conf in progress, cannot issue a new update operation
            return Ok(());
        }

        assert_eq!(self.conf.conf_node_value_committed().value.cluster_name,
                   conf_value.cluster_name);

        let ctv = ConfVersion {
            term: self.current_term,
            version: self.conf.conf_node_new().conf_version().version + 1,
            index: self.commit_index,
        };


        let conf_node = ConfNode {
            conf_version: ctv,
            nid_vote: MTSet::from_vec(nid_vote),
            nid_log: MTSet::from_vec(nid_log),
        };
        let conf = ConfNodeValue {
            node: conf_node,
            value: conf_value,
        };

        Self::__update_conf::<false>(&mut self.conf, state, conf);
        self.update_conf_local();
        Ok(())
    }


    /// TLA+ {D240423N-LeaderReConfCommit}
    fn leader_re_conf_commit(&mut self, state:&mut RaftState<T>) -> Res<()> {
        if self.can_re_conf_commit()? {
            let conf = self.conf.conf_node_value_new().clone();
            Self::__update_conf::<true>(&mut self.conf, state, conf);
            self.update_conf_local();
        }
        Ok(())
    }


    fn can_re_conf_commit(&self) -> Res<bool> {
        if self.role != RaftRole::Leader {
            //  only leader can request update conf
            return Ok(false);
        }

        if self.conf.conf_node_value_committed().term_version().eq(self.conf.conf_node_new().conf_version()) {
            // updating conf is done
            return Ok(false);
        }

        let check_old_conf_accepted_by_old_quorum = quorum_check_conf_term_version(
            self.node_id(),
            self.conf.conf_node_value_committed().nid_vote(),
            self.conf.conf_node_value_committed().term_version(),
            &self.follower_conf_committed,
        );
        if !check_old_conf_accepted_by_old_quorum {
            return Ok(false);
        }

        let check_new_conf_accepted_by_old_quorum = quorum_check_conf_term_version(
            self.node_id(),
            self.conf.conf_node_value_committed().nid_vote(),
            self.conf.conf_node_new().conf_version(),
            &self.follower_conf_new,
        );
        if !check_new_conf_accepted_by_old_quorum {
            return Ok(false);
        }

        let check_new_conf_accepted_by_new_quorum = quorum_check_conf_term_version(
            self.node_id(),
            self.conf.conf_node_value_new().nid_vote(),
            self.conf.conf_node_value_new().term_version(),
            &self.follower_conf_new,
        );
        if !check_new_conf_accepted_by_new_quorum {
            return Ok(false);
        }

        let check_term_and_committed_index_accepted_by_new_quorum = quorum_check_term_commit_index(
            self.node_id(),
            self.conf.conf_node_value_new().nid_vote(),
            self.current_term,
            self.conf.conf_node_value_new().node.conf_version().index,
            &self.follower_term_committed_index,
        );
        if !check_term_and_committed_index_accepted_by_new_quorum {
            return Ok(false);
        }

        Ok(true)
    }


    /// TLA+ {D240422N-HandleUpdateConfResp}
    fn handle_update_conf_resp(&mut self, source: NID, msg: MUpdateConfResp) -> Res<()> {
        if !self.node_set.can_vote(self.node_id()) ||
            !self.node_set.can_log(source)
        {
            return Ok(())
        }
        if self.role != RaftRole::Leader {
            return Ok(());
        }
        if self.current_term != msg.term {
            return Ok(());
        }
        let _ = self.follower_conf_committed.insert(source, msg.conf_committed);
        let _ = self.follower_conf_new.insert(source, msg.conf_new);
        Ok(())
    }

    fn node_id(&self) -> NID {
        self.conf.node_id()
    }

    pub async fn check_storage(&self, storage: &Storage<T>) -> Res<()> {
        let it = storage.snapshot_index_term().await?;
        assert_eq!(it, self.snapshot_max_index_term);
        let log = storage.read_log_entries(Bound::Unbounded, Bound::Unbounded).await?;
        assert_eq!(log, self.log);
        Ok(())
    }
}

impl NodeSet {
    fn can_vote(&self, nid: NID) -> bool {
        self.committed_vote.contains(&nid) && self.committed_log.contains(&nid)
    }

    fn can_log(&self, nid:NID) -> bool {
        self.committed_log.contains(&nid) && self.committed_log.contains(&nid)
    }
}