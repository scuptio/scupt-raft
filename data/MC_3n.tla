---- MODULE MC ----
EXTENDS raft, TLC

\* MV CONSTANT declarations@modelParameterConstants
CONSTANTS
A_v1, A_v2
----

\* MV CONSTANT declarations@modelParameterConstants
CONSTANTS
A_n1, A_n2, A_n3
----

\* MV CONSTANT definitions VALUE
const_171655968391738000 == 
{A_v1, A_v2}
----

\* MV CONSTANT definitions NODE_ID
const_171655968391739000 == 
{A_n1, A_n2, A_n3}
----

\* SYMMETRY definition
symm_171655968391740000 == 
Permutations(const_171655968391738000)
----

\* CONSTANT definitions @modelParameterConstants:0RESTART
const_171655968391741000 == 
2
----

\* CONSTANT definitions @modelParameterConstants:1MAX_TERM
const_171655968391742000 == 
3
----

\* CONSTANT definitions @modelParameterConstants:2DB_INIT_STATE_PATH
const_171655968391743000 == 
""
----

\* CONSTANT definitions @modelParameterConstants:4ENABLE_ACTION
const_171655968391744000 == 
TRUE
----

\* CONSTANT definitions @modelParameterConstants:5APPEND_ENTRIES_MAX
const_171655968391745000 == 
2
----

\* CONSTANT definitions @modelParameterConstants:6CHECK_SAFETY
const_171655968391746000 == 
FALSE
----

\* CONSTANT definitions @modelParameterConstants:8DB_SAVE_STATE_PATH
const_171655968391747000 == 
"/home/ybbh/workspace/tlaplus-specification/data/raft_save_state_3n.db"
----

\* CONSTANT definitions @modelParameterConstants:9RECONFIG
const_171655968391748000 == 
0
----

\* CONSTANT definitions @modelParameterConstants:11REPLICATE
const_171655968391749000 == 
2
----

\* CONSTANT definitions @modelParameterConstants:12DB_ACTION_PATH
const_171655968391750000 == 
"/home/ybbh/workspace/tlaplus-specification/data/raft_action_3n.db"
----

\* CONSTANT definitions @modelParameterConstants:13VOTE
const_171655968391751000 == 
2
----

\* CONSTANT definitions @modelParameterConstants:14CLIENT_REQUEST
const_171655968391752000 == 
2
----

\* VIEW definition @modelParameterView
view_171655968391753000 == 
vars_view
----

=============================================================================
\* Modification History
\* Created Fri May 24 22:08:03 CST 2024 by ybbh
