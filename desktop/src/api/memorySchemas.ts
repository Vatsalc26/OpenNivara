import { z } from "zod";

const MemoryModeSchema = z.enum([
	"off",
	"ask_before_saving",
	"auto_save_low_risk",
	"full_life_journal",
]);

export const MemorySettingsSchema = z.object({
	schema_version: z.number(),
	mode: MemoryModeSchema,
	pause_memory: z.boolean(),
	private_chat: z.boolean(),
	allow_location_memories: z.boolean(),
	sensitive_approval_required: z.boolean(),
});

export const MemoryStatusSchema = z.object({
	db_path: z.string(),
	initialized: z.boolean(),
	schema_version: z.number(),
	item_count: z.number(),
	proposal_count: z.number(),
	vector_enabled: z.boolean(),
});

export const MemoryItemSchema = z.object({
	id: z.string(),
	memory_type: z.string(),
	title: z.string(),
	summary: z.string(),
	details_json: z.string(),
	status: z.string(),
	confidence: z.number(),
	user_verified: z.boolean(),
	sensitivity: z.string(),
	visibility: z.string(),
	source_id: z.string(),
	created_at: z.string(),
	updated_at: z.string(),
	observed_at: z.string(),
	valid_from: z.string().nullable(),
	valid_until: z.string().nullable(),
	happened_at: z.string().nullable(),
	starts_at: z.string().nullable(),
	ends_at: z.string().nullable(),
	due_at: z.string().nullable(),
	completed_at: z.string().nullable(),
	timezone: z.string(),
	time_precision: z.string(),
	natural_time_phrase: z.string().nullable(),
	recurrence_rule: z.string().nullable(),
	superseded_by: z.string().nullable(),
	deleted_at: z.string().nullable(),
});

export const MemorySearchResultSchema = z.object({
	item: MemoryItemSchema,
	score: z.number(),
	reason: z.string(),
	answerability: z.string(),
});

const LocationContextSchema = z.object({
	status: z.string(),
	latitude: z.number().nullable(),
	longitude: z.number().nullable(),
	accuracy_meters: z.number().nullable(),
	source: z.string(),
	captured_at: z.string().nullable(),
	freshness_seconds: z.number().nullable(),
	timezone_hint: z.string().nullable(),
	city: z.string().nullable(),
	region: z.string().nullable(),
	country: z.string().nullable(),
	label: z.string().nullable(),
	permission_state: z.string(),
	privacy_level: z.string(),
});

export const RuntimeContextSchema = z.object({
	now_utc: z.string(),
	now_local: z.string(),
	timezone: z.string(),
	date_local: z.string(),
	day_of_week: z.string(),
	locale: z.string().nullable(),
	calendar_week: z.number().nullable(),
	relative_date_context: z.object({
		today_start: z.string(),
		today_end: z.string(),
		tomorrow_start: z.string(),
		tomorrow_end: z.string(),
		yesterday_start: z.string(),
		yesterday_end: z.string(),
		current_week_start: z.string(),
		current_week_end: z.string(),
		next_week_start: z.string(),
		next_week_end: z.string(),
		current_month_start: z.string(),
		current_month_end: z.string(),
		next_month_start: z.string(),
		next_month_end: z.string(),
	}),
	location: LocationContextSchema,
	model: z.object({
		provider: z.string(),
		model_name: z.string(),
		context_window_tokens: z.number(),
		default_reserved_output_tokens: z.number(),
		supports_token_counting: z.boolean(),
		supports_usage_metadata: z.boolean(),
		tokenizer_strategy: z.string(),
	}),
});

export const SavedPlaceSchema = z.object({
	id: z.string(),
	label: z.string(),
	place_type: z.string(),
	latitude: z.number().nullable(),
	longitude: z.number().nullable(),
	address: z.string().nullable(),
	city: z.string().nullable(),
	region: z.string().nullable(),
	country: z.string().nullable(),
	timezone: z.string().nullable(),
	details_json: z.string(),
	created_at: z.string(),
	updated_at: z.string(),
	deleted_at: z.string().nullable(),
});

const MemoryGraphNodeSchema = z.object({
	id: z.string(),
	node_type: z.string(),
	source_table: z.string(),
	source_id: z.string(),
	label: z.string(),
	properties_json: z.string(),
	sensitivity: z.string(),
	updated_at: z.string(),
});

const MemoryGraphEdgeSchema = z.object({
	id: z.string(),
	from_node_id: z.string(),
	edge_type: z.string(),
	to_node_id: z.string(),
	weight: z.number(),
	confidence: z.number(),
	source_memory_id: z.string().nullable(),
	properties_json: z.string(),
	valid_from: z.string().nullable(),
	valid_until: z.string().nullable(),
	created_at: z.string(),
	updated_at: z.string(),
});

export const MemoryGraphContextSchema = z.object({
	focus_node_id: z.string(),
	nodes: z.array(MemoryGraphNodeSchema),
	edges: z.array(MemoryGraphEdgeSchema),
	depth: z.number(),
});

export const MemoryGraphStatusSchema = z.object({
	node_count: z.number(),
	edge_count: z.number(),
	index_count: z.number(),
	validation_errors: z.array(z.string()),
});

export const MemoryExtractionProposalSchema = z.object({
	id: z.string(),
	source_id: z.string(),
	proposal_json: z.string(),
	sensitivity: z.string(),
	confidence: z.number(),
	status: z.string(),
	created_at: z.string(),
});

export const ContextCompilerOutputSchema = z.object({
	system_policy: z.string(),
	current_user_message: z.string(),
	recent_conversation_window: z.string(),
	session_summary: z.string(),
	profile_brief: z.string(),
	style_brief: z.string(),
	preference_brief: z.string(),
	memory_brief: z.string(),
	task_reminder_brief: z.string(),
	workspace_brief: z.string(),
	route_brief: z.string(),
	raw_prompt: z.string(),
	token_budget_report: z.object({
		model_context_limit: z.number(),
		reserved_output_tokens: z.number(),
		input_budget_tokens: z.number(),
		estimated_prompt_tokens: z.number(),
		trimmed_sections: z.array(z.string()),
		sections: z.array(
			z.object({
				section: z.string(),
				priority: z.number(),
				estimated_tokens: z.number(),
				included: z.boolean(),
				reason: z.string(),
			}),
		),
		notes: z.array(z.string()),
	}),
	audit: z.any(),
	intent: z.object({
		labels: z.array(z.string()),
		confidence: z.number(),
		reason: z.string(),
	}),
	included_memory_ids: z.array(z.string()),
	included_graph_edge_ids: z.array(z.string()),
	runtime_decision: z.string(),
	location_decision: z.string(),
});
