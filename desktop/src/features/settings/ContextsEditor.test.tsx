import { fireEvent, render, screen, waitFor } from "@testing-library/react";
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

describe("ContextsEditor Tests", () => {
	beforeEach(() => {
		clearTauriMocks();
		mockTauriCommand("get_profile", {
			schema_version: 1,
			identity: { display_name: "John" },
			location: {},
			languages: {},
			technical: { coding_level: "beginner", preferred_coding_languages: [] },
			personal: {},
			privacy: {},
		});
		mockTauriCommand("get_style", {
			schema_version: 1,
			communication: {},
			coding: {},
			formatting: {},
			behavior: {},
		});
		mockTauriCommand("get_preferences", { schema_version: 1, sections: [] });
		mockTauriCommand("get_contexts", fullContexts);
		mockTauriCommand("marketplace_get_addon_settings", {
			schema_version: 1,
			active_theme_id: null,
			active_theme_source_pack_id: null,
			enabled_packs: [],
			disabled_contributions: [],
		});
		mockTauriCommand("marketplace_get_effective_settings_preview", {
			base_preferences: [],
			addon_preferences: [],
			base_contexts: [],
			addon_contexts: [],
			addon_quick_prompts: [],
			active_theme_id: null,
			active_theme_name: null,
			active_theme_source_pack_id: null,
			disabled_contributions: [],
			enabled_packs: [],
		});
		mockTauriCommand("marketplace_list_installed_packs", { installed: [] });
		mockTauriCommand("marketplace_has_legacy_modes", false);
		mockTauriCommand("marketplace_list_installed_themes", []);
	});

	test("Verify all Goal Context rendering & sub-accordions navigate", async () => {
		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} defaultTab="contexts" />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("Base Project Goals")).toBeInTheDocument();
		});

		// Check Basic tab is active by default
		expect(screen.getByText("Summary descriptor")).toBeInTheDocument();
		expect(
			screen.getByDisplayValue(
				"unified Rust CLI and Tauri desktop UI context description",
			),
		).toBeInTheDocument();

		// Click Trigger Rules
		const triggerRulesBtn = screen.getByRole("button", {
			name: "Trigger Rules",
		});
		fireEvent.click(triggerRulesBtn);
		await waitFor(() => {
			expect(
				screen.getByText("Keyword Triggers (comma-separated)"),
			).toBeInTheDocument();
			expect(screen.getByDisplayValue("opennivara")).toBeInTheDocument();
			expect(screen.getByDisplayValue("tauri")).toBeInTheDocument();
		});

		// Click Facts List
		const factsBtn = screen.getByRole("button", { name: "Facts List" });
		fireEvent.click(factsBtn);
		await waitFor(() => {
			expect(screen.getByText("Workspace Context Facts")).toBeInTheDocument();
			expect(
				screen.getByDisplayValue("Rust project facts"),
			).toBeInTheDocument();
		});

		// Click Rules List
		const rulesBtn = screen.getByRole("button", { name: "Rules List" });
		fireEvent.click(rulesBtn);
		await waitFor(() => {
			expect(screen.getByText("Context Rule Constraints")).toBeInTheDocument();
			expect(
				screen.getByDisplayValue("Use when discussing OpenNivara"),
			).toBeInTheDocument();
		});
	});

	test("Editing context fields, selector rules, facts, and rules saves full context", async () => {
		let savedContexts: any = null;
		mockTauriCommand("save_contexts", (args: any) => {
			savedContexts = args.contexts;
			return null;
		});

		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} defaultTab="contexts" />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("Base Project Goals")).toBeInTheDocument();
		});

		fireEvent.change(screen.getByDisplayValue("OpenNivara Project goals"), {
			target: { value: "Edited OpenNivara goals" },
		});
		fireEvent.change(
			screen.getByDisplayValue(
				"unified Rust CLI and Tauri desktop UI context description",
			),
			{ target: { value: "Edited context summary" } },
		);
		fireEvent.change(screen.getAllByRole("combobox")[0], {
			target: { value: "goal" },
		});
		fireEvent.change(screen.getAllByRole("combobox")[1], {
			target: { value: "always" },
		});

		fireEvent.click(screen.getByRole("button", { name: "Trigger Rules" }));
		fireEvent.change(screen.getByDisplayValue("opennivara"), {
			target: { value: "opennivara, tauri" },
		});
		fireEvent.change(screen.getByDisplayValue("tauri"), {
			target: { value: "desktop, rust" },
		});
		fireEvent.change(screen.getByDisplayValue("movie"), {
			target: { value: "recipe, movie" },
		});
		fireEvent.change(screen.getByDisplayValue("3"), {
			target: { value: "5" },
		});

		fireEvent.click(screen.getByRole("button", { name: "Facts List" }));
		fireEvent.change(screen.getByDisplayValue("Rust project facts"), {
			target: { value: "Edited Rust fact" },
		});
		fireEvent.click(screen.getByRole("button", { name: /Add Fact/i }));
		expect(
			screen.getByDisplayValue("New workspace project fact statement"),
		).toBeInTheDocument();

		fireEvent.click(screen.getByRole("button", { name: "Rules List" }));
		fireEvent.change(
			screen.getByDisplayValue("Use when discussing OpenNivara"),
			{
				target: { value: "Use only for OpenNivara project work" },
			},
		);
		fireEvent.click(screen.getByRole("button", { name: /Add Rule/i }));
		expect(
			screen.getByDisplayValue("New prompt guidance constraint rule"),
		).toBeInTheDocument();

		fireEvent.click(
			screen.getByRole("button", { name: /Save Goal Contexts/i }),
		);

		await waitFor(() => {
			const context = savedContexts.contexts[0];
			expect(context.title).toBe("Edited OpenNivara goals");
			expect(context.summary).toBe("Edited context summary");
			expect(context.kind).toBe("goal");
			expect(context.send_policy).toBe("always");
			expect(context.triggers).toEqual(["opennivara", "tauri"]);
			expect(context.required_any).toEqual(["desktop", "rust"]);
			expect(context.negative_triggers).toEqual(["recipe", "movie"]);
			expect(context.min_score).toBe(5);
			expect(context.facts[0]).toBe("Edited Rust fact");
			expect(context.facts[2]).toBe("New workspace project fact statement");
			expect(context.rules[0]).toBe("Use only for OpenNivara project work");
			expect(context.rules[1]).toBe("New prompt guidance constraint rule");
		});
	});
});
