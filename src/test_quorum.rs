pub mod tests {
    use std::collections::HashMap;

    use arbitrary::{Arbitrary, Unstructured};
    use scupt_util::mt_set::MTSet;
    use scupt_util::node_id::NID;
    use serde::{Deserialize, Serialize};

    use crate::conf_node::ConfNode;
    use crate::conf_version::ConfVersion;
    use crate::quorum::{
        quorum_agree_match_index,
        quorum_agree_vote,
        quorum_check_conf_term_version,
        quorum_check_term_commit_index,
    };
    use crate::term_index::TermIndex;
    use crate::test_data::tests::test_data;

    const MAX_NODE: u64 = 11;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TestParamAgreeMatchIndex {
        leader_nid: NID,
        leader_last_index: u64,
        follower_match_index: HashMap<NID, u64>,
        conf_committed: ConfNode,
        conf_new: ConfNode,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TestParamAgreeVote {
        leader_nid: NID,
        current_term: u64,
        follower_vote_granted: HashMap<NID, u64>,
        conf_committed: ConfNode,
        conf_new: ConfNode,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TestParamCheckConfTermVersion {
        leader_nid: NID,
        node: Vec<NID>,
        leader_term_version: ConfVersion,
        follower_conf: HashMap<NID, ConfVersion>,
    }


    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TestParamCheckTermCommitIndex {
        leader_nid: NID,
        nid_set: Vec<NID>,
        leader_term: u64,
        leader_conf_commit_index: u64,
        follower_term_and_committed_index: HashMap<NID, TermIndex>,
    }

    fn arbitrary_node_ids(u: &mut Unstructured) -> arbitrary::Result<(NID, Vec<NID>)> {
        let num_nodes = u32::arbitrary(u)? as u64 % MAX_NODE + 1;
        let leader_nid = 1;
        let mut nodes = vec![];
        for i in 1..=num_nodes {
            nodes.push(i as NID);
        }
        return Ok((leader_nid, nodes));
    }

    fn arbitrary_conf_node(u: &mut Unstructured, nodes: &Vec<NID>) -> arbitrary::Result<ConfNode> {
        let n = u32::arbitrary(u)? as usize % (nodes.len()) + 1;
        let mut nid_vote = vec![];
        for i in 0..n {
            nid_vote.push(nodes[i].clone());
        }
        let cn = ConfNode {
            conf_version: ConfVersion::arbitrary(u)?,
            nid_vote: MTSet::from_vec(nid_vote),
            nid_log: MTSet::from_vec(nodes.clone()),
        };
        Ok(cn)
    }

    impl<'a> Arbitrary<'a> for TestParamAgreeMatchIndex {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let (leader_nid, nid_set) = arbitrary_node_ids(u)?;
            let leader_last_index = u32::arbitrary(u)? as u64 % MAX_NODE;
            let mut follower_match_index = HashMap::default();
            for i in nid_set.iter() {
                if bool::arbitrary(u)? {
                    if *i != leader_nid {
                        let index = u32::arbitrary(u)? as u64 % MAX_NODE;
                        follower_match_index.insert(*i, index);
                    } else {
                        follower_match_index.insert(*i, 0);
                    }
                }
            }
            let nid1 = arbitrary_conf_node(u, &nid_set)?;
            let nid2 = arbitrary_conf_node(u, &nid_set)?;
            let param = TestParamAgreeMatchIndex {
                leader_nid,
                leader_last_index,
                follower_match_index,
                conf_committed: nid1,
                conf_new: nid2,
            };
            Ok(param)
        }
    }

    impl<'a> Arbitrary<'a> for TestParamAgreeVote {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let (leader_nid, nid_set) = arbitrary_node_ids(u)?;
            let mut follower_vote_granted = HashMap::default();
            let current_term = u32::arbitrary(u)? as u64 % MAX_NODE;
            for i in nid_set.iter() {
                if bool::arbitrary(u)? {
                    if *i != leader_nid {
                        let term = u32::arbitrary(u)? as u64 % MAX_NODE;
                        follower_vote_granted.insert(*i, term);
                    } else {
                        follower_vote_granted.insert(*i, current_term);
                    }
                }
            }
            let nid1 = arbitrary_conf_node(u, &nid_set)?;
            let nid2 = arbitrary_conf_node(u, &nid_set)?;
            let param = TestParamAgreeVote {
                leader_nid,
                current_term,
                follower_vote_granted,
                conf_committed: nid1,
                conf_new: nid2,
            };
            Ok(param)
        }
    }


    impl<'a> Arbitrary<'a> for TestParamCheckConfTermVersion {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<TestParamCheckConfTermVersion> {
            let (leader_nid, node) = arbitrary_node_ids(u)?;
            let mut follower_conf = HashMap::new();
            let leader_term_version = ConfVersion::arbitrary(u)?;
            for i in node.iter() {
                if bool::arbitrary(u)? {
                    if *i == leader_nid {
                        follower_conf.insert(*i, leader_term_version.clone());
                    } else {
                        follower_conf.insert(*i, ConfVersion::arbitrary(u)?);
                    }
                }
            }
            let param = TestParamCheckConfTermVersion {
                leader_nid,
                leader_term_version,
                node,
                follower_conf,
            };
            Ok(param)
        }
    }

    impl<'a> Arbitrary<'a> for TestParamCheckTermCommitIndex {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let (leader_nid, nid_set) = arbitrary_node_ids(u)?;
            let mut follower_term_and_committed_index = HashMap::new();
            for i in nid_set.iter() {
                if bool::arbitrary(u)? {
                    if *i == leader_nid {
                        let term = u32::arbitrary(u)? as u64 % MAX_NODE;
                        let index = u32::arbitrary(u)? as u64 % MAX_NODE;
                        follower_term_and_committed_index.insert(*i, TermIndex { term, index });
                    } else {
                        follower_term_and_committed_index.insert(*i, TermIndex::default());
                    }
                }
            }
            let param = TestParamCheckTermCommitIndex {
                leader_nid,
                nid_set,
                leader_term: 0,
                leader_conf_commit_index: 0,
                follower_term_and_committed_index,
            };
            Ok(param)
        }
    }

    impl<'a> Arbitrary<'a> for ConfVersion {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<ConfVersion> {
            let max = MAX_NODE / 2 + 1;
            let term = (u16::arbitrary(u)? as u64) % max;
            let version = (u16::arbitrary(u)? as u64) % max;
            let commit_index = (u16::arbitrary(u)? as u64) % max;
            Ok(ConfVersion {
                term,
                version,
                index: commit_index,
            })
        }
    }


    pub fn _test_quorum_agree_match_index(param: &TestParamAgreeMatchIndex) -> u64 {
        let index = quorum_agree_match_index(
            param.leader_nid,
            param.leader_last_index,
            &param.follower_match_index,
            &param.conf_committed,
            &param.conf_new,
        );
        index
    }

    pub fn _test_quorum_agree_vote(param: &TestParamAgreeVote) -> bool {
        let ok = quorum_agree_vote(
            param.leader_nid,
            param.current_term,
            &param.follower_vote_granted,
            &param.conf_committed,
            &param.conf_new,
        );
        ok
    }

    pub fn _test_quorum_check_conf_term_version(param: &TestParamCheckConfTermVersion) -> bool {
        let ok = quorum_check_conf_term_version(
            param.leader_nid,
            &param.node,
            &param.leader_term_version,
            &param.follower_conf,
        );
        ok
    }


    pub fn _test_quorum_check_term_commit_index(param: &TestParamCheckTermCommitIndex) -> bool {
        let ok = quorum_check_term_commit_index(
            param.leader_nid,
            &param.nid_set,
            param.leader_term,
            param.leader_conf_commit_index,
            &param.follower_term_and_committed_index,
        );
        ok
    }

    #[test]
    fn test_quorum_agree_vote() {
        let test_data = test_data::<TestParamAgreeVote, bool>("quorum_agree_vote".to_string());
        for (input, result, _p) in test_data.iter() {
            let ok = _test_quorum_agree_vote(&input);
            assert_eq!(ok, *result);
        }
    }

    #[test]
    fn test_quorum_agree_match_index() {
        let test_data = test_data::<TestParamAgreeMatchIndex, u64>("quorum_agree_match_index".to_string());
        for (input, result, _p) in test_data.iter() {
            let ok = _test_quorum_agree_match_index(input);
            assert_eq!(ok, *result);
        }
    }

    #[test]
    fn test_quorum_check_conf_term_version() {
        let test_data = test_data::<TestParamCheckConfTermVersion, bool>("quorum_check_conf_term_version".to_string());
        for (input, result, _p) in test_data.iter() {
            let ok = _test_quorum_check_conf_term_version(input);
            assert_eq!(ok, *result);
        }
    }

    #[test]
    fn test_quorum_check_term_commit_index() {
        let test_data = test_data::<TestParamCheckTermCommitIndex, bool>("quorum_check_term_commit_index".to_string());
        for (input, result, _p) in test_data.iter() {
            let ok = _test_quorum_check_term_commit_index(input);
            assert_eq!(ok, *result);
        }
    }
}
