import { beforeEach, describe, expect, test } from "vitest";
import { builtinPacksFixture } from "../test/fixtures/builtinPacks";
import { effectiveSettingsPreviewFixture } from "../test/fixtures/effectiveSettingsPreview";
import { installedPacksFixture } from "../test/fixtures/installedPacks";
import { packPreviewsFixture } from "../test/fixtures/packPreviews";
import { themesFixture } from "../test/fixtures/themes";
import { clearTauriMocks, mockTauriCommand } from "../test/mockTauri";
import {
	addPackToMode,
	addPackToModeWithActivation,
	applyTheme,
	createMode,
	createModeFromPack,
	getActiveCommandSnippets,
	getActiveTheme,
	getAddonSettings,
	getAppearanceSettings,
	getEffectiveSettingsPreview,
	getMarketplaceStatus,
	getModes,
	getPackActivationCapabilities,
	hasLegacyModes,
	initMarketplace,
	installBuiltinPack,
	installBuiltinTheme,
	installPack,
	installTheme,
	listBuiltinPacks,
	listInstalledPacks,
	listInstalledThemeRecords,
	listInstalledThemes,
	listThemeStoreItems,
	migrateAddons,
	previewBuiltinPack,
	previewInstalledPack,
	previewPack,
	previewTheme,
	removePackFromMode,
	repairMarketplace,
	resetMarketplace,
	resetTheme,
	saveAddonSettings,
	setActiveMode,
	setActiveTheme,
	setPackEnabled,
	toggleContributionEnabled,
	togglePackEnabled,
	uninstallPack,
	uninstallTheme,
	updateModeStylePack,
	updateModeTheme,
} from "./marketplaceClient";

const installedPack = installedPacksFixture.installed[0];
const modes = {
	schema_version: 1,
	active_mode: "coding",
	modes: [
		{
			id: "coding",
			name: "Coding",
			description: "Coding mode",
			enabled_pack_ids: ["coding_basics"],
			theme_id: null,
			style_pack_id: null,
		},
	],
};
const theme = {
	schema_version: 1,
	id: "coding_cyan",
	name: "Coding Cyan",
	description: "Theme",
	colors: {
		background: "#000",
		panel: "#111",
		card: "#222",
		primary: "#0ff",
		accent: "#f0f",
		success: "#0f0",
		warning: "#ff0",
		danger: "#f00",
		foreground: "#fff",
		muted: "#888",
	},
	effects: {
		background_gradient: true,
		glow: "medium",
		density: "normal",
	},
};
const status = {
	marketplace_dir: "D:\\store",
	installed_count: 1,
	enabled_count: 1,
	disabled_count: 0,
	modes_count: 1,
	active_mode_id: "coding",
	active_mode_name: "Coding",
	active_theme_id: "coding_cyan",
	active_theme_name: "Coding Cyan",
	missing_pack_ids: [],
	disabled_packs_in_active_mode: [],
	builtin_packs_available: ["coding_basics"],
	builtin_resource_path_checked: "D:\\packs",
	builtin_resource_path_exists: true,
};
const addonSettings = {
	schema_version: 1,
	active_theme_id: "coding_cyan",
	active_theme_source_pack_id: "coding_basics",
	enabled_packs: ["coding_basics"],
	disabled_contributions: [],
};
const installedTheme = {
	id: "coding_cyan",
	name: "Coding Cyan",
	version: "1.0.0",
	source_kind: "builtin",
	installed_at: "2026-01-01T00:00:00Z",
	manifest_path: "D:\\themes\\coding_cyan\\theme.toml",
};
const themeSafety = {
	data_only: true,
	contains_executable_code: false,
	modifies_tool_security: false,
	requires_network: false,
};
const legacyInstalledThemeSummaries = [
	{
		theme_id: "coding_cyan",
		theme_name: "Coding Cyan",
		description: "Sleek high-contrast neon theme for developer focus.",
		source_pack_id: "coding_basics",
		source_pack_name: "Coding Basics Pack",
		pack_enabled: true,
	},
];

describe("marketplaceClient command contracts", () => {
	beforeEach(() => {
		clearTauriMocks();
	});

	test("read commands parse backend responses", async () => {
		mockTauriCommand("marketplace_init", "initialized");
		mockTauriCommand("marketplace_list_installed_packs", installedPacksFixture);
		mockTauriCommand("marketplace_list_builtin_packs", builtinPacksFixture);
		mockTauriCommand(
			"marketplace_preview_pack",
			packPreviewsFixture.coding_basics,
		);
		mockTauriCommand(
			"marketplace_preview_builtin_pack",
			packPreviewsFixture.study_coach,
		);
		mockTauriCommand(
			"marketplace_preview_installed_pack",
			packPreviewsFixture.coding_basics,
		);
		mockTauriCommand("marketplace_get_modes", modes);
		mockTauriCommand("theme_get_active", theme);
		mockTauriCommand("theme_store_list", themesFixture);
		mockTauriCommand("theme_list_installed", [installedTheme]);
		mockTauriCommand("theme_get_appearance_settings", {
			schema_version: 1,
			active_theme_id: "coding_cyan",
			active_theme_source: "installed",
		});
		mockTauriCommand("theme_preview", {
			manifest: {
				id: "coding_cyan",
				name: "Coding Cyan",
				description: "Theme",
				author: "Vatsal Chavda",
				version: "1.0.0",
				source_kind: "builtin",
				safety: themeSafety,
			},
			theme,
			installed: true,
			applied: false,
		});
		mockTauriCommand("marketplace_get_active_addon_theme", theme);
		mockTauriCommand(
			"marketplace_list_installed_themes",
			legacyInstalledThemeSummaries,
		);
		mockTauriCommand("marketplace_get_active_command_snippets", [
			{
				id: "explain",
				title: "Explain",
				description: "Explain code",
				category: "Coding",
				prompt: "Explain this:",
				tags: ["code"],
			},
		]);
		mockTauriCommand("marketplace_repair", {
			repaired: true,
			actions: ["fixed"],
			warnings: [],
			errors: [],
		});
		mockTauriCommand("marketplace_status", status);
		mockTauriCommand("marketplace_get_pack_activation_capabilities", {
			pack_id: "coding_basics",
			has_theme: true,
			theme_id: "coding_cyan",
			theme_name: "Coding Cyan",
			has_style: true,
			has_preferences: true,
			has_contexts: true,
			has_command_snippets: true,
			has_workspace_rules: false,
		});
		mockTauriCommand("marketplace_add_pack_to_mode_with_activation", {
			mode_id: "coding",
			pack_id: "coding_basics",
			added_pack: true,
			applied_theme_id: "coding_cyan",
			applied_style_pack_id: "coding_basics",
			warnings: [],
		});
		mockTauriCommand("marketplace_create_mode_from_pack", modes.modes[0]);
		mockTauriCommand("marketplace_get_addon_settings", addonSettings);
		mockTauriCommand(
			"marketplace_get_effective_settings_preview",
			effectiveSettingsPreviewFixture,
		);
		mockTauriCommand("marketplace_has_legacy_modes", true);

		expect(await initMarketplace()).toBe("initialized");
		expect(await listInstalledPacks()).toEqual(installedPacksFixture);
		expect(await listBuiltinPacks()).toHaveLength(2);
		expect(await previewPack("D:\\pack")).toEqual(
			packPreviewsFixture.coding_basics,
		);
		expect(await previewBuiltinPack("study_coach")).toEqual(
			packPreviewsFixture.study_coach,
		);
		expect(await previewInstalledPack("coding_basics")).toEqual(
			packPreviewsFixture.coding_basics,
		);
		expect(await getModes()).toEqual(modes);
		expect(await getActiveTheme()).toEqual(theme);
		expect(await listThemeStoreItems()).toEqual(themesFixture);
		expect(await listInstalledThemeRecords()).toEqual([installedTheme]);
		expect(await getAppearanceSettings()).toMatchObject({
			active_theme_id: "coding_cyan",
		});
		expect(await previewTheme("coding_cyan")).toMatchObject({
			installed: true,
			applied: false,
		});
		expect(await listInstalledThemes()).toEqual(legacyInstalledThemeSummaries);
		expect(await getActiveCommandSnippets()).toHaveLength(1);
		expect(await repairMarketplace(true)).toMatchObject({ repaired: true });
		expect(await getMarketplaceStatus()).toEqual(status);
		expect(await getPackActivationCapabilities("coding_basics")).toMatchObject({
			has_theme: true,
		});
		expect(
			await addPackToModeWithActivation("coding", "coding_basics", true, true),
		).toMatchObject({ added_pack: true });
		expect(
			await createModeFromPack(
				"coding_basics",
				"coding",
				"Coding",
				true,
				true,
				true,
			),
		).toEqual(modes.modes[0]);
		expect(await getAddonSettings()).toEqual(addonSettings);
		expect(await getEffectiveSettingsPreview()).toEqual(
			effectiveSettingsPreviewFixture,
		);
		expect(await hasLegacyModes()).toBe(true);
	});

	test("write commands send exact payloads", async () => {
		const calls: Record<string, any> = {};
		const capture = (name: string) => (args: any) => {
			calls[name] = args;
			return name.includes("install") ? installedPack : null;
		};
		mockTauriCommand("marketplace_install_pack", capture("installPack"));
		mockTauriCommand(
			"marketplace_install_builtin_pack",
			capture("installBuiltinPack"),
		);
		mockTauriCommand("theme_install_builtin", (args: any) => {
			calls.installBuiltinTheme = args;
			return installedTheme;
		});
		mockTauriCommand("theme_install_from_path", (args: any) => {
			calls.installTheme = args;
			return installedTheme;
		});
		mockTauriCommand("theme_uninstall", capture("uninstallTheme"));
		mockTauriCommand("theme_apply", capture("applyTheme"));
		mockTauriCommand("theme_reset", capture("resetTheme"));
		mockTauriCommand("marketplace_uninstall_pack", capture("uninstallPack"));
		mockTauriCommand("marketplace_set_active_mode", capture("setActiveMode"));
		mockTauriCommand("marketplace_create_mode", capture("createMode"));
		mockTauriCommand("marketplace_add_pack_to_mode", capture("addPackToMode"));
		mockTauriCommand(
			"marketplace_remove_pack_from_mode",
			capture("removePackFromMode"),
		);
		mockTauriCommand("marketplace_enable_pack", capture("enablePack"));
		mockTauriCommand("marketplace_disable_pack", capture("disablePack"));
		mockTauriCommand("marketplace_reset", capture("reset"));
		mockTauriCommand("marketplace_update_mode_theme", capture("updateTheme"));
		mockTauriCommand(
			"marketplace_update_mode_style_pack",
			capture("updateStylePack"),
		);
		mockTauriCommand(
			"marketplace_save_addon_settings",
			capture("saveSettings"),
		);
		mockTauriCommand("marketplace_toggle_pack_enabled", capture("togglePack"));
		mockTauriCommand(
			"marketplace_toggle_contribution_enabled",
			capture("toggleContribution"),
		);
		mockTauriCommand("marketplace_set_active_theme", capture("setActiveTheme"));
		mockTauriCommand("marketplace_migrate_addons", capture("migrate"));

		expect(await installPack("D:\\pack")).toEqual(installedPack);
		expect(await installBuiltinPack("coding_basics")).toEqual(installedPack);
		expect(await installTheme("D:\\theme")).toEqual(installedTheme);
		expect(await installBuiltinTheme("coding_cyan")).toEqual(installedTheme);
		await uninstallTheme("coding_cyan");
		await applyTheme("coding_cyan");
		await resetTheme();
		await uninstallPack("coding_basics");
		await setActiveMode("coding");
		await createMode(modes.modes[0]);
		await addPackToMode("coding", "coding_basics");
		await removePackFromMode("coding", "coding_basics");
		await setPackEnabled("coding_basics", true);
		await setPackEnabled("coding_basics", false);
		await resetMarketplace();
		await updateModeTheme("coding", "coding_cyan");
		await updateModeStylePack("coding", "coding_basics");
		await saveAddonSettings(addonSettings);
		await togglePackEnabled("coding_basics", false);
		await toggleContributionEnabled(
			"coding_basics",
			"preference",
			"mvp",
			false,
		);
		await setActiveTheme("coding_cyan", "coding_basics");
		await migrateAddons();

		expect(calls.installPack).toEqual({ path: "D:\\pack" });
		expect(calls.installBuiltinPack).toEqual({ packId: "coding_basics" });
		expect(calls.installTheme).toEqual({ path: "D:\\theme" });
		expect(calls.installBuiltinTheme).toEqual({ themeId: "coding_cyan" });
		expect(calls.uninstallTheme).toEqual({ themeId: "coding_cyan" });
		expect(calls.applyTheme).toEqual({ themeId: "coding_cyan" });
		expect(calls.resetTheme).toBeUndefined();
		expect(calls.uninstallPack).toEqual({ packId: "coding_basics" });
		expect(calls.setActiveMode).toEqual({ modeId: "coding" });
		expect(calls.createMode).toEqual({ mode: modes.modes[0] });
		expect(calls.addPackToMode).toEqual({
			modeId: "coding",
			packId: "coding_basics",
		});
		expect(calls.removePackFromMode).toEqual({
			modeId: "coding",
			packId: "coding_basics",
		});
		expect(calls.enablePack).toEqual({ packId: "coding_basics" });
		expect(calls.disablePack).toEqual({ packId: "coding_basics" });
		expect(calls.reset).toEqual({ confirm: true });
		expect(calls.updateTheme).toEqual({
			modeId: "coding",
			themeId: "coding_cyan",
		});
		expect(calls.updateStylePack).toEqual({
			modeId: "coding",
			stylePackId: "coding_basics",
		});
		expect(calls.saveSettings).toEqual({ settings: addonSettings });
		expect(calls.togglePack).toEqual({
			packId: "coding_basics",
			enabled: false,
		});
		expect(calls.toggleContribution).toEqual({
			packId: "coding_basics",
			contributionType: "preference",
			contributionId: "mvp",
			enabled: false,
		});
		expect(calls.setActiveTheme).toEqual({
			themeId: "coding_cyan",
			sourcePackId: "coding_basics",
		});
		expect(calls.migrate).toBeUndefined();
	});

	test("active theme returns null when backend has no theme", async () => {
		mockTauriCommand("theme_get_active", null);

		expect(await getActiveTheme()).toBeNull();
	});
});
