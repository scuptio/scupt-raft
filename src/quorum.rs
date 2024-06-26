use std::cmp::min;
use std::collections::HashMap;

use scupt_util::node_id::NID;

use crate::conf_node::ConfNode;
use crate::conf_version::ConfVersion;
use crate::term_index::TermIndex;

pub fn quorum_agree_vote(
    leader_nid: NID,
    current_term: u64,
    follower_vote_granted: &HashMap<NID, u64>,
    conf_committed: &ConfNode,
    conf_new: &ConfNode,
) -> bool {
    let voter = conf_committed.nid_vote.vec();
    let filter = |id| {
        let opt = follower_vote_granted.get(&id);
        match opt {
            Some(term) => {
                *term == current_term
            }
            None => { false }
        }
    };
    let agreed_num = _count(voter, leader_nid, filter) + 1;
    if conf_committed.conf_version().eq(conf_new.conf_version()) {
        return agreed_num * 2 > voter.len();
    } else {
        let voter_new = conf_new.nid_vote.vec();
        let agreed_num_new = _count(voter_new, leader_nid, filter) + 1;
        return agreed_num * 2 > voter.len() && agreed_num_new * 2 > voter_new.len();
    }
}

pub fn quorum_agree_match_index(
    leader_nid: NID,
    leader_last_index: u64,
    follower_match_index: &HashMap<NID, u64>,
    conf_committed: &ConfNode,
    conf_new: &ConfNode,
) -> u64 {
    let filter = |id| {
        let opt = follower_match_index.get(&id);
        match opt {
            Some(i) => {
                *i
            }
            None => { 0 }
        }
    };
    let quorum_agreed = {
        let node = conf_committed.nid_vote.vec();
        let mut agreed_index = _index(node, leader_nid, filter);
        agreed_index.push(leader_last_index);
        let quorum_agreed = _majority_agree_index(agreed_index, node.len());
        quorum_agreed
    };
    if conf_committed.conf_version().eq(conf_new.conf_version()) {
        quorum_agreed
    } else {
        let node_new = conf_new.nid_vote.vec();
        let mut agreed_index_new = _index(node_new, leader_nid, filter);
        agreed_index_new.push(leader_last_index);
        let quorum_agreed_new = _majority_agree_index(agreed_index_new, node_new.len());
        min(quorum_agreed, quorum_agreed_new)
    }
}

pub fn quorum_check_conf_term_version(
    leader_nid: NID,
    nid_set: &Vec<NID>,
    term_version: &ConfVersion,
    follower_conf: &HashMap<NID, ConfVersion>,
) -> bool {
    let filter = |_n| {
        let opt = follower_conf.get(&_n);
        match opt {
            Some(tv) => {
                *tv == *term_version
            }
            None => {
                false
            }
        }
    };
    let agreed_num = _count(nid_set, leader_nid, filter) + 1;
    return agreed_num * 2 > nid_set.len();
}

pub fn quorum_check_term_commit_index(
    leader_nid: NID,
    nid_set: &Vec<NID>,
    leader_term: u64,
    leader_conf_commit_index: u64,
    follower_term_and_committed_index: &HashMap<NID, TermIndex>,
) -> bool {
    let filter = |_n| {
        let opt = follower_term_and_committed_index.get(&_n);
        match opt {
            Some(term_index) => {
                term_index.term == leader_term && term_index.index >= leader_conf_commit_index
            }
            None => { false }
        }
    };
    let agreed_num = _count(nid_set, leader_nid, filter) + 1;
    return agreed_num * 2 > nid_set.len();
}


fn _count<F: Fn(NID) -> bool>(node: &Vec<NID>, exclude: NID, cond: F) -> usize {
    let mut n = 0;
    for id in node {
        if *id != exclude {
            if cond(*id) {
                n += 1
            }
        }
    }
    n
}

fn _index<F: Fn(NID) -> u64>(node: &Vec<NID>, exclude: NID, get_index: F) -> Vec<u64> {
    let mut vec = vec![];
    for id in node {
        if *id != exclude {
            vec.push(get_index(*id))
        }
    }
    vec
}

fn _majority_agree_index(
    vec: Vec<u64>,
    nodes_num: usize,
) -> u64 {
    let mut agreed_index = vec;
    agreed_index.sort_by(|x, y| { x.cmp(y).reverse() } );
    let len = agreed_index.len();
    return if len == nodes_num && len >= 1 {
        let n = len / 2 + 1;
        assert!(n * 2 > len); // reach majority
        let new_commit_index = agreed_index[n - 1];
        new_commit_index
    } else {
        0
    };
}