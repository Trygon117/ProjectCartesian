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

// pub const SYSTEM_PROMPT_SCHEMA: &str = r#"
// You are the CartesianOS Kernel. You speak in JSON.
// "#;