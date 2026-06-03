use super::manifest::{SkillManifest, SkillRoutePolicy};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillRouteRequest {
    pub message: String,
    pub explicit_skill_id: Option<String>,
    pub pack_hint: Option<String>,
    pub ui_selected_skill_id: Option<String>,
    #[serde(default)]
    pub session_pinned_skill_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteDecision {
    pub primary_skill: Option<SelectedSkill>,
    pub supporting_skills: Vec<SelectedSkill>,
    pub candidates: Vec<SkillCandidate>,
    pub confidence: f32,
    pub reason: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectedSkill {
    pub id: String,
    pub pack_id: Option<String>,
    pub name: String,
    pub score: u32,
    pub reason: String,
    pub allowed_tools: Vec<String>,
    pub denied_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillCandidate {
    pub id: String,
    pub name: String,
    pub score: u32,
    pub accepted: bool,
    pub reason: String,
}

impl RouteDecision {
    pub fn selected_skills(&self) -> Vec<SelectedSkill> {
        let mut selected = Vec::new();
        if let Some(primary) = &self.primary_skill {
            selected.push(primary.clone());
        }
        selected.extend(self.supporting_skills.clone());
        selected
    }
}

pub fn select_skill_route(skills: &[SkillManifest], request: SkillRouteRequest) -> RouteDecision {
    let normalized_message = normalize(&request.message);
    let command_skill_id = parse_skill_command(&normalized_message);
    let explicit_skill_id = request
        .explicit_skill_id
        .clone()
        .or(command_skill_id)
        .map(|value| normalize_id(&value));
    let mentioned_skill = skills
        .iter()
        .find(|skill| normalized_message.contains(&format!("@{}", normalize_id(&skill.id))))
        .map(|skill| normalize_id(&skill.id));
    let explicit_id = explicit_skill_id.or(mentioned_skill);
    let ui_id = request
        .ui_selected_skill_id
        .as_ref()
        .map(|id| normalize_id(id));
    let pack_hint = request
        .pack_hint
        .as_ref()
        .map(|id| normalize_id(id))
        .or_else(|| parse_pack_hint(skills, &normalized_message));
    let pinned: HashSet<String> = request
        .session_pinned_skill_ids
        .iter()
        .map(|id| normalize_id(id))
        .collect();

    let mut scored = Vec::new();
    let mut warnings = Vec::new();

    for skill in skills {
        let (score, accepted, reason) = score_skill(
            skill,
            &normalized_message,
            explicit_id.as_deref(),
            ui_id.as_deref(),
            pack_hint.as_deref(),
            &pinned,
        );
        if score > 0 || !accepted {
            scored.push((skill, score, accepted, reason));
        }
    }

    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.id.cmp(&b.0.id)));
    let candidates: Vec<SkillCandidate> = scored
        .iter()
        .map(|(skill, score, accepted, reason)| SkillCandidate {
            id: skill.id.clone(),
            name: skill.name.clone(),
            score: *score,
            accepted: *accepted,
            reason: reason.clone(),
        })
        .collect();

    let selectable: Vec<_> = scored
        .iter()
        .filter(|(skill, _, accepted, _)| {
            *accepted && skill.route_policy != SkillRoutePolicy::SuggestOnly
        })
        .collect();

    let mut primary_skill = None;
    let mut reason = "No skill met the routing threshold.".to_string();
    let mut confidence = 0.0;

    if let Some((top_skill, top_score, _, top_reason)) = selectable.first() {
        let explicit = explicit_id.as_deref() == Some(top_skill.id.as_str())
            || ui_id.as_deref() == Some(top_skill.id.as_str());
        let ambiguous = selectable
            .get(1)
            .map(|(_, second_score, _, _)| top_score.saturating_sub(*second_score) <= 5)
            .unwrap_or(false);

        if explicit || (*top_score >= 35 && !ambiguous) {
            primary_skill = Some(selected_from_manifest(top_skill, *top_score, top_reason));
            confidence = (*top_score as f32 / 100.0).min(1.0);
            reason = top_reason.clone();
        } else if ambiguous {
            reason = "Top skill candidates are ambiguous; no primary skill selected.".to_string();
        } else if *top_score >= 20 {
            reason = "Best skill is a suggestion below auto-selection confidence.".to_string();
        }
    }

    let mut supporting_skills = Vec::new();
    for (skill, score, accepted, candidate_reason) in selectable.into_iter().skip(1) {
        if supporting_skills.len() >= 2 {
            break;
        }
        if *accepted && *score >= 35 && skill.prompt.instructions.len() <= 400 {
            supporting_skills.push(selected_from_manifest(skill, *score, candidate_reason));
        }
    }

    if candidates
        .iter()
        .any(|candidate| candidate.reason.contains("fresh"))
    {
        warnings
            .push("One or more skills may require up-to-date official information.".to_string());
    }

    RouteDecision {
        primary_skill,
        supporting_skills,
        candidates,
        confidence,
        reason,
        warnings,
    }
}

fn score_skill(
    skill: &SkillManifest,
    message: &str,
    explicit_id: Option<&str>,
    ui_id: Option<&str>,
    pack_hint: Option<&str>,
    pinned: &HashSet<String>,
) -> (u32, bool, String) {
    if !skill.enabled {
        return (0, false, "disabled skill".to_string());
    }
    if skill.route_policy == SkillRoutePolicy::Disabled {
        return (0, false, "route policy disabled".to_string());
    }

    let skill_id = normalize_id(&skill.id);
    let has_explicit =
        explicit_id == Some(skill_id.as_str()) || message.contains(&format!("@{}", skill_id));
    let has_ui = ui_id == Some(skill_id.as_str());

    if skill.route_policy == SkillRoutePolicy::ManualOnly && !has_ui {
        return (0, false, "manual_only without UI selection".to_string());
    }
    if skill.route_policy == SkillRoutePolicy::ExplicitOnly && !has_explicit {
        return (
            0,
            false,
            "explicit_only without /skill or @ mention".to_string(),
        );
    }
    if contains_any(message, &skill.negative_triggers) {
        return (0, false, "negative trigger matched".to_string());
    }
    if !skill.required_any.is_empty()
        && !contains_any(message, &skill.required_any)
        && !has_explicit
        && !has_ui
    {
        return (0, false, "required_any not matched".to_string());
    }

    let mut score = 0;
    let mut reasons = Vec::new();

    if has_explicit {
        score += 100;
        reasons.push("explicit selection +100");
    }
    if has_ui {
        score += 100;
        reasons.push("UI selection +100");
    }
    if let Some(pack) = pack_hint {
        if skill.pack_id.as_ref().map(|id| normalize_id(id)) == Some(pack.to_string()) {
            score += 40;
            reasons.push("@pack hint +40");
        }
    }
    if pinned.contains(&skill_id) {
        score += 25;
        reasons.push("session pin +25");
    }
    for alias in &skill.aliases {
        if contains_phrase(message, alias) {
            score += 25;
            reasons.push("alias phrase +25");
            break;
        }
    }
    if contains_any(message, &skill.required_any) {
        score += 10;
        reasons.push("required_any +10");
    }
    for trigger in &skill.triggers {
        let normalized_trigger = normalize(trigger);
        if normalized_trigger.contains(' ') && message.contains(&normalized_trigger) {
            score += 8;
            reasons.push("phrase trigger +8");
        } else if message
            .split_whitespace()
            .any(|word| word == normalized_trigger)
        {
            score += 5;
            reasons.push("trigger word +5");
        } else if message
            .split_whitespace()
            .any(|word| strsim::normalized_levenshtein(word, &normalized_trigger) > 0.82)
        {
            score += 2;
            reasons.push("fuzzy trigger +2");
        }
    }
    if skill
        .examples
        .iter()
        .any(|example| phrase_similarity(message, &normalize(example)) >= 0.34)
    {
        score += 15;
        reasons.push("example similarity +15");
    }
    if skill.safety.requires_fresh_info {
        reasons.push("fresh information warning");
    }

    let accepted = has_explicit || has_ui || score >= skill.min_score;
    (score, accepted, reasons.join(", "))
}

fn selected_from_manifest(skill: &SkillManifest, score: u32, reason: &str) -> SelectedSkill {
    SelectedSkill {
        id: skill.id.clone(),
        pack_id: skill.pack_id.clone(),
        name: skill.name.clone(),
        score,
        reason: reason.to_string(),
        allowed_tools: skill.tools.allow.clone(),
        denied_tools: skill.tools.deny.clone(),
    }
}

fn parse_skill_command(message: &str) -> Option<String> {
    let mut parts = message.split_whitespace();
    if parts.next()? != "/skill" {
        return None;
    }
    parts
        .next()
        .map(|value| value.trim_start_matches('@').to_string())
}

fn parse_pack_hint(skills: &[SkillManifest], message: &str) -> Option<String> {
    skills.iter().find_map(|skill| {
        let pack_id = skill.pack_id.as_ref()?;
        let normalized = normalize_id(pack_id);
        if message.contains(&format!("@{}", normalized)) {
            Some(normalized)
        } else {
            None
        }
    })
}

fn contains_any(message: &str, phrases: &[String]) -> bool {
    phrases
        .iter()
        .any(|phrase| contains_phrase(message, phrase))
}

fn contains_phrase(message: &str, phrase: &str) -> bool {
    message.contains(&normalize(phrase))
}

fn phrase_similarity(a: &str, b: &str) -> f64 {
    let a_words: HashSet<&str> = a.split_whitespace().collect();
    let b_words: HashSet<&str> = b.split_whitespace().collect();
    if a_words.is_empty() || b_words.is_empty() {
        return 0.0;
    }
    let overlap = a_words.intersection(&b_words).count() as f64;
    overlap / b_words.len() as f64
}

fn normalize(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .replace(
            |c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '@' || c == '/'),
            " ",
        )
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_id(value: &str) -> String {
    value.trim().trim_start_matches('@').to_ascii_lowercase()
}
