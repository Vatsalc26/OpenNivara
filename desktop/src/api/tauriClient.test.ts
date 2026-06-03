import { beforeEach, describe, expect, test } from "vitest";
import { clearTauriMocks, mockTauriCommand } from "../test/mockTauri";
import {
	tauriAskOpenNivara,
	tauriCheckApiKey,
	tauriGetContexts,
	tauriGetContextsPath,
	tauriGetPreferences,
	tauriGetPreferencesPath,
	tauriGetProfile,
	tauriGetProfilePath,
	tauriGetSessionMessages,
	tauriGetStyle,
	tauriGetStylePath,
	tauriGetTelegramPath,
	tauriGetToolsPath,
	tauriListSessions,
	tauriListTools,
	tauriMapSummary,
	tauriPinContext,
	tauriPreviewContextForMessage,
	tauriSaveContexts,
	tauriSavePreferences,
	tauriSaveProfile,
	tauriSaveStyle,
	tauriUnpinContext,
} from "./tauriClient";

const profile = {
	schema_version: 1,
	identity: {
		display_name: "OpenNivara User",
		full_name: "OpenNivara User",
		gender: "",
		pronouns: "",
		date_of_birth: "",
		timezone: "UTC",
	},
	location: {
		country: "US",
		state_or_region: "CA",
		city: "SF",
		living_situation: "",
	},
	languages: {
		preferred_human_language: "English",
		other_human_languages: ["Spanish"],
	},
	technical: {
		coding_level: "advanced",
		preferred_coding_languages: ["Rust"],
		current_os: "Windows",
		main_editor: "VS Code",
		secondary_editor: "",
		terminal: "PowerShell",
	},
	personal: {
		occupation_or_role: "Engineer",
		education_level: "",
		interests: [],
	},
	privacy: {
		send_identity: true,
		send_location: false,
		send_gender: false,
		send_technical: true,
		send_personal: false,
	},
};

const style = {
	schema_version: 1,
	communication: {
		tone: "Helpful",
		detail_level: "Detailed",
		use_examples: true,
		use_step_by_step: true,
		avoid_unexplained_jargon: true,
		ask_fewer_questions: false,
		prefer_actionable_answers: true,
	},
	coding: {
		show_simple_solution_first: true,
		explain_after_code: true,
		prefer_mvp_architecture: true,
		avoid_overengineering: true,
		use_beginner_comments: false,
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
		do_not_reveal_private_context_unless_relevant: true,
	},
};

const preferences = {
	schema_version: 1,
	sections: [
		{
			id: "coding",
			enabled: true,
			send_policy: "always",
			description: "Coding prefs",
			triggers: ["rust"],
			required_any: [],
			negative_triggers: [],
			min_score: 0,
			likes: [{ item: "examples", strength: 4 }],
			dislikes: [],
			notes: ["keep it short"],
		},
	],
};

const contexts = {
	schema_version: 1,
	contexts: [
		{
			id: "project",
			enabled: true,
			kind: "goal",
			send_policy: "always",
			title: "Project",
			summary: "Project summary",
			triggers: ["opennivara"],
			required_any: [],
			negative_triggers: [],
			min_score: 0,
			facts: ["fact"],
			rules: ["rule"],
		},
	],
};

describe("tauriClient command contracts", () => {
	beforeEach(() => {
		clearTauriMocks();
	});

	test("chat, sessions, tools, and config paths call their backend commands", async () => {
		mockTauriCommand("ask_opennivara", (args: any) => ({
			session_id: args.sessionId ?? "new_session",
			answer: `answer:${args.message}`,
		}));
		mockTauriCommand("list_sessions", [
			{
				id: "s1",
				title: "Session",
				created_at: "2026-06-01T00:00:00Z",
				updated_at: "2026-06-01T00:00:00Z",
				status: "active",
				source_created: "desktop",
				active: true,
			},
		]);
		mockTauriCommand("get_session_messages", (args: any) => [
			{
				id: "m1",
				session_id: args.sessionId,
				role: "user",
				source: "desktop",
				content: "hello",
				created_at: "2026-06-01T00:00:00Z",
				metadata_json: null,
			},
		]);
		mockTauriCommand("list_tools", {
			general: {
				enabled: true,
				max_tool_rounds: 3,
				show_tool_activity: true,
			},
			paths: { allowed_roots: ["D:\\repo"], blocked_patterns: [] },
			tools: {},
		});
		mockTauriCommand("get_tools_path", "tools.toml");
		mockTauriCommand("get_profile_path", "profile.toml");
		mockTauriCommand("get_preferences_path", "preferences.toml");
		mockTauriCommand("get_style_path", "style.toml");
		mockTauriCommand("get_contexts_path", "contexts.toml");
		mockTauriCommand("get_telegram_path", "telegram.toml");
		mockTauriCommand("map_summary", "workspace summary");
		mockTauriCommand("check_api_key", true);

		expect(await tauriAskOpenNivara("hello", "s1")).toEqual({
			session_id: "s1",
			answer: "answer:hello",
		});
		expect(await tauriListSessions()).toHaveLength(1);
		expect(await tauriGetSessionMessages("s1")).toHaveLength(1);
		expect(await tauriListTools()).toMatchObject({
			general: { enabled: true },
		});
		expect(await tauriGetToolsPath()).toBe("tools.toml");
		expect(await tauriGetProfilePath()).toBe("profile.toml");
		expect(await tauriGetPreferencesPath()).toBe("preferences.toml");
		expect(await tauriGetStylePath()).toBe("style.toml");
		expect(await tauriGetContextsPath()).toBe("contexts.toml");
		expect(await tauriGetTelegramPath()).toBe("telegram.toml");
		expect(await tauriMapSummary()).toBe("workspace summary");
		expect(await tauriCheckApiKey()).toBe(true);
	});

	test("profile, style, preferences, contexts, preview, and pin commands round-trip payloads", async () => {
		const saved: Record<string, any> = {};
		mockTauriCommand("get_profile", profile);
		mockTauriCommand("save_profile", (args: any) => {
			saved.profile = args.profile;
		});
		mockTauriCommand("get_style", style);
		mockTauriCommand("save_style", (args: any) => {
			saved.style = args.style;
		});
		mockTauriCommand("get_preferences", preferences);
		mockTauriCommand("save_preferences", (args: any) => {
			saved.preferences = args.preferences;
		});
		mockTauriCommand("get_contexts", contexts);
		mockTauriCommand("save_contexts", (args: any) => {
			saved.contexts = args.contexts;
		});
		mockTauriCommand("preview_context_for_message", (args: any) => ({
			profile_sent: [],
			style_sent: [],
			preferences_sent: [args.message],
			contexts_sent: [],
			contexts_pinned: [],
			contexts_not_sent: [],
			warnings: [],
			final_context_text: "prompt",
			active_theme: null,
		}));
		mockTauriCommand("pin_context", (args: any) => {
			saved.pin = args;
		});
		mockTauriCommand("unpin_context", (args: any) => {
			saved.unpin = args;
		});

		expect(await tauriGetProfile()).toEqual(profile);
		await tauriSaveProfile(profile);
		expect(saved.profile).toEqual(profile);

		expect(await tauriGetStyle()).toEqual(style);
		await tauriSaveStyle(style);
		expect(saved.style).toEqual(style);

		expect(await tauriGetPreferences()).toEqual(preferences);
		await tauriSavePreferences(preferences);
		expect(saved.preferences).toEqual(preferences);

		expect(await tauriGetContexts()).toEqual(contexts);
		await tauriSaveContexts(contexts);
		expect(saved.contexts).toEqual(contexts);

		const preview = await tauriPreviewContextForMessage("hello", "s1");
		expect(preview.preferences_sent).toEqual(["hello"]);

		await tauriPinContext("s1", "project");
		await tauriUnpinContext("s1", "project");
		expect(saved.pin).toEqual({ sessionId: "s1", contextId: "project" });
		expect(saved.unpin).toEqual({ sessionId: "s1", contextId: "project" });
	});

	test("invalid style shape fails with a clear schema error", async () => {
		mockTauriCommand("get_style", {
			schema_version: 1,
			communication: { tone: 42 },
		});

		await expect(tauriGetStyle()).rejects.toThrow(
			"Style config shape does not match frontend schema.",
		);
	});
});
