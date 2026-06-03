import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { themesFixture } from "../../test/fixtures/themes";
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

const mockProfile = {
	schema_version: 1,
	identity: {
		display_name: "John Doe",
		full_name: "John Doe",
		gender: "male",
		pronouns: "he/him",
		date_of_birth: "",
		timezone: "",
	},
	location: {
		country: "US",
		state_or_region: "CA",
		city: "SF",
		living_situation: "",
	},
	languages: { preferred_human_language: "English", other_human_languages: [] },
	technical: {
		coding_level: "Advanced",
		preferred_coding_languages: ["Rust", "TypeScript"],
		current_os: "Windows",
		main_editor: "VS Code",
		secondary_editor: "",
		terminal: "",
	},
	personal: { occupation_or_role: "", education_level: "", interests: [] },
	privacy: {
		send_identity: true,
		send_location: false,
		send_gender: false,
		send_technical: true,
		send_personal: false,
	},
};

const mockStyle = {
	schema_version: 1,
	communication: {
		tone: "helpful",
		detail_level: "medium",
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

const mockPreferences = {
	schema_version: 1,
	sections: [],
};

const mockContexts = {
	schema_version: 1,
	contexts: [],
};

describe("SettingsView Integration Tests", () => {
	beforeEach(() => {
		clearTauriMocks();
		Object.assign(navigator, {
			clipboard: {
				writeText: vi.fn(),
			},
		});
		mockTauriCommand("get_profile", mockProfile);
		mockTauriCommand("get_style", mockStyle);
		mockTauriCommand("get_preferences", mockPreferences);
		mockTauriCommand("get_contexts", mockContexts);
		mockTauriCommand("theme_get_active", {
			schema_version: 1,
			id: "coding_cyan",
			name: "Coding Cyan",
			description: "Sleek neon cyan theme",
			colors: {
				background: "#0f172a",
				foreground: "#f8fafc",
				primary: "#06b6d4",
				accent: "#a78bfa",
				card: "#1e293b",
				panel: "#1e293b",
				muted: "#64748b",
				success: "#10b981",
				warning: "#f59e0b",
				danger: "#ef4444",
			},
			effects: {
				background_gradient: true,
				glow: "medium",
				density: "normal",
			},
		});
		mockTauriCommand("marketplace_has_legacy_modes", false);
		mockTauriCommand("theme_store_list", themesFixture);
		mockTauriCommand("theme_apply", null);
		mockTauriCommand("theme_reset", null);
	});

	test("1. Renders settings category sidebar navigation properly", async () => {
		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getAllByText("User Identity")[0]).toBeInTheDocument();
			expect(screen.getAllByText("Response Style")[0]).toBeInTheDocument();
			expect(screen.getAllByText("Topic Prefs")[0]).toBeInTheDocument();
			expect(screen.getAllByText("Project Goals")[0]).toBeInTheDocument();
			expect(screen.getAllByText("Appearance")[0]).toBeInTheDocument();
			expect(screen.getAllByText("Config Files")[0]).toBeInTheDocument();
			expect(screen.queryByText("Quick Prompts")).not.toBeInTheDocument();
			expect(screen.queryByText("Add-ons")).not.toBeInTheDocument();
		});
	});

	test("2. Response Style renders communication and formatting groups", async () => {
		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} />
			</ThemeProvider>,
		);

		// Switch to Response Style
		await waitFor(() => {
			const button = screen.getByRole("button", { name: /Response Style/i });
			fireEvent.click(button);
		});

		expect(
			screen.getByText("Communication Style Guidelines"),
		).toBeInTheDocument();
		expect(screen.getByText("Coding Output Guidance")).toBeInTheDocument();
		expect(
			screen.getByText("Behavior & Integrity Constraints"),
		).toBeInTheDocument();
		// Formatting & Layout should be displayed
		expect(screen.getByText("Formatting & Layout")).toBeInTheDocument();
		expect(screen.getByText("Use Markdown")).toBeInTheDocument();
		expect(screen.getByText("Use Short Sections")).toBeInTheDocument();
		expect(screen.getByText("Include Next Step")).toBeInTheDocument();
		expect(screen.getByText("Avoid Long Walls of Text")).toBeInTheDocument();
	});

	test("3. Appearance renders themes from Dynamic backend data", async () => {
		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} />
			</ThemeProvider>,
		);

		await waitFor(() => {
			const button = screen.getByRole("button", { name: /Appearance/i });
			fireEvent.click(button);
		});

		// Check for Coding Cyan from fixture
		expect(screen.getAllByText("Coding Cyan")[0]).toBeInTheDocument();
		// Check for Calm Focus from fixture
		expect(screen.getByText("Calm Focus")).toBeInTheDocument();
		expect(
			screen.getByText("Relaxing deep forest-green tones for study sessions."),
		).toBeInTheDocument();
	});

	test("4. Applying a theme calls theme_apply and updates Provider", async () => {
		let calledThemeId: string | null = null;
		mockTauriCommand("theme_apply", (args: any) => {
			calledThemeId = args.themeId;
			return null;
		});

		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} />
			</ThemeProvider>,
		);

		await waitFor(() => {
			const button = screen.getByRole("button", { name: /Appearance/i });
			fireEvent.click(button);
		});

		// Find the apply button for Coding Cyan
		const applyButton = screen.getAllByRole("button", {
			name: /Apply Theme/i,
		})[0];
		fireEvent.click(applyButton);

		await waitFor(() => {
			expect(calledThemeId).toBe("coding_cyan");
		});
	});

	test("5. Topic preferences do not render Store add-on contributions", async () => {
		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} />
			</ThemeProvider>,
		);

		await waitFor(() => {
			const button = screen.getByRole("button", { name: /Topic Prefs/i });
			fireEvent.click(button);
		});

		expect(screen.getByText("Base Topic Preferences")).toBeInTheDocument();
		expect(screen.queryByText("Coding Basics Pack")).not.toBeInTheDocument();
		expect(
			screen.queryByText("Prefer simple MVP architecture"),
		).not.toBeInTheDocument();
	});

	test("6. All settings category branches render populated data and local actions", async () => {
		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("System Config Hub")).toBeInTheDocument();
		});

		fireEvent.click(screen.getByRole("button", { name: /Appearance/i }));
		await waitFor(() => {
			expect(screen.getByText("Current Theme")).toBeInTheDocument();
			expect(screen.getByText("Theme Preview")).toBeInTheDocument();
		});

		expect(screen.queryByRole("button", { name: /Quick Prompts/i })).toBeNull();
		expect(screen.queryByRole("button", { name: /Add-ons/i })).toBeNull();

		fireEvent.click(screen.getByRole("button", { name: /Config Files/i }));
		await waitFor(() => {
			expect(
				screen.getByText("User Profile Configuration"),
			).toBeInTheDocument();
			expect(
				screen.getByText("Appearance Theme Configuration"),
			).toBeInTheDocument();
		});
		fireEvent.click(screen.getAllByRole("button", { name: /Copy Path/i })[0]);
		expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
			mockPaths.profile,
		);
	});
});
