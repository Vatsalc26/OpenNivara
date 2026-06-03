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

describe("PreferencesEditor Tests", () => {
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
		mockTauriCommand("get_preferences", fullPreferences);
		mockTauriCommand("get_contexts", { schema_version: 1, contexts: [] });
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

	test("Verify all Preference section components are rendered & tabs function", async () => {
		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} defaultTab="preferences" />
			</ThemeProvider>,
		);

		// Switch to Preferences view
		await waitFor(() => {
			expect(screen.getByText("Base Topic Preferences")).toBeInTheDocument();
		});

		// Check Basic tab fields (default active tab)
		expect(screen.getByText("Send Policy")).toBeInTheDocument();

		// Click Trigger Rules tab
		const triggerRulesBtn = screen.getByRole("button", {
			name: "Trigger Rules",
		});
		fireEvent.click(triggerRulesBtn);
		await waitFor(() => {
			expect(
				screen.getByText("Keyword Triggers (comma-separated)"),
			).toBeInTheDocument();
			expect(
				screen.getByText("Required Any (comma-separated)"),
			).toBeInTheDocument();
			expect(
				screen.getByText("Negative Triggers (comma-separated)"),
			).toBeInTheDocument();
			expect(screen.getByText("Minimum Trigger Score")).toBeInTheDocument();
		});

		// Click Likes / Strengths tab
		const likesBtn = screen.getByRole("button", { name: "Likes / Strengths" });
		fireEvent.click(likesBtn);
		await waitFor(() => {
			expect(
				screen.getByText("Preferences Bullet Guidelines (Likes)"),
			).toBeInTheDocument();
			expect(
				screen.getByDisplayValue("Step by step fixes"),
			).toBeInTheDocument();
			expect(screen.getByDisplayValue("5")).toBeInTheDocument();
		});

		// Click Dislikes / Strengths tab
		const dislikesBtn = screen.getByRole("button", {
			name: "Dislikes / Strengths",
		});
		fireEvent.click(dislikesBtn);
		await waitFor(() => {
			expect(
				screen.getByText("Preferences Bullet Guidelines (Dislikes)"),
			).toBeInTheDocument();
			expect(screen.getByDisplayValue("Boilerplate code")).toBeInTheDocument();
			expect(screen.getByDisplayValue("4")).toBeInTheDocument();
		});

		// Click Notes tab
		const notesBtn = screen.getByRole("button", { name: "Notes" });
		fireEvent.click(notesBtn);
		await waitFor(() => {
			expect(screen.getByText("Guidance Notes")).toBeInTheDocument();
			expect(
				screen.getByDisplayValue("Only use when coding"),
			).toBeInTheDocument();
		});
	});

	test("Editing preference selector fields, strengths, dislikes, and notes saves without dropping fields", async () => {
		let savedPreferences: any = null;
		mockTauriCommand("save_preferences", (args: any) => {
			savedPreferences = args.preferences;
			return null;
		});

		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} defaultTab="preferences" />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("Base Topic Preferences")).toBeInTheDocument();
		});

		fireEvent.change(
			screen.getByDisplayValue("Coding preferences section desc"),
			{
				target: { value: "Edited coding preferences" },
			},
		);
		fireEvent.change(screen.getAllByRole("combobox")[0], {
			target: { value: "always" },
		});

		fireEvent.click(screen.getByRole("button", { name: "Trigger Rules" }));
		fireEvent.change(screen.getByDisplayValue("rust, typescript"), {
			target: { value: "rust, tauri" },
		});
		fireEvent.change(screen.getByDisplayValue("build, run"), {
			target: { value: "debug, compile" },
		});
		fireEvent.change(screen.getByDisplayValue("node"), {
			target: { value: "movie, food" },
		});
		fireEvent.change(screen.getByDisplayValue("2"), {
			target: { value: "4" },
		});

		fireEvent.click(screen.getByRole("button", { name: "Likes / Strengths" }));
		fireEvent.change(screen.getByDisplayValue("Step by step fixes"), {
			target: { value: "Explain the exact fix" },
		});
		fireEvent.change(screen.getByDisplayValue("5"), {
			target: { value: "2" },
		});
		fireEvent.click(screen.getByRole("button", { name: /Add Bullet/i }));
		expect(
			screen.getByDisplayValue("New liked guideline item"),
		).toBeInTheDocument();

		fireEvent.click(
			screen.getByRole("button", { name: "Dislikes / Strengths" }),
		);
		fireEvent.change(screen.getByDisplayValue("Boilerplate code"), {
			target: { value: "Avoid boilerplate" },
		});
		fireEvent.change(screen.getByDisplayValue("4"), {
			target: { value: "1" },
		});
		fireEvent.click(screen.getByRole("button", { name: /Add Bullet/i }));
		expect(
			screen.getByDisplayValue("New disliked guideline item"),
		).toBeInTheDocument();

		fireEvent.click(screen.getByRole("button", { name: "Notes" }));
		fireEvent.change(screen.getByDisplayValue("Only use when coding"), {
			target: { value: "Only use for code work" },
		});
		fireEvent.click(screen.getByRole("button", { name: /Add Note/i }));
		expect(
			screen.getByDisplayValue("New guidance note text"),
		).toBeInTheDocument();

		fireEvent.click(screen.getByRole("button", { name: /Save Preferences/i }));

		await waitFor(() => {
			const section = savedPreferences.sections[0];
			expect(section.description).toBe("Edited coding preferences");
			expect(section.send_policy).toBe("always");
			expect(section.triggers).toEqual(["rust", "tauri"]);
			expect(section.required_any).toEqual(["debug", "compile"]);
			expect(section.negative_triggers).toEqual(["movie", "food"]);
			expect(section.min_score).toBe(4);
			expect(section.likes[0]).toEqual({
				item: "Explain the exact fix",
				strength: 2,
			});
			expect(section.dislikes[0]).toEqual({
				item: "Avoid boilerplate",
				strength: 1,
			});
			expect(section.notes[0]).toBe("Only use for code work");
			expect(section.likes[1].item).toBe("New liked guideline item");
			expect(section.dislikes[1].item).toBe("New disliked guideline item");
			expect(section.notes[1]).toBe("New guidance note text");
		});
	});
});
