use super::types::{TokenBudgetReport, TokenBudgetSection};
use crate::runtime::model_registry::{estimate_tokens as model_estimate_tokens, ModelContextInfo};

pub fn estimate_tokens(text: &str) -> u32 {
    ((text.chars().count() as f64) / 4.0).ceil() as u32
}

pub fn build_report(
    model_context_limit: u32,
    reserved_output_tokens: u32,
    raw_prompt: &str,
    notes: Vec<String>,
) -> TokenBudgetReport {
    let input_budget_tokens = model_context_limit.saturating_sub(reserved_output_tokens);
    TokenBudgetReport {
        model_context_limit,
        reserved_output_tokens,
        input_budget_tokens,
        estimated_prompt_tokens: estimate_tokens(raw_prompt),
        trimmed_sections: vec![],
        sections: vec![],
        notes,
    }
}

#[derive(Debug, Clone)]
pub struct PromptSectionBudget {
    pub label: String,
    pub body: String,
    pub priority: u32,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct BudgetedPrompt {
    pub raw_prompt: String,
    pub included_sections: Vec<PromptSectionBudget>,
    pub report: TokenBudgetReport,
}

pub fn build_budgeted_prompt(
    model: &ModelContextInfo,
    model_context_limit: u32,
    reserved_output_tokens: u32,
    sections: Vec<PromptSectionBudget>,
    notes: Vec<String>,
) -> BudgetedPrompt {
    let effective_context_limit = model_context_limit.min(model.context_window_tokens);
    let effective_reserved = if reserved_output_tokens == 0 {
        model.default_reserved_output_tokens
    } else {
        reserved_output_tokens
    }
    .min(effective_context_limit);
    let input_budget_tokens = effective_context_limit.saturating_sub(effective_reserved);

    let mut tracked: Vec<(PromptSectionBudget, TokenBudgetSection)> = sections
        .into_iter()
        .filter(|section| !section.body.trim().is_empty())
        .map(|section| {
            let estimated_tokens = model_estimate_tokens(&format_section(&section), model);
            let budget_section = TokenBudgetSection {
                section: section.label.clone(),
                priority: section.priority,
                estimated_tokens,
                included: true,
                reason: "included".into(),
            };
            (section, budget_section)
        })
        .collect();

    while tracked
        .iter()
        .filter(|(_, budget)| budget.included)
        .map(|(_, budget)| budget.estimated_tokens)
        .sum::<u32>()
        > input_budget_tokens
    {
        let Some((_, budget)) = tracked
            .iter_mut()
            .filter(|(section, budget)| !section.required && budget.included)
            .max_by_key(|(section, budget)| (section.priority, budget.estimated_tokens))
        else {
            break;
        };
        budget.included = false;
        budget.reason = "trimmed:token_budget".into();
    }

    let included_sections: Vec<_> = tracked
        .iter()
        .filter(|(_, budget)| budget.included)
        .map(|(section, _)| section.clone())
        .collect();
    let raw_prompt = included_sections
        .iter()
        .map(format_section)
        .collect::<Vec<_>>()
        .join("\n\n");
    let sections_report: Vec<_> = tracked.into_iter().map(|(_, budget)| budget).collect();
    let trimmed_sections = sections_report
        .iter()
        .filter(|section| !section.included)
        .map(|section| section.section.clone())
        .collect::<Vec<_>>();

    BudgetedPrompt {
        raw_prompt: raw_prompt.clone(),
        included_sections,
        report: TokenBudgetReport {
            model_context_limit: effective_context_limit,
            reserved_output_tokens: effective_reserved,
            input_budget_tokens,
            estimated_prompt_tokens: model_estimate_tokens(&raw_prompt, model),
            trimmed_sections,
            sections: sections_report,
            notes,
        },
    }
}

fn format_section(section: &PromptSectionBudget) -> String {
    format!("[{}]\n{}", section.label, section.body)
}
