import { describe, expect, test } from "vitest";
import { PackManifestSchema, PackSafetySchema } from "./marketplaceSchemas";
import {
	ContextPreviewSchema,
	PreferencesSchema,
	StyleSchema,
} from "./schemas";

describe("API schema contracts", () => {
	test("accepts context preview payloads without store behavior metadata", () => {
		const parsed = ContextPreviewSchema.parse({
			profile_sent: ["identity.display_name"],
			style_sent: ["communication.tone"],
			preferences_sent: [],
			contexts_sent: ["project"],
			contexts_pinned: [],
			contexts_not_sent: ["private"],
			warnings: [],
			final_context_text: "context",
			active_theme: {
				id: "coding_cyan",
				name: "Coding Cyan",
				ui_only: true,
			},
		});

		expect(parsed.active_theme?.ui_only).toBe(true);
		expect("active_packs" in parsed).toBe(false);
		expect("active_mode" in parsed).toBe(false);
		expect("style_source_pack" in parsed).toBe(false);
	});

	test("rejects invalid marketplace safety risk levels", () => {
		expect(() =>
			PackSafetySchema.parse({
				contains_executable_code: false,
				modifies_tool_permissions: false,
				requires_network: false,
				risk_level: "critical",
			}),
		).toThrow();
	});

	test("fills marketplace manifest default strings", () => {
		const parsed = PackManifestSchema.parse({
			schema_version: 1,
			id: "coding_basics",
			name: "Coding Basics Pack",
			version: "1.0.0",
			author: "Vatsal Chavda",
			category: "coding",
			description: "Adds coding focused defaults.",
			compatibility: {
				opennivara_min_version: "0.1.0",
			},
			contents: {
				preferences: true,
				contexts: true,
				style_presets: true,
				profile_templates: false,
				tool_presets: false,
				workspace_map_rules: false,
				prompt_behaviors: false,
				command_snippets: true,
				theme: true,
			},
			safety: {
				contains_executable_code: false,
				modifies_tool_permissions: false,
				requires_network: false,
				risk_level: "low",
			},
		});

		expect(parsed.homepage).toBe("");
		expect(parsed.license).toBe("");
		expect(parsed.compatibility.opennivara_max_version).toBe("");
	});

	test("keeps preferences and style payloads structurally strict", () => {
		expect(() =>
			PreferencesSchema.parse({
				schema_version: 1,
				sections: [
					{
						id: "coding",
						enabled: true,
						send_policy: "always",
						triggers: [],
						required_any: [],
						negative_triggers: [],
						min_score: 0,
						likes: [{ item: "rust", strength: "5" }],
						dislikes: [],
						notes: [],
					},
				],
			}),
		).toThrow();

		expect(() =>
			StyleSchema.parse({
				schema_version: 1,
				communication: {
					tone: "direct",
					detail_level: "medium",
					use_examples: true,
					use_step_by_step: true,
					avoid_unexplained_jargon: true,
					ask_fewer_questions: true,
					prefer_actionable_answers: true,
				},
				coding: {
					show_simple_solution_first: true,
					explain_after_code: true,
					prefer_mvp_architecture: true,
					avoid_overengineering: true,
					use_beginner_comments: true,
				},
				formatting: {
					use_markdown: true,
					use_short_sections: true,
					include_next_step: true,
					avoid_long_walls_of_text: true,
				},
				behavior: {
					be_honest_about_uncertainty: true,
					do_not_pretend_to_have_done_things: true,
				},
			}),
		).toThrow();
	});
});
