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

const fullProfile = {
	schema_version: 2,
	identity: {
		display_name: "Profile Testing",
		full_name: "Testing full name",
		gender: "Male",
		pronouns: "he/him",
		date_of_birth: "",
		timezone: "UTC",
	},
	location: {
		country: "United States",
		state_or_region: "California",
		city: "SF",
		living_situation: "",
	},
	languages: {
		preferred_human_language: "English",
		other_human_languages: [],
	},
	technical: {
		coding_level: "expert",
		preferred_coding_languages: [],
		current_os: "Windows",
		main_editor: "VS Code",
		secondary_editor: "Antigravity",
		terminal: "",
	},
	personal: {
		occupation_or_role: "",
		education_level: "",
		interests: [],
	},
	privacy: {
		send_identity: true,
		send_location: true,
		send_gender: true,
		send_technical: true,
		send_personal: true,
	},
};

describe("ProfileEditor Tests", () => {
	beforeEach(() => {
		clearTauriMocks();
		mockTauriCommand("get_profile", fullProfile);
		mockTauriCommand("get_style", {
			schema_version: 1,
			communication: {},
			coding: {},
			formatting: {},
			behavior: {},
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

	test("Verify Profile inputs render", async () => {
		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} defaultTab="profile" />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByPlaceholderText("e.g. Alice")).toHaveValue(
				"Profile Testing",
			);
			expect(screen.getByPlaceholderText("e.g. Alice Smith")).toHaveValue(
				"Testing full name",
			);
		});
	});

	test("Editing profile fields and privacy toggles saves the full payload", async () => {
		let savedProfile: any = null;
		mockTauriCommand("save_profile", (args: any) => {
			savedProfile = args.profile;
			return null;
		});

		render(
			<ThemeProvider>
				<SettingsView paths={mockPaths} defaultTab="profile" />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByPlaceholderText("e.g. Alice")).toHaveValue(
				"Profile Testing",
			);
		});

		fireEvent.change(screen.getByPlaceholderText("e.g. Alice"), {
			target: { value: "Edited Display" },
		});
		fireEvent.change(screen.getByPlaceholderText("e.g. Alice Smith"), {
			target: { value: "Edited Full Name" },
		});
		fireEvent.change(screen.getByPlaceholderText("e.g. she/her"), {
			target: { value: "they/them" },
		});
		fireEvent.change(screen.getByPlaceholderText("e.g. CA"), {
			target: { value: "WA" },
		});
		fireEvent.change(screen.getByPlaceholderText("e.g. Rust, TypeScript"), {
			target: { value: "Rust, TypeScript, Python" },
		});

		const identityToggle = screen
			.getByText("Include Basic Identity Info")
			.closest("label")
			?.querySelector("input");
		expect(identityToggle).toBeTruthy();
		fireEvent.click(identityToggle as HTMLInputElement);

		fireEvent.click(
			screen.getByRole("button", { name: /Save Identity Profile/i }),
		);

		await waitFor(() => {
			expect(savedProfile.identity.display_name).toBe("Edited Display");
			expect(savedProfile.identity.full_name).toBe("Edited Full Name");
			expect(savedProfile.identity.pronouns).toBe("they/them");
			expect(savedProfile.location.state_or_region).toBe("WA");
			expect(savedProfile.technical.preferred_coding_languages).toEqual([
				"Rust",
				"TypeScript",
				"Python",
			]);
			expect(savedProfile.privacy.send_identity).toBe(false);
		});
	});
});
