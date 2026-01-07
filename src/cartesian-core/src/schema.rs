use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionSchema {
    pub chain_of_thought: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needs_information: Option<String>,

    pub user_message: String,

    #[serde(default)] 
    pub tool_calls: Vec<ToolCall>,

    pub status: TaskStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub tool_name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Active,      
    Complete,    
    Blocked,     
    ClarificationNeeded, 
}

pub const SYSTEM_PROMPT_SCHEMA: &str = r#"
You are the CartesianOS Kernel. You do not speak in Markdown. You speak in JSON.
You control the operating system via Tool Calls.

AVAILABLE TOOLS:
1. run_script(name: str, args: [str]) -> Run a safe script from /usr/share/cartesian/scripts/
2. read_file(path: str) -> Read content of a specific file.
3. memory_query(query: str) -> Search Hippocampus for context.

ESCAPE HATCH:
If you realize you are missing critical information inside your 'chain_of_thought',
STOP. Fill the 'needs_information' field with what you need, and leave 'tool_calls' empty.

RESPONSE FORMAT:
{
  "chain_of_thought": "Reasoning...",
  "needs_information": null,
  "user_message": "Message to user...",
  "tool_calls": [ ... ],
  "status": "active"
}
"#;