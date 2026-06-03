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

describe("ResponseStyleEditor Tests", () => {
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
		});
		mockTauriCommand("get_preferences", { schema_version: 1, sections: [] });
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

	test("Verify all style segments rendering", async () => {
		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} defaultTab="style" />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(
				screen.getByText("Communication Style Guidelines"),
			).toBeInTheDocument();
			expect(screen.getByText("Coding Output Guidance")).toBeInTheDocument();
			expect(screen.getByText("Formatting & Layout")).toBeInTheDocument();
			expect(
				screen.getByText("Behavior & Integrity Constraints"),
			).toBeInTheDocument();
		});
	});

	test("Editing style tone, detail level, and toggles saves complete style", async () => {
		let savedStyle: any = null;
		mockTauriCommand("save_style", (args: any) => {
			savedStyle = args.style;
			return null;
		});

		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} defaultTab="style" />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(
				screen.getByPlaceholderText("e.g. clear, direct, beginner-friendly"),
			).toHaveValue("Professional");
		});

		fireEvent.change(
			screen.getByPlaceholderText("e.g. clear, direct, beginner-friendly"),
			{ target: { value: "Direct and kind" } },
		);
		fireEvent.change(screen.getAllByRole("combobox")[0], {
			target: { value: "high" },
		});

		for (const label of [
			"Include Usage Examples",
			"Show Simple Solution First",
			"Use Markdown",
			"Honest About Uncertainty",
		]) {
			const checkbox = screen
				.getByText(label)
				.closest("label")
				?.querySelector("input");
			expect(checkbox).toBeTruthy();
			fireEvent.click(checkbox as HTMLInputElement);
		}

		fireEvent.click(
			screen.getByRole("button", { name: /Save Style Guidelines/i }),
		);

		await waitFor(() => {
			expect(savedStyle.communication.tone).toBe("Direct and kind");
			expect(savedStyle.communication.detail_level).toBe("high");
			expect(savedStyle.communication.use_examples).toBe(false);
			expect(savedStyle.coding.show_simple_solution_first).toBe(false);
			expect(savedStyle.formatting.use_markdown).toBe(false);
			expect(savedStyle.behavior.be_honest_about_uncertainty).toBe(false);
		});
	});
});
