use crate::context::{ContextEntry, ContextsFile};
use crate::preferences::{PreferenceSection, PreferencesFile};

/// Helper to calculate the match score of a single trigger keyword against a lowercase user message.
/// - Exact word boundary match: +3 points.
/// - Partial substring match: +1 point.
/// - No match: 0 points.
fn calculate_single_score(lowercase_msg: &str, trigger: &str) -> u32 {
    let lowercase_trig = trigger.to_lowercase();
    if lowercase_trig.is_empty() {
        return 0;
    }

    let mut score = 0;
    let mut search_start = 0;

    while let Some(idx) = lowercase_msg[search_start..].find(&lowercase_trig) {
        let abs_idx = search_start + idx;

        let left_boundary = if abs_idx == 0 {
            true
        } else {
            let prev_char = lowercase_msg.chars().nth(abs_idx - 1).unwrap_or(' ');
            !prev_char.is_alphanumeric() && prev_char != '_'
        };

        let right_boundary = {
            let right_idx = abs_idx + lowercase_trig.len();
            if right_idx >= lowercase_msg.len() {
                true
            } else {
                let next_char = lowercase_msg.chars().nth(right_idx).unwrap_or(' ');
                !next_char.is_alphanumeric() && next_char != '_'
            }
        };

        let current_score = if left_boundary && right_boundary {
            3
        } else {
            1
        };
        if current_score > score {
            score = current_score;
        }

        search_start = abs_idx + 1;
        if search_start >= lowercase_msg.len() {
            break;
        }
    }

    score
}

/// Evaluates triggers against a user message.
/// Returns a total match score, or 0 if negative triggers match or required_any terms are absent.
pub fn evaluate_triggers(
    user_message: &str,
    triggers: &[String],
    required_any: &[String],
    negative_triggers: &[String],
) -> u32 {
    let lowercase_msg = user_message.to_lowercase();

    // 1. Negative triggers: discard immediately if any matches
    for neg in negative_triggers {
        if lowercase_msg.contains(&neg.to_lowercase()) {
            return 0;
        }
    }

    // 2. Required any: discard immediately if none is matched
    if !required_any.is_empty() {
        let mut matched_req = false;
        for req in required_any {
            if lowercase_msg.contains(&req.to_lowercase()) {
                matched_req = true;
                break;
            }
        }
        if !matched_req {
            return 0;
        }
    }

    // 3. Compute total score
    let mut total_score = 0;
    for trigger in triggers {
        total_score += calculate_single_score(&lowercase_msg, trigger);
    }

    total_score
}

/// Selects relevant preference sections based on the strict selection policy.
pub fn select_relevant_preference_sections(
    prefs: &PreferencesFile,
    user_message: &str,
) -> Vec<PreferenceSection> {
    let mut always_list = Vec::new();
    let mut triggered_list: Vec<(PreferenceSection, u32)> = Vec::new();

    for section in &prefs.sections {
        if !section.enabled {
            continue;
        }

        match section.send_policy.as_str() {
            "always" => {
                always_list.push(section.clone());
            }
            "triggered_strict" => {
                let score = evaluate_triggers(
                    user_message,
                    &section.triggers,
                    &section.required_any,
                    &section.negative_triggers,
                );
                if score >= section.min_score {
                    triggered_list.push((section.clone(), score));
                }
            }
            _ => {} // never, disabled
        }
    }

    // Sort triggered sections by highest score first
    triggered_list.sort_by_key(|b| std::cmp::Reverse(b.1));

    // Combine always lists and up to 3 triggered sections
    let mut selected = always_list;
    for (sec, _) in triggered_list.into_iter().take(3) {
        selected.push(sec);
    }

    selected
}

/// Selects relevant contexts based on the strict selection policy and session pinned lists.
pub fn select_relevant_contexts(
    contexts_file: &ContextsFile,
    user_message: &str,
    pinned_ids: &[String],
) -> Vec<ContextEntry> {
    let mut always_list = Vec::new();
    let mut pinned_list = Vec::new();
    let mut triggered_list: Vec<(ContextEntry, u32)> = Vec::new();

    for entry in &contexts_file.contexts {
        if !entry.enabled {
            continue;
        }

        // Check if explicitly pinned
        if pinned_ids.contains(&entry.id)
            && entry.send_policy != "disabled"
            && entry.send_policy != "never"
        {
            pinned_list.push(entry.clone());
            continue;
        }

        match entry.send_policy.as_str() {
            "always" => {
                always_list.push(entry.clone());
            }
            "triggered_strict" => {
                let score = evaluate_triggers(
                    user_message,
                    &entry.triggers,
                    &entry.required_any,
                    &entry.negative_triggers,
                );
                if score >= entry.min_score {
                    triggered_list.push((entry.clone(), score));
                }
            }
            _ => {} // manual (if not pinned), never, disabled, session_pinned (if not pinned)
        }
    }

    // Sort triggered contexts by score desc
    triggered_list.sort_by_key(|b| std::cmp::Reverse(b.1));

    // Combine: always + pinned + up to 3 triggered contexts
    let mut selected = always_list;
    selected.extend(pinned_list);
    for (entry, _) in triggered_list.into_iter().take(3) {
        selected.push(entry);
    }

    selected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ContextEntry;
    use crate::preferences::{PreferenceSection, PreferencesFile};
    use pretty_assertions::assert_eq;
    use proptest::prelude::*;

    #[test]
    fn test_calculate_single_score() {
        // Exact boundary (left and right) -> 3 points
        assert_eq!(calculate_single_score("rust is great", "rust"), 3);
        assert_eq!(calculate_single_score("i love rust", "rust"), 3);
        assert_eq!(calculate_single_score("rust-lang is cool", "rust"), 3);

        // Substring boundary -> 1 point
        assert_eq!(calculate_single_score("trusting rust", "rust"), 3);
        assert_eq!(calculate_single_score("crusty food", "rust"), 1);
        assert_eq!(calculate_single_score("trust is important", "rust"), 1);

        // No match -> 0 points
        assert_eq!(calculate_single_score("go is nice", "rust"), 0);
    }

    #[test]
    fn test_evaluate_triggers() {
        let triggers = vec!["rust".to_string(), "javascript".to_string()];
        let required_any = vec!["code".to_string(), "programming".to_string()];
        let negative_triggers = vec!["python".to_string()];

        // 1. Negative trigger match -> 0
        assert_eq!(
            evaluate_triggers(
                "i write rust code in python",
                &triggers,
                &required_any,
                &negative_triggers
            ),
            0
        );

        // 2. Missing required_any -> 0
        assert_eq!(
            evaluate_triggers("i write rust", &triggers, &required_any, &negative_triggers),
            0
        );

        // 3. Match with exact word trigger -> 3
        assert_eq!(
            evaluate_triggers(
                "i write rust code",
                &triggers,
                &required_any,
                &negative_triggers
            ),
            3
        );

        // 4. Match with substring trigger
        assert_eq!(
            evaluate_triggers(
                "my codebase uses javascript-related tools",
                &triggers,
                &required_any,
                &negative_triggers
            ),
            3
        );
        assert_eq!(
            evaluate_triggers(
                "i love programming in trust issues",
                &triggers,
                &required_any,
                &negative_triggers
            ),
            1
        );
    }

    #[test]
    fn test_select_relevant_preference_sections() {
        let section_always = PreferenceSection {
            id: "always_sec".to_string(),
            enabled: true,
            send_policy: "always".to_string(),
            description: None,
            triggers: vec![],
            required_any: vec![],
            negative_triggers: vec![],
            min_score: 0,
            likes: vec![],
            dislikes: vec![],
            notes: vec![],
        };
        let section_triggered = PreferenceSection {
            id: "triggered_sec".to_string(),
            enabled: true,
            send_policy: "triggered_strict".to_string(),
            description: None,
            triggers: vec!["rust".to_string()],
            required_any: vec![],
            negative_triggers: vec![],
            min_score: 2,
            likes: vec![],
            dislikes: vec![],
            notes: vec![],
        };
        let section_disabled = PreferenceSection {
            id: "disabled_sec".to_string(),
            enabled: false,
            send_policy: "always".to_string(),
            description: None,
            triggers: vec![],
            required_any: vec![],
            negative_triggers: vec![],
            min_score: 0,
            likes: vec![],
            dislikes: vec![],
            notes: vec![],
        };

        let prefs = PreferencesFile {
            schema_version: 1,
            sections: vec![
                section_always.clone(),
                section_triggered.clone(),
                section_disabled.clone(),
            ],
        };

        // If message triggers, both always and triggered should be selected
        let selected = select_relevant_preference_sections(&prefs, "rust is fun");
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].id, "always_sec");
        assert_eq!(selected[1].id, "triggered_sec");

        // If message does not trigger, only always should be selected
        let selected_no_trigger = select_relevant_preference_sections(&prefs, "python is fun");
        assert_eq!(selected_no_trigger.len(), 1);
        assert_eq!(selected_no_trigger[0].id, "always_sec");
    }

    #[test]
    fn test_select_relevant_contexts() {
        let contexts_file = ContextsFile {
            schema_version: 1,
            contexts: vec![
                ContextEntry {
                    id: "always_ctx".to_string(),
                    enabled: true,
                    kind: "goal".to_string(),
                    send_policy: "always".to_string(),
                    title: "Always Context".to_string(),
                    summary: "Always Summary".to_string(),
                    triggers: vec![],
                    required_any: vec![],
                    negative_triggers: vec![],
                    min_score: 0,
                    facts: vec![],
                    rules: vec![],
                },
                ContextEntry {
                    id: "pinned_ctx".to_string(),
                    enabled: true,
                    kind: "goal".to_string(),
                    send_policy: "session_pinned".to_string(),
                    title: "Pinned Context".to_string(),
                    summary: "Pinned Summary".to_string(),
                    triggers: vec![],
                    required_any: vec![],
                    negative_triggers: vec![],
                    min_score: 0,
                    facts: vec![],
                    rules: vec![],
                },
                ContextEntry {
                    id: "triggered_ctx".to_string(),
                    enabled: true,
                    kind: "goal".to_string(),
                    send_policy: "triggered_strict".to_string(),
                    title: "Triggered Context".to_string(),
                    summary: "Triggered Summary".to_string(),
                    triggers: vec!["rust".to_string()],
                    required_any: vec![],
                    negative_triggers: vec![],
                    min_score: 3,
                    facts: vec![],
                    rules: vec![],
                },
            ],
        };

        // Pinned context passed in pinned_ids list, triggered context matches trigger
        let selected =
            select_relevant_contexts(&contexts_file, "i love rust", &["pinned_ctx".to_string()]);
        assert_eq!(selected.len(), 3);
        assert_eq!(selected[0].id, "always_ctx");
        assert_eq!(selected[1].id, "pinned_ctx");
        assert_eq!(selected[2].id, "triggered_ctx");

        // Pinned context not in list, no trigger match
        let selected_none = select_relevant_contexts(&contexts_file, "hello", &[]);
        assert_eq!(selected_none.len(), 1);
        assert_eq!(selected_none[0].id, "always_ctx");
    }

    proptest! {
        #[test]
        fn empty_trigger_lists_never_score(message in ".*") {
            prop_assert_eq!(evaluate_triggers(&message, &[], &[], &[]), 0);
        }

        #[test]
        fn negative_trigger_dominates_positive_matches(
            prefix in "[a-z ]{0,24}",
            suffix in "[a-z ]{0,24}",
            trigger in "[a-z]{1,12}",
        ) {
            let message = format!("{prefix} {trigger} {suffix}");
            prop_assert_eq!(
                evaluate_triggers(
                    &message,
                    std::slice::from_ref(&trigger),
                    &[],
                    std::slice::from_ref(&trigger),
                ),
                0
            );
        }
    }
}
