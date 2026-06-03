import { z } from "zod";
import { SkillManifestSchema } from "./skillSchemas";

export const PackCompatibilitySchema = z.object({
	opennivara_min_version: z.string(),
	opennivara_max_version: z.string().default(""),
});

export const PackContentsSchema = z.object({
	preferences: z.boolean(),
	contexts: z.boolean(),
	style_presets: z.boolean(),
	profile_templates: z.boolean(),
	tool_presets: z.boolean(),
	workspace_map_rules: z.boolean(),
	prompt_behaviors: z.boolean(),
	command_snippets: z.boolean(),
	theme: z.boolean(),
	skills: z.boolean().default(false),
});

export const PackSafetySchema = z.object({
	contains_executable_code: z.boolean(),
	modifies_tool_permissions: z.boolean(),
	requires_network: z.boolean(),
	risk_level: z.enum(["low", "medium", "high"]),
});

export const PackManifestSchema = z.object({
	schema_version: z.number(),
	id: z.string(),
	name: z.string(),
	version: z.string(),
	author: z.string(),
	category: z.string(),
	description: z.string(),
	homepage: z.string().default(""),
	license: z.string().default(""),
	compatibility: PackCompatibilitySchema,
	contents: PackContentsSchema,
	safety: PackSafetySchema,
});

export const InstalledPackSchema = z.object({
	id: z.string(),
	name: z.string(),
	version: z.string(),
	installed_at: z.string(),
	source: z.string(),
	enabled: z.boolean(),
});

export const InstalledPacksFileSchema = z.object({
	schema_version: z.number(),
	installed: z.array(InstalledPackSchema),
});

export const OpenNivaraModeSchema = z.object({
	id: z.string(),
	name: z.string(),
	description: z.string(),
	enabled_pack_ids: z.array(z.string()),
	theme_id: z.string().nullable().optional(),
	style_pack_id: z.string().nullable().optional(),
	style_preset_id: z.string().nullable().optional(),
});

export const ModesFileSchema = z.object({
	schema_version: z.number(),
	active_mode: z.string(),
	modes: z.array(OpenNivaraModeSchema),
});

export const PackAdditionsSummarySchema = z.object({
	preferences_count: z.number(),
	contexts_count: z.number(),
	style_presets_count: z.number(),
	themes_count: z.number(),
	command_snippets_count: z.number(),
	workspace_rules_count: z.number(),
	profile_templates_count: z.number(),
	tool_presets_count: z.number(),
	skills_count: z.number().default(0),
});

export const PackSafetySummarySchema = z.object({
	allowed_to_install: z.boolean(),
	risk_level: z.string(),
	modifies_tool_permissions: z.boolean(),
	contains_executable_code: z.boolean(),
	requires_network: z.boolean(),
});

export const PackPreviewSchema = z.object({
	manifest: PackManifestSchema,
	source_path: z.string(),
	warnings: z.array(z.string()),
	errors: z.array(z.string()),
	additions: PackAdditionsSummarySchema,
	safety_summary: PackSafetySummarySchema,
	skill_previews: z.array(SkillManifestSchema).default([]),
});

export const ThemeColorsSchema = z.object({
	background: z.string(),
	panel: z.string(),
	card: z.string(),
	primary: z.string(),
	accent: z.string(),
	success: z.string(),
	warning: z.string(),
	danger: z.string(),
	foreground: z.string(),
	muted: z.string(),
});

export const ThemeEffectsSchema = z.object({
	background_gradient: z.boolean(),
	glow: z.string(),
	density: z.string(),
});

export const OpenNivaraThemeSchema = z.object({
	schema_version: z.number(),
	id: z.string(),
	name: z.string(),
	description: z.string(),
	colors: ThemeColorsSchema,
	effects: ThemeEffectsSchema,
});

export const ThemeSafetySchema = z.object({
	data_only: z.literal(true),
	contains_executable_code: z.literal(false),
	modifies_tool_security: z.literal(false),
	requires_network: z.literal(false),
});

export const ThemeStoreItemSchema = z.object({
	id: z.string(),
	name: z.string(),
	description: z.string(),
	author: z.string(),
	version: z.string(),
	source_kind: z.enum(["builtin", "local", "installed"]).or(z.string()),
	installed: z.boolean(),
	applied: z.boolean(),
	preview_colors: ThemeColorsSchema,
	safety: ThemeSafetySchema,
});

export const InstalledThemeSchema = z.object({
	id: z.string(),
	name: z.string(),
	version: z.string(),
	source_kind: z.enum(["builtin", "local", "installed"]).or(z.string()),
	installed_at: z.string(),
	manifest_path: z.string(),
});

export const AppearanceSettingsSchema = z.object({
	schema_version: z.number(),
	active_theme_id: z.string().nullable().optional(),
	active_theme_source: z.string().nullable().optional(),
});

export const ThemeManifestSchema = z.object({
	id: z.string(),
	name: z.string(),
	description: z.string(),
	author: z.string(),
	version: z.string(),
	source_kind: z.string(),
	safety: ThemeSafetySchema,
});

export const ThemePreviewSchema = z.object({
	manifest: ThemeManifestSchema,
	theme: OpenNivaraThemeSchema,
	installed: z.boolean(),
	applied: z.boolean(),
});

export const CommandSnippetSchema = z.object({
	id: z.string(),
	title: z.string(),
	description: z.string(),
	category: z.string(),
	prompt: z.string(),
	tags: z.array(z.string()).default([]),
});

export const BuiltinPackSummarySchema = z.object({
	id: z.string(),
	name: z.string(),
	version: z.string(),
	author: z.string(),
	category: z.string(),
	description: z.string(),
	risk_level: z.string(),
});

export const MarketplaceRepairReportSchema = z.object({
	repaired: z.boolean(),
	actions: z.array(z.string()),
	warnings: z.array(z.string()),
	errors: z.array(z.string()),
});

export const MarketplaceStatusSchema = z.object({
	marketplace_dir: z.string(),
	installed_count: z.number(),
	enabled_count: z.number(),
	disabled_count: z.number(),
	modes_count: z.number(),
	active_mode_id: z.string(),
	active_mode_name: z.string(),
	active_theme_id: z.string().nullable().optional(),
	active_theme_name: z.string().nullable().optional(),
	missing_pack_ids: z.array(z.string()),
	disabled_packs_in_active_mode: z.array(z.string()),
	builtin_packs_available: z.array(z.string()),
	builtin_resource_path_checked: z.string(),
	builtin_resource_path_exists: z.boolean(),
});

export const PackActivationCapabilitiesSchema = z.object({
	pack_id: z.string(),
	has_theme: z.boolean(),
	theme_id: z.string().nullable().optional(),
	theme_name: z.string().nullable().optional(),
	has_style: z.boolean(),
	has_preferences: z.boolean(),
	has_contexts: z.boolean(),
	has_command_snippets: z.boolean(),
	has_workspace_rules: z.boolean(),
});

export const ModeActivationResultSchema = z.object({
	mode_id: z.string(),
	pack_id: z.string(),
	added_pack: z.boolean(),
	applied_theme_id: z.string().nullable().optional(),
	applied_style_pack_id: z.string().nullable().optional(),
	warnings: z.array(z.string()),
});

export const AddonSettingsSchema = z.object({
	schema_version: z.number(),
	active_theme_id: z.string().nullable().optional(),
	active_theme_source_pack_id: z.string().nullable().optional(),
	enabled_packs: z.array(z.string()),
	disabled_contributions: z.array(z.string()),
});

export const AddonContributionPreviewSchema = z.object({
	pack_id: z.string(),
	pack_name: z.string(),
	contribution_id: z.string(),
	title: z.string(),
	description: z.string(),
	enabled: z.boolean(),
});

export const QuickPromptContributionPreviewSchema = z.object({
	pack_id: z.string(),
	pack_name: z.string(),
	prompt_id: z.string(),
	title: z.string(),
	description: z.string(),
	prompt_body: z.string(),
	category: z.string(),
	enabled: z.boolean(),
});

export const EffectiveSettingsPreviewSchema = z.object({
	base_preferences: z.array(z.any()),
	addon_preferences: z.array(AddonContributionPreviewSchema),
	base_contexts: z.array(z.any()),
	addon_contexts: z.array(AddonContributionPreviewSchema),
	addon_quick_prompts: z.array(QuickPromptContributionPreviewSchema),
	active_theme_id: z.string().nullable().optional(),
	active_theme_name: z.string().nullable().optional(),
	active_theme_source_pack_id: z.string().nullable().optional(),
	active_style_pack_id: z.string().nullable().optional(),
	active_style_pack_name: z.string().nullable().optional(),
	disabled_contributions: z.array(z.string()),
	enabled_packs: z.array(z.string()),
});

export const InstalledThemeSummarySchema = z.object({
	theme_id: z.string(),
	theme_name: z.string(),
	description: z.string(),
	source_pack_id: z.string(),
	source_pack_name: z.string(),
	pack_enabled: z.boolean(),
});
