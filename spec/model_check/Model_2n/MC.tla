---- MODULE MC ----
EXTENDS raft, TLC

\* MV CONSTANT declarations@modelParameterConstants
CONSTANTS
A_v1, A_v2
----

\* MV CONSTANT declarations@modelParameterConstants
CONSTANTS
A_n1, A_n2
----

\* MV CONSTANT definitions VALUE
const_1719323875171274000 == 
{A_v1, A_v2}
----

\* MV CONSTANT definitions NODE_ID
const_1719323875171275000 == 
{A_n1, A_n2}
----

\* SYMMETRY definition
symm_1719323875171276000 == 
Permutations(const_1719323875171275000)
----

\* CONSTANT definitions @modelParameterConstants:0RESTART
const_1719323875171277000 == 
2
----

\* CONSTANT definitions @modelParameterConstants:1MAX_TERM
const_1719323875171278000 == 
3
----

\* CONSTANT definitions @modelParameterConstants:2DB_INIT_STATE_PATH
const_1719323875171279000 == 
""
----

\* CONSTANT definitions @modelParameterConstants:4ENABLE_ACTION
const_1719323875171280000 == 
TRUE
----

\* CONSTANT definitions @modelParameterConstants:5APPEND_ENTRIES_MAX
const_1719323875171281000 == 
2
----

\* CONSTANT definitions @modelParameterConstants:6CHECK_SAFETY
const_1719323875171282000 == 
FALSE
----

\* CONSTANT definitions @modelParameterConstants:8DB_SAVE_STATE_PATH
const_1719323875171283000 == 
""
----

\* CONSTANT definitions @modelParameterConstants:9RECONFIG
const_1719323875171284000 == 
2
----

\* CONSTANT definitions @modelParameterConstants:11REPLICATE
const_1719323875171285000 == 
2
----

\* CONSTANT definitions @modelParameterConstants:12DB_ACTION_PATH
const_1719323875171286000 == 
"/root/workspace/scupt-raft/spec/model_check/Model_2n/action.db"
----

\* CONSTANT definitions @modelParameterConstants:13VOTE
const_1719323875171287000 == 
1000
----

\* CONSTANT definitions @modelParameterConstants:14CLIENT_REQUEST
const_1719323875171288000 == 
2
----

\* VIEW definition @modelParameterView
view_1719323875171289000 == 
vars_view
----

=============================================================================
\* Modification History
\* Created Tue Jun 25 21:57:55 CST 2024 by ybbh
