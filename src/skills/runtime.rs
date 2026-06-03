use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillRuntimePolicy {
    pub executable_runtime_available: bool,
    pub wasm_allowed: bool,
    pub python_allowed: bool,
    pub javascript_allowed: bool,
    pub remote_execution_allowed: bool,
}

pub fn runtime_policy_v1() -> SkillRuntimePolicy {
    SkillRuntimePolicy {
        executable_runtime_available: false,
        wasm_allowed: false,
        python_allowed: false,
        javascript_allowed: false,
        remote_execution_allowed: false,
    }
}
