---- MODULE MC ----
EXTENDS raft, TLC

\* MV CONSTANT declarations@modelParameterConstants
CONSTANTS
A_v1, A_v2
----

\* MV CONSTANT declarations@modelParameterConstants
CONSTANTS
A_n1
----

\* MV CONSTANT definitions VALUE
const_1717334316002212000 == 
{A_v1, A_v2}
----

\* MV CONSTANT definitions NODE_ID
const_1717334316002213000 == 
{A_n1}
----

\* SYMMETRY definition
symm_1717334316002214000 == 
Permutations(const_1717334316002212000)
----

\* CONSTANT definitions @modelParameterConstants:0RESTART
const_1717334316002215000 == 
3
----

\* CONSTANT definitions @modelParameterConstants:1MAX_TERM
const_1717334316002216000 == 
3
----

\* CONSTANT definitions @modelParameterConstants:2DB_INIT_STATE_PATH
const_1717334316002217000 == 
""
----

\* CONSTANT definitions @modelParameterConstants:4ENABLE_ACTION
const_1717334316002218000 == 
TRUE
----

\* CONSTANT definitions @modelParameterConstants:5APPEND_ENTRIES_MAX
const_1717334316002219000 == 
2
----

\* CONSTANT definitions @modelParameterConstants:6CHECK_SAFETY
const_1717334316002220000 == 
FALSE
----

\* CONSTANT definitions @modelParameterConstants:8DB_SAVE_STATE_PATH
const_1717334316002221000 == 
"/home/ybbh/workspace/tlaplus-specification/data/raft_state_1n.db"
----

\* CONSTANT definitions @modelParameterConstants:9RECONFIG
const_1717334316002222000 == 
3
----

\* CONSTANT definitions @modelParameterConstants:11REPLICATE
const_1717334316002223000 == 
3
----

\* CONSTANT definitions @modelParameterConstants:12DB_ACTION_PATH
const_1717334316002224000 == 
"/home/ybbh/workspace/tlaplus-specification/data/raft_action_1n.db"
----

\* CONSTANT definitions @modelParameterConstants:13VOTE
const_1717334316002225000 == 
3
----

\* CONSTANT definitions @modelParameterConstants:14CLIENT_REQUEST
const_1717334316002226000 == 
3
----

\* VIEW definition @modelParameterView
view_1717334316002227000 == 
vars_view
----

=============================================================================
\* Modification History
\* Created Sun Jun 02 21:18:36 CST 2024 by ybbh
