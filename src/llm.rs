#![allow(dead_code)]
use serde::{Deserialize, Serialize};

/// High-level struct representing a multi-turn conversation turn.
/// Fits the official Gemini REST JSON structure.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

/// A specific item inside a conversation turn, containing text,
/// function calls, or function responses. We use camelCase naming
/// matching Google's specifications.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_response: Option<FunctionResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}

/// Serde struct representing the Gemini API Request payload.
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Debug, Serialize, Clone)]
struct Tool {
    #[serde(rename = "functionDeclarations")]
    function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Debug, Serialize, Clone)]
struct FunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

/// Serde struct representing the Gemini API Response payload.
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<ApiErrorDetail>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Option<Content>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    code: Option<i32>,
    message: Option<String>,
}

/// Core loop function that coordinates context, queries Gemini, validates tool requests,
/// executes them locally under safety constraints, and feeds results back to the LLM.
pub async fn ask_gemini(
    profile_context: &str,
    style_context: Option<&str>,
    relevant_preferences_context: Option<&str>,
    tools_config: Option<&crate::tools::ToolsConfig>,
    user_prompt: &str,
) -> anyhow::Result<String> {
    // 1. Load the Gemini API key from the environment
    let api_key = std::env::var("GEMINI_API_KEY").map_err(|_| {
        anyhow::anyhow!(
            "Missing GEMINI_API_KEY environment variable.\n\n\
             Please double-check that you have:\n\
             1. Created a `.env` file in the folder where you run the CLI.\n\
             2. Added `GEMINI_API_KEY=your_actual_key_here` to it.\n\
             3. Or set it in your terminal environment directly."
        )
    })?;

    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow::anyhow!(
            "GEMINI_API_KEY environment variable is present, but it is empty.\n\
             Please make sure it contains your valid Gemini API key."
        ));
    }

    // 2. Check if the workspace map database is available
    let has_map = if let Ok(db_path) = crate::workspace_map::get_db_path() {
        db_path.exists()
    } else {
        false
    };

    // 3. Prepare the system prompt instructions that instruct the AI on its personality and rules.
    let mut system_instructions =
        "You are OpenNivara, a helpful, intelligent, and friendly personal CLI assistant. \
        Use profile context to understand the user. \
        Use style context to control tone and formatting. \
        Use triggered preferences only if relevant. \
        You may request local tools only when needed. \
        Never ask for unrelated private files. \
        Do not try to read secrets, API keys, SSH keys, .env files, passwords, or credentials. \
        If a tool result is blocked by policy, explain that safely and continue. \
        Do not claim you read a file unless a tool result confirms it. \
        Focus directly on answering the user's question concisely."
            .to_string();

    if has_map {
        system_instructions.push_str(" \
            A workspace map is available through map tools. Use map_search/map_tree/map_summary before reading files when locating project context.");
    }

    // 4. Assemble the prompt components
    let mut context_block = format!("User Profile Context:\n{}\n", profile_context);

    if let Some(style) = style_context {
        context_block.push_str(&format!("\n{}\n", style));
    }

    if let Some(pref_context) = relevant_preferences_context {
        if !pref_context.trim().is_empty() {
            context_block.push_str(&format!("\n{}\n", pref_context));
        }
    }

    let full_prompt = format!(
        "{}\n\n\
         {}\n\n\
         Question: {}\n\n\
         OpenNivara:",
        system_instructions, context_block, user_prompt
    );

    // 5. Set up multi-turn conversation history
    let mut history = vec![Content {
        role: "user".to_string(),
        parts: vec![Part {
            text: Some(full_prompt),
            function_call: None,
            function_response: None,
        }],
    }];

    // 6. Check tool settings
    let tools_enabled = tools_config.map(|c| c.general.enabled).unwrap_or(false);
    let tools_declaration = if tools_enabled {
        let registry = crate::tools::ToolRegistry::new(has_map);
        let function_declarations = registry
            .declared_definitions(
                tools_config.expect("tools_config is present when tools_enabled is true"),
                None,
            )
            .into_iter()
            .map(|definition| FunctionDeclaration {
                name: definition.name,
                description: definition.description,
                parameters: definition.parameters,
            })
            .collect();
        Some(vec![Tool {
            function_declarations,
        }])
    } else {
        None
    };

    let max_rounds = tools_config.map(|c| c.general.max_tool_rounds).unwrap_or(3);
    let show_activity = tools_config
        .map(|c| c.general.show_tool_activity)
        .unwrap_or(true);

    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let mut current_round = 0;

    // 7. Tool Execution Loop
    loop {
        let request_payload = GeminiRequest {
            contents: history.clone(),
            tools: tools_declaration.clone(),
        };

        let response = client.post(&url)
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!(
                "Network request failed: {}\n\
                 Please ensure you are connected to the internet and that the API endpoint is accessible.", 
                e
            ))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

        let api_response: GeminiResponse = serde_json::from_str(&response_text).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse Gemini API JSON response: {}\n\
                 Raw Response: {}",
                e,
                response_text
            )
        })?;

        if let Some(err) = api_response.error {
            return Err(anyhow::anyhow!(
                "Gemini API Error (Code: {}): {}",
                err.code.unwrap_or(0),
                err.message
                    .unwrap_or_else(|| "No error message".to_string())
            ));
        }

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Gemini API returned an unsuccessful HTTP status: {}\n\
                 Raw Response: {}",
                status,
                response_text
            ));
        }

        let candidate = match api_response.candidates.as_ref().and_then(|c| c.first()) {
            Some(c) => c,
            None => {
                return Err(anyhow::anyhow!(
                    "Received a successful response from Gemini, but no candidate was returned."
                ))
            }
        };

        let content = match &candidate.content {
            Some(c) => c,
            None => {
                return Err(anyhow::anyhow!(
                    "Received response from Gemini, but candidate contains no content."
                ))
            }
        };

        // Save model output turn in history
        history.push(content.clone());

        // Check if Gemini requested a function call
        let mut requested_call = None;
        for part in &content.parts {
            if let Some(call) = &part.function_call {
                requested_call = Some(call.clone());
                break;
            }
        }

        if let Some(call) = requested_call {
            // Check if we exceeded max tool rounds
            if current_round >= max_rounds {
                if show_activity {
                    println!(
                        "\n\x1b[1;33m[OpenNivara Limit]\x1b[0m Max tool rounds limit ({}) reached. Halting tool loop.", 
                        max_rounds
                    );
                }

                history.push(Content {
                    role: "function".to_string(),
                    parts: vec![Part {
                        text: None,
                        function_call: None,
                        function_response: Some(FunctionResponse {
                            name: call.name.clone(),
                            response: serde_json::json!({
                                "error": format!("Tool execution halted: maximum tool rounds (rounds limit: {}) was reached.", max_rounds)
                            }),
                        }),
                    }],
                });

                current_round += 1;
                continue;
            }

            // Print activity status
            if show_activity {
                let args_str = serde_json::to_string(&call.args).unwrap_or_default();
                println!(
                    "\x1b[1;34mOpenNivara tool:\x1b[0m {} {}",
                    call.name, args_str
                );
            }

            // 8. Execute local or workspace map tools through the central registry.
            let tool_result = if let Some(config) = tools_config {
                crate::tools::ToolRegistry::new(has_map).execute(&call.name, &call.args, config)
            } else {
                serde_json::json!({
                    "error": "Local tools execution is disabled. No configuration is loaded."
                })
            };

            // Feed results back to conversational history
            history.push(Content {
                role: "function".to_string(),
                parts: vec![Part {
                    text: None,
                    function_call: None,
                    function_response: Some(FunctionResponse {
                        name: call.name.clone(),
                        response: tool_result,
                    }),
                }],
            });

            current_round += 1;
            continue; // Query Gemini again with the tool response
        }

        // If no function call was requested, return the final text response
        let mut final_text = None;
        for part in &content.parts {
            if let Some(text) = &part.text {
                final_text = Some(text.clone());
                break;
            }
        }

        match final_text {
            Some(text) => return Ok(text.trim().to_string()),
            None => {
                return Err(anyhow::anyhow!(
                    "Gemini returned a final response candidate, but no text part was found."
                ))
            }
        }
    }
}
