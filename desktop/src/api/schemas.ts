import { z } from "zod";

// --- Style Zod Schema ---
export const StyleSchema = z.object({
	schema_version: z.number(),
	communication: z.object({
		tone: z.string(),
		detail_level: z.string(),
		use_examples: z.boolean(),
		use_step_by_step: z.boolean(),
		avoid_unexplained_jargon: z.boolean(),
		ask_fewer_questions: z.boolean(),
		prefer_actionable_answers: z.boolean(),
	}),
	coding: z.object({
		show_simple_solution_first: z.boolean(),
		explain_after_code: z.boolean(),
		prefer_mvp_architecture: z.boolean(),
		avoid_overengineering: z.boolean(),
		use_beginner_comments: z.boolean(),
	}),
	formatting: z.object({
		use_markdown: z.boolean(),
		use_short_sections: z.boolean(),
		include_next_step: z.boolean(),
		avoid_long_walls_of_text: z.boolean(),
	}),
	behavior: z.object({
		be_honest_about_uncertainty: z.boolean(),
		do_not_pretend_to_have_done_things: z.boolean(),
		do_not_reveal_private_context_unless_relevant: z.boolean(),
	}),
});

// --- Profile Zod Schema ---
export const ProfileSchema = z.object({
	schema_version: z.number(),
	identity: z.object({
		display_name: z.string(),
		full_name: z.string().nullable().or(z.string()),
		gender: z.string().nullable().or(z.string()),
		pronouns: z.string().nullable().or(z.string()),
		date_of_birth: z.string().nullable().or(z.string()),
		timezone: z.string().nullable().or(z.string()),
	}),
	location: z.object({
		country: z.string().nullable().or(z.string()),
		state_or_region: z.string().nullable().or(z.string()),
		city: z.string().nullable().or(z.string()),
		living_situation: z.string().nullable().or(z.string()),
	}),
	languages: z.object({
		preferred_human_language: z.string(),
		other_human_languages: z.array(z.string()),
	}),
	technical: z.object({
		coding_level: z.string(),
		preferred_coding_languages: z.array(z.string()),
		current_os: z.string().nullable().or(z.string()),
		main_editor: z.string().nullable().or(z.string()),
		secondary_editor: z.string().nullable().or(z.string()),
		terminal: z.string().nullable().or(z.string()),
	}),
	personal: z.object({
		occupation_or_role: z.string().nullable().or(z.string()),
		education_level: z.string().nullable().or(z.string()),
		interests: z.array(z.string()),
	}),
	privacy: z.object({
		send_identity: z.boolean(),
		send_location: z.boolean(),
		send_gender: z.boolean(),
		send_technical: z.boolean(),
		send_personal: z.boolean(),
	}),
});

// --- Preferences Zod Schema ---
export const PreferenceItemSchema = z.object({
	item: z.string(),
	strength: z.number(),
});

export const PreferenceSectionSchema = z.object({
	id: z.string(),
	enabled: z.boolean(),
	send_policy: z.string(),
	description: z.string().optional(),
	triggers: z.array(z.string()),
	required_any: z.array(z.string()),
	negative_triggers: z.array(z.string()),
	min_score: z.number(),
	likes: z.array(PreferenceItemSchema),
	dislikes: z.array(PreferenceItemSchema),
	notes: z.array(z.string()),
});

export const PreferencesSchema = z.object({
	schema_version: z.number(),
	sections: z.array(PreferenceSectionSchema),
});

// --- Contexts Zod Schema ---
export const ContextEntrySchema = z.object({
	id: z.string(),
	enabled: z.boolean(),
	kind: z.string(),
	send_policy: z.string(),
	title: z.string(),
	summary: z.string(),
	triggers: z.array(z.string()),
	required_any: z.array(z.string()),
	negative_triggers: z.array(z.string()),
	min_score: z.number(),
	facts: z.array(z.string()),
	rules: z.array(z.string()),
});

export const ContextsSchema = z.object({
	schema_version: z.number(),
	contexts: z.array(ContextEntrySchema),
});

// --- ContextPreview Zod Schema ---
export const ContextPreviewSchema = z.object({
	profile_sent: z.array(z.string()),
	style_sent: z.array(z.string()),
	preferences_sent: z.array(z.string()),
	contexts_sent: z.array(z.string()),
	contexts_pinned: z.array(z.string()),
	contexts_not_sent: z.array(z.string()),
	warnings: z.array(z.string()),
	final_context_text: z.string(),
	active_theme: z
		.object({
			id: z.string(),
			name: z.string(),
			ui_only: z.literal(true),
		})
		.nullable()
		.optional(),
});
