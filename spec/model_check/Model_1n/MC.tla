---- MODULE MC ----
EXTENDS raft, TLC

\* MV CONSTANT declarations@modelParameterConstants
CONSTANTS
A_v1
----

\* MV CONSTANT declarations@modelParameterConstants
CONSTANTS
A_n1
----

\* MV CONSTANT definitions VALUE
const_1718865728589375000 == 
{A_v1}
----

\* MV CONSTANT definitions NODE_ID
const_1718865728589376000 == 
{A_n1}
----

\* CONSTANT definitions @modelParameterConstants:0RESTART
const_1718865728589377000 == 
3
----

\* CONSTANT definitions @modelParameterConstants:1MAX_TERM
const_1718865728589378000 == 
3
----

\* CONSTANT definitions @modelParameterConstants:2DB_INIT_STATE_PATH
const_1718865728589379000 == 
""
----

\* CONSTANT definitions @modelParameterConstants:4ENABLE_ACTION
const_1718865728589380000 == 
TRUE
----

\* CONSTANT definitions @modelParameterConstants:5APPEND_ENTRIES_MAX
const_1718865728589381000 == 
3
----

\* CONSTANT definitions @modelParameterConstants:6CHECK_SAFETY
const_1718865728589382000 == 
FALSE
----

\* CONSTANT definitions @modelParameterConstants:8DB_SAVE_STATE_PATH
const_1718865728589383000 == 
""
----

\* CONSTANT definitions @modelParameterConstants:9RECONFIG
const_1718865728589384000 == 
3
----

\* CONSTANT definitions @modelParameterConstants:11REPLICATE
const_1718865728589385000 == 
3
----

\* CONSTANT definitions @modelParameterConstants:12DB_ACTION_PATH
const_1718865728589386000 == 
"/root/workspace/scupt-raft/spec/model_check/Model_1n/action.db"
----

\* CONSTANT definitions @modelParameterConstants:13VOTE
const_1718865728589387000 == 
2
----

\* CONSTANT definitions @modelParameterConstants:14CLIENT_REQUEST
const_1718865728589388000 == 
2
----

\* VIEW definition @modelParameterView
view_1718865728589389000 == 
vars_view
----

=============================================================================
\* Modification History
\* Created Thu Jun 20 14:42:08 CST 2024 by ybbh
