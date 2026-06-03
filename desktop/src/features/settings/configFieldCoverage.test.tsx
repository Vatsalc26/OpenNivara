import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test } from "vitest";
import { clearTauriMocks, mockTauriCommand } from "../../test/mockTauri";
import { ThemeProvider } from "../../theme/ThemeProvider";
import { SettingsView } from "./SettingsView";

const mockPaths = {
	profile: "C:\\mock\\profile.toml",
	preferences: "C:\\mock\\preferences.toml",
	style: "C:\\mock\\style.toml",
	tools: "C:\\mock\\tools.toml",
	contexts: "C:\\mock\\contexts.toml",
};

const fullProfile = {
	schema_version: 2,
	identity: {
		display_name: "John Display",
		full_name: "John Full Name",
		gender: "Non-binary",
		pronouns: "they/them",
		date_of_birth: "1990-01-01",
		timezone: "UTC",
	},
	location: {
		country: "United States",
		state_or_region: "California",
		city: "San Francisco",
		living_situation: "Apartment",
	},
	languages: {
		preferred_human_language: "English",
		other_human_languages: ["Spanish"],
	},
	technical: {
		coding_level: "expert",
		preferred_coding_languages: ["Rust", "TypeScript"],
		current_os: "Windows",
		main_editor: "VS Code",
		secondary_editor: "Antigravity",
		terminal: "PowerShell",
	},
	personal: {
		occupation_or_role: "Software Engineer",
		education_level: "Bachelor",
		interests: ["Coding", "Hiking"],
	},
	privacy: {
		send_identity: true,
		send_location: true,
		send_gender: true,
		send_technical: true,
		send_personal: true,
	},
};

const fullStyle = {
	schema_version: 1,
	communication: {
		tone: "Professional",
		detail_level: "Detailed",
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
		do_not_reveal_private_context_unless_relevant: true,
	},
};

const fullPreferences = {
	schema_version: 2,
	sections: [
		{
			id: "coding_pref",
			enabled: true,
			send_policy: "triggered_strict",
			description: "Coding preferences section desc",
			triggers: ["rust", "typescript"],
			required_any: ["build", "run"],
			negative_triggers: ["node"],
			min_score: 2,
			likes: [{ item: "Step by step fixes", strength: 5 }],
			dislikes: [{ item: "Boilerplate code", strength: 4 }],
			notes: ["Only use when coding"],
		},
	],
};

const fullContexts = {
	schema_version: 1,
	contexts: [
		{
			id: "opennivara_project",
			enabled: true,
			kind: "project",
			send_policy: "session_pinned",
			title: "OpenNivara Project goals",
			summary: "unified Rust CLI and Tauri desktop UI context description",
			triggers: ["opennivara"],
			required_any: ["tauri"],
			negative_triggers: ["movie"],
			min_score: 3,
			facts: ["Rust project facts", "Tauri application framework"],
			rules: ["Use when discussing OpenNivara"],
		},
	],
};

describe("Configuration Field Coverage Tests", () => {
	beforeEach(() => {
		clearTauriMocks();
		mockTauriCommand("get_profile", fullProfile);
		mockTauriCommand("get_style", fullStyle);
		mockTauriCommand("get_preferences", fullPreferences);
		mockTauriCommand("get_contexts", fullContexts);
		mockTauriCommand("marketplace_get_addon_settings", {
			schema_version: 1,
			active_theme_id: "coding_cyan",
			active_theme_source_pack_id: "coding_basics",
			enabled_packs: ["coding_basics"],
			disabled_contributions: [],
		});
		mockTauriCommand("marketplace_get_active_addon_theme", {
			schema_version: 1,
			id: "coding_cyan",
			colors: { background: "#000", foreground: "#fff" },
		});
		mockTauriCommand("marketplace_get_effective_settings_preview", {
			base_preferences: [],
			addon_preferences: [],
			base_contexts: [],
			addon_contexts: [],
			addon_quick_prompts: [],
			active_theme_id: "coding_cyan",
			active_theme_name: "Coding Cyan",
			active_theme_source_pack_id: "coding_basics",
			disabled_contributions: [],
			enabled_packs: ["coding_basics"],
		});
		mockTauriCommand("marketplace_list_installed_packs", { installed: [] });
		mockTauriCommand("marketplace_has_legacy_modes", false);
		mockTauriCommand("marketplace_list_installed_themes", []);
	});

	test("Assert Profile editor field visibility", async () => {
		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} defaultTab="profile" />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByPlaceholderText("e.g. Alice")).toHaveValue(
				"John Display",
			);
			expect(screen.getByPlaceholderText("e.g. Alice Smith")).toHaveValue(
				"John Full Name",
			);
			expect(screen.getByPlaceholderText("e.g. Female")).toHaveValue(
				"Non-binary",
			);
			expect(screen.getByPlaceholderText("e.g. she/her")).toHaveValue(
				"they/them",
			);
			expect(screen.getByPlaceholderText("English")).toHaveValue("English");
			expect(screen.getByPlaceholderText("e.g. VS Code")).toHaveValue(
				"VS Code",
			);
			expect(screen.getByPlaceholderText("e.g. Windows")).toHaveValue(
				"Windows",
			);
		});
	});

	test("Assert Style editor field visibility", async () => {
		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} defaultTab="style" />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(
				screen.getByText("Communication Style Guidelines"),
			).toBeInTheDocument();
			expect(screen.getByText("Formatting & Layout")).toBeInTheDocument();
		});
	});
});
