use super::manifest::{
    validate_skill_manifest, SkillManifest, SkillRoutePolicy, SkillSafety, SkillToolPolicy,
};
use super::selector::{select_skill_route, SkillRouteRequest};
use crate::marketplace::packs::{validate_pack_manifest, PackManifest};
use crate::tools::ToolRegistry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct RoutingFixtureFile {
    #[serde(default)]
    pub cases: Vec<RoutingCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct RoutingCase {
    pub id: String,
    pub message: String,
    #[serde(default)]
    pub enabled_skill_ids: Vec<String>,
    #[serde(default)]
    pub explicit_skill_id: Option<String>,
    #[serde(default)]
    pub ui_selected_skill_id: Option<String>,
    #[serde(default)]
    pub session_pinned_skill_ids: Vec<String>,
    #[serde(default)]
    pub expected_primary_skill: Option<String>,
    #[serde(default)]
    pub expected_no_primary: bool,
    #[serde(default)]
    pub expected_candidate_ids: Vec<String>,
    #[serde(default)]
    pub expected_reason_contains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct PromptContractsFile {
    #[serde(default)]
    pub contracts: Vec<PromptContractCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PromptContractCase {
    pub id: String,
    pub skill_id: String,
    #[serde(default)]
    pub required_instruction_phrases: Vec<String>,
    #[serde(default)]
    pub prohibited_instruction_phrases: Vec<String>,
    pub expected_route_policy: SkillRoutePolicy,
    pub expected_tools_allow_empty: bool,
    #[serde(default)]
    pub required_constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct SafetyFixtureFile {
    #[serde(default)]
    pub cases: Vec<SafetyCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct SafetyCase {
    pub id: String,
    pub skill_id: String,
    pub message: String,
    #[serde(default)]
    pub expected_selection: Option<String>,
    #[serde(default)]
    pub required_prompt_boundary_phrases: Vec<String>,
    #[serde(default)]
    pub prohibited_claims: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SkillEvaluationSchema {
    #[serde(default)]
    pub cases: Vec<GenericEvaluationCase>,
    #[serde(default)]
    pub contracts: Vec<PromptContractCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct GenericEvaluationCase {
    pub id: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub skill_id: Option<String>,
    #[serde(default)]
    pub enabled_skill_ids: Vec<String>,
    #[serde(default)]
    pub explicit_skill_id: Option<String>,
    #[serde(default)]
    pub ui_selected_skill_id: Option<String>,
    #[serde(default)]
    pub session_pinned_skill_ids: Vec<String>,
    #[serde(default)]
    pub expected_primary_skill: Option<String>,
    #[serde(default)]
    pub expected_no_primary: bool,
    #[serde(default)]
    pub expected_candidate_ids: Vec<String>,
    #[serde(default)]
    pub expected_reason_contains: Option<String>,
    #[serde(default)]
    pub expected_selection: Option<String>,
    #[serde(default)]
    pub required_prompt_boundary_phrases: Vec<String>,
    #[serde(default)]
    pub prohibited_claims: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct EvalReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<String>,
}

impl EvalReport {
    fn check(&mut self, passed: bool, label: impl Into<String>) {
        self.total += 1;
        if passed {
            self.passed += 1;
        } else {
            self.failed += 1;
            self.failures.push(label.into());
        }
    }
}

pub fn builtin_pack_dir(pack_id: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("packs")
        .join("builtin")
        .join(pack_id)
}

pub fn load_builtin_pack_skills(
    pack_id: &str,
    enabled: bool,
) -> anyhow::Result<Vec<SkillManifest>> {
    let skills_dir = builtin_pack_dir(pack_id).join("skills");
    if !skills_dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<_> = fs::read_dir(&skills_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
        .collect();
    paths.sort();

    let tool_registry = ToolRegistry::new(true);
    let mut skills = Vec::new();
    for path in paths {
        let mut skill: SkillManifest = crate::config_store::read_toml_file(&path)?;
        skill.pack_id = Some(skill.pack_id.unwrap_or_else(|| pack_id.to_string()));
        skill.enabled = enabled;
        validate_skill_manifest(&skill, &tool_registry).map_err(|err| anyhow::anyhow!(err))?;
        skills.push(skill);
    }
    Ok(skills)
}

pub fn validate_builtin_pack(pack_id: &str) -> anyhow::Result<Vec<String>> {
    let pack_dir = builtin_pack_dir(pack_id);
    let manifest: PackManifest = crate::config_store::read_toml_file(&pack_dir.join("pack.toml"))?;
    validate_pack_manifest(&manifest).map_err(|err| anyhow::anyhow!(err))?;
    if manifest.contents.skills && !pack_dir.join("skills").exists() {
        return Err(anyhow::anyhow!(
            "Pack '{}' declares skills but has no skills directory.",
            pack_id
        ));
    }
    let mut warnings = Vec::new();
    let tool_registry = ToolRegistry::new(true);
    let mut seen = HashSet::new();
    for mut skill in load_builtin_pack_skills(pack_id, false)? {
        skill.enabled = false;
        if !seen.insert(skill.id.clone()) {
            return Err(anyhow::anyhow!("Duplicate skill ID '{}'.", skill.id));
        }
        warnings.extend(
            validate_skill_manifest(&skill, &tool_registry).map_err(|err| anyhow::anyhow!(err))?,
        );
    }
    let evals_dir = pack_dir.join("evals");
    if pack_id == "india_student_essentials" {
        for file in [
            "routing.toml",
            "collisions.toml",
            "language_variants.toml",
            "safety.toml",
            "prompt_contracts.toml",
        ] {
            if !evals_dir.join(file).exists() {
                return Err(anyhow::anyhow!("Missing eval fixture '{}'.", file));
            }
        }
    }
    Ok(warnings)
}

pub fn route_builtin_pack(pack_id: &str, message: String) -> anyhow::Result<String> {
    let skills = load_builtin_pack_skills(pack_id, true)?;
    let decision = select_skill_route(
        &skills,
        SkillRouteRequest {
            message,
            explicit_skill_id: None,
            pack_hint: None,
            ui_selected_skill_id: None,
            session_pinned_skill_ids: vec![],
        },
    );
    Ok(serde_json::to_string_pretty(&decision)?)
}

pub fn eval_builtin_pack(pack_id: &str) -> anyhow::Result<EvalReport> {
    let pack_dir = builtin_pack_dir(pack_id);
    let skills = load_builtin_pack_skills(pack_id, true)?;
    let skills_by_id: HashMap<_, _> = skills
        .iter()
        .map(|skill| (skill.id.clone(), skill.clone()))
        .collect();
    let mut report = EvalReport::default();

    for file in ["routing.toml", "collisions.toml", "language_variants.toml"] {
        let path = pack_dir.join("evals").join(file);
        if path.exists() {
            let fixtures: RoutingFixtureFile = crate::config_store::read_toml_file(&path)?;
            eval_routing_cases(file, &skills, fixtures.cases, &mut report);
        }
    }

    let prompt_path = pack_dir.join("evals").join("prompt_contracts.toml");
    if prompt_path.exists() {
        let fixtures: PromptContractsFile = crate::config_store::read_toml_file(&prompt_path)?;
        eval_prompt_contracts(&skills_by_id, fixtures.contracts, &mut report);
    }

    let safety_path = pack_dir.join("evals").join("safety.toml");
    if safety_path.exists() {
        let fixtures: SafetyFixtureFile = crate::config_store::read_toml_file(&safety_path)?;
        eval_safety_cases(&skills, &skills_by_id, fixtures.cases, &mut report);
    }

    for skill in &skills {
        report.check(
            skill.tools.allow.is_empty(),
            format!("{} store/tools: tools.allow must be empty", skill.id),
        );
        report.check(
            !skill.store_preview.sample_prompts.is_empty()
                && !skill.store_preview.best_for.is_empty()
                && !skill.store_preview.what_it_will_not_do.is_empty(),
            format!("{} store preview must be populated", skill.id),
        );
        report.check(
            validate_skill_manifest(skill, &ToolRegistry::new(true)).is_ok(),
            format!("{} manifest validation must pass", skill.id),
        );
    }

    Ok(report)
}

fn eval_routing_cases(
    file: &str,
    skills: &[SkillManifest],
    cases: Vec<RoutingCase>,
    report: &mut EvalReport,
) {
    for case in cases {
        let enabled_ids: HashSet<_> = case.enabled_skill_ids.iter().cloned().collect();
        let scoped_skills: Vec<_> = if enabled_ids.is_empty() {
            skills.to_vec()
        } else {
            skills
                .iter()
                .cloned()
                .map(|mut skill| {
                    skill.enabled = enabled_ids.contains(&skill.id);
                    skill
                })
                .collect()
        };
        let decision = select_skill_route(
            &scoped_skills,
            SkillRouteRequest {
                message: case.message.clone(),
                explicit_skill_id: case.explicit_skill_id.clone(),
                pack_hint: None,
                ui_selected_skill_id: case.ui_selected_skill_id.clone(),
                session_pinned_skill_ids: case.session_pinned_skill_ids.clone(),
            },
        );
        let primary = decision
            .primary_skill
            .as_ref()
            .map(|skill| skill.id.as_str());
        if case.expected_no_primary {
            report.check(
                primary.is_none(),
                format!("{file}/{} expected no primary, got {:?}", case.id, primary),
            );
        }
        if let Some(expected) = case.expected_primary_skill.as_deref() {
            report.check(
                primary == Some(expected),
                format!(
                    "{file}/{} expected primary {expected}, got {:?}",
                    case.id, primary
                ),
            );
        }
        for expected_candidate in &case.expected_candidate_ids {
            report.check(
                decision
                    .candidates
                    .iter()
                    .any(|candidate| candidate.id == *expected_candidate),
                format!(
                    "{file}/{} expected candidate {expected_candidate}, got {:?}",
                    case.id, decision.candidates
                ),
            );
        }
        if let Some(reason) = case.expected_reason_contains.as_deref() {
            report.check(
                decision.reason.contains(reason)
                    || decision
                        .candidates
                        .iter()
                        .any(|candidate| candidate.reason.contains(reason)),
                format!("{file}/{} expected reason containing {reason}", case.id),
            );
        }
    }
}

fn eval_prompt_contracts(
    skills_by_id: &HashMap<String, SkillManifest>,
    cases: Vec<PromptContractCase>,
    report: &mut EvalReport,
) {
    for case in cases {
        let Some(skill) = skills_by_id.get(&case.skill_id) else {
            report.check(false, format!("{} missing skill", case.id));
            continue;
        };
        report.check(
            skill.route_policy == case.expected_route_policy,
            format!("{} route policy mismatch", case.id),
        );
        if case.expected_tools_allow_empty {
            report.check(
                skill.tools.allow.is_empty(),
                format!("{} tools.allow is not empty", case.id),
            );
        }
        for phrase in &case.required_instruction_phrases {
            report.check(
                skill.prompt.instructions.contains(phrase)
                    || skill
                        .prompt
                        .constraints
                        .iter()
                        .any(|item| item.contains(phrase)),
                format!("{} missing required phrase '{}'", case.id, phrase),
            );
        }
        for phrase in &case.prohibited_instruction_phrases {
            report.check(
                !skill.prompt.instructions.contains(phrase),
                format!("{} contains prohibited phrase '{}'", case.id, phrase),
            );
        }
        for phrase in &case.required_constraints {
            report.check(
                skill
                    .prompt
                    .constraints
                    .iter()
                    .any(|item| item.contains(phrase)),
                format!("{} missing required constraint '{}'", case.id, phrase),
            );
        }
    }
}

fn eval_safety_cases(
    skills: &[SkillManifest],
    skills_by_id: &HashMap<String, SkillManifest>,
    cases: Vec<SafetyCase>,
    report: &mut EvalReport,
) {
    for case in cases {
        let Some(skill) = skills_by_id.get(&case.skill_id) else {
            report.check(false, format!("{} missing skill", case.id));
            continue;
        };
        let decision = select_skill_route(
            skills,
            SkillRouteRequest {
                message: case.message.clone(),
                explicit_skill_id: None,
                pack_hint: None,
                ui_selected_skill_id: None,
                session_pinned_skill_ids: vec![],
            },
        );
        let primary = decision
            .primary_skill
            .as_ref()
            .map(|selected| selected.id.as_str());
        match case.expected_selection.as_deref() {
            Some("none") => report.check(
                primary.is_none(),
                format!(
                    "{} expected no safety selection, got {:?}",
                    case.id, primary
                ),
            ),
            Some(expected) => report.check(
                primary == Some(expected),
                format!(
                    "{} expected safety selection {expected}, got {:?}",
                    case.id, primary
                ),
            ),
            None => {}
        }
        let prompt = format!(
            "{}\n{}",
            skill.prompt.instructions,
            skill.prompt.constraints.join("\n")
        );
        for phrase in &case.required_prompt_boundary_phrases {
            report.check(
                prompt.contains(phrase),
                format!("{} missing boundary phrase '{}'", case.id, phrase),
            );
        }
        for claim in &case.prohibited_claims {
            report.check(
                !prompt.contains(claim),
                format!("{} contains prohibited claim '{}'", case.id, claim),
            );
        }
    }
}

pub fn generate_schema_files() -> anyhow::Result<Vec<PathBuf>> {
    let schemas_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas");
    fs::create_dir_all(&schemas_dir)?;
    let files = [
        (
            "skill-manifest.schema.json",
            serde_json::to_string_pretty(&schemars::schema_for!(SkillManifest))?,
        ),
        (
            "skill-pack.schema.json",
            serde_json::to_string_pretty(&schemars::schema_for!(PackManifest))?,
        ),
        (
            "skill-evaluation.schema.json",
            serde_json::to_string_pretty(&schemars::schema_for!(SkillEvaluationSchema))?,
        ),
    ];
    let mut written = Vec::new();
    for (name, content) in files {
        let path = schemas_dir.join(name);
        fs::write(&path, content)?;
        written.push(path);
    }
    Ok(written)
}

pub fn ensure_schema_files_current() -> anyhow::Result<()> {
    let before = read_schema_files();
    let written = generate_schema_files()?;
    let after = read_schema_files();
    if before != after {
        return Err(anyhow::anyhow!(
            "Schema files were stale and have been regenerated: {:?}",
            written
        ));
    }
    Ok(())
}

fn read_schema_files() -> HashMap<String, String> {
    let schemas_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas");
    [
        "skill-manifest.schema.json",
        "skill-pack.schema.json",
        "skill-evaluation.schema.json",
    ]
    .into_iter()
    .map(|name| {
        let path = schemas_dir.join(name);
        (
            name.to_string(),
            fs::read_to_string(path).unwrap_or_default(),
        )
    })
    .collect()
}

pub fn report_builtin_pack(pack_id: &str) -> anyhow::Result<String> {
    let skills = load_builtin_pack_skills(pack_id, false)?;
    let eval = eval_builtin_pack(pack_id)?;
    let policy_counts = skills
        .iter()
        .fold(HashMap::<String, usize>::new(), |mut acc, skill| {
            *acc.entry(format!("{:?}", skill.route_policy)).or_default() += 1;
            acc
        });
    Ok(format!(
        "pack={pack_id}\nskill_count={}\nroute_policies={:?}\nvalidation_warnings={:?}\neval_total={}\neval_passed={}\neval_failed={}\nsafety_sensitive_cases={}",
        skills.len(),
        policy_counts,
        validate_builtin_pack(pack_id)?,
        eval.total,
        eval.passed,
        eval.failed,
        read_safety_count(&builtin_pack_dir(pack_id).join("evals").join("safety.toml")),
    ))
}

fn read_safety_count(path: &Path) -> usize {
    crate::config_store::read_toml_file::<SafetyFixtureFile>(path)
        .map(|file| file.cases.len())
        .unwrap_or(0)
}

#[allow(dead_code)]
fn _schema_types_keep_public(_: SkillToolPolicy, _: SkillSafety) {}
