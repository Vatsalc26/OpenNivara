import { z } from "zod";

const SkillRoutePolicySchema = z.enum([
	"auto",
	"manual_only",
	"explicit_only",
	"suggest_only",
	"disabled",
]);

const SkillPromptSchema = z.object({
	role: z.string(),
	instructions: z.string(),
	constraints: z.array(z.string()).default([]),
});

const SkillToolPolicySchema = z.object({
	allow: z.array(z.string()).default([]),
	deny: z.array(z.string()).default([]),
});

const SkillSafetySchema = z.object({
	risk_level: z.string(),
	requires_confirmation: z.boolean(),
	allows_file_write: z.boolean(),
	allows_shell: z.boolean(),
	allows_network: z.boolean(),
	requires_fresh_info: z.boolean().default(false),
});

const SkillMetadataSchema = z.object({
	audience: z.array(z.string()).default([]),
	country: z.string().nullable().optional(),
	exam: z.string().nullable().optional(),
	exam_stage: z.string().nullable().optional(),
	language_style: z.array(z.string()).default([]),
	last_reviewed_at: z.string().nullable().optional(),
	freshness_sensitive: z.boolean().default(false),
	official_source_labels: z.array(z.string()).default([]),
});

const SkillStorePreviewSchema = z.preprocess(
	(value) => {
		if (value && typeof value === "object" && !Array.isArray(value)) {
			const record = value as Record<string, unknown>;
			if (!("what_it_does" in record) && "what_it_will_do" in record) {
				return { ...record, what_it_does: record.what_it_will_do };
			}
		}
		return value;
	},
	z.object({
		best_for: z.array(z.string()).default([]),
		not_for: z.array(z.string()).default([]),
		sample_prompts: z.array(z.string()).default([]),
		what_it_does: z.array(z.string()).default([]),
		what_it_will_not_do: z.array(z.string()).default([]),
	}),
);

export const SkillManifestSchema = z.object({
	schema_version: z.number(),
	id: z.string(),
	pack_id: z.string().nullable().optional(),
	name: z.string(),
	description: z.string(),
	enabled: z.boolean(),
	category: z.string(),
	route_policy: SkillRoutePolicySchema,
	aliases: z.array(z.string()).default([]),
	triggers: z.array(z.string()).default([]),
	required_any: z.array(z.string()).default([]),
	negative_triggers: z.array(z.string()).default([]),
	examples: z.array(z.string()).default([]),
	min_score: z.number(),
	prompt: SkillPromptSchema,
	tools: SkillToolPolicySchema,
	safety: SkillSafetySchema,
	metadata: SkillMetadataSchema.default({
		audience: [],
		country: undefined,
		exam: undefined,
		exam_stage: undefined,
		language_style: [],
		last_reviewed_at: undefined,
		freshness_sensitive: false,
		official_source_labels: [],
	}),
	store_preview: SkillStorePreviewSchema.default({
		best_for: [],
		not_for: [],
		sample_prompts: [],
		what_it_does: [],
		what_it_will_not_do: [],
	}),
});

export const SkillSummarySchema = z.object({
	id: z.string(),
	pack_id: z.string().nullable().optional(),
	name: z.string(),
	description: z.string(),
	category: z.string(),
	enabled: z.boolean(),
	route_policy: SkillRoutePolicySchema,
	risk_level: z.string(),
	allowed_tools: z.array(z.string()),
	denied_tools: z.array(z.string()).default([]),
	exam: z.string().nullable().optional(),
	exam_stage: z.string().nullable().optional(),
	audience: z.array(z.string()).default([]),
	language_style: z.array(z.string()).default([]),
	freshness_sensitive: z.boolean().default(false),
	official_source_labels: z.array(z.string()).default([]),
	best_for: z.array(z.string()).default([]),
	not_for: z.array(z.string()).default([]),
});

const SelectedSkillSchema = z.object({
	id: z.string(),
	pack_id: z.string().nullable().optional(),
	name: z.string(),
	score: z.number(),
	reason: z.string(),
	allowed_tools: z.array(z.string()),
	denied_tools: z.array(z.string()),
});

const SkillCandidateSchema = z.object({
	id: z.string(),
	name: z.string(),
	score: z.number(),
	accepted: z.boolean(),
	reason: z.string(),
});

export const RouteDecisionSchema = z.object({
	primary_skill: SelectedSkillSchema.nullable().optional(),
	supporting_skills: z.array(SelectedSkillSchema),
	candidates: z.array(SkillCandidateSchema),
	confidence: z.number(),
	reason: z.string(),
	warnings: z.array(z.string()),
});
