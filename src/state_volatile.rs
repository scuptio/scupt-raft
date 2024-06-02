use crate::raft_role::RaftRole;

pub struct StateVolatile {
    pub role: Option<RaftRole>,
}

impl Default for StateVolatile {
    fn default() -> Self {
        Self {
            role: None,
        }
    }
}