import { safeInvoke } from "./tauriBridge";

const invoke = safeInvoke;

import { z } from "zod";
import {
	AddonSettingsSchema,
	AppearanceSettingsSchema,
	BuiltinPackSummarySchema,
	CommandSnippetSchema,
	EffectiveSettingsPreviewSchema,
	InstalledPackSchema,
	InstalledPacksFileSchema,
	InstalledThemeSchema,
	InstalledThemeSummarySchema,
	MarketplaceRepairReportSchema,
	MarketplaceStatusSchema,
	ModeActivationResultSchema,
	ModesFileSchema,
	OpenNivaraModeSchema,
	OpenNivaraThemeSchema,
	PackActivationCapabilitiesSchema,
	type PackManifestSchema,
	PackPreviewSchema,
	ThemePreviewSchema,
	ThemeStoreItemSchema,
} from "./marketplaceSchemas";

export type PackManifest = z.infer<typeof PackManifestSchema>;
export type BuiltinPackSummary = z.infer<typeof BuiltinPackSummarySchema>;
export type PackPreview = z.infer<typeof PackPreviewSchema>;
export type InstalledPacksFile = z.infer<typeof InstalledPacksFileSchema>;
export type InstalledPack = z.infer<typeof InstalledPackSchema>;
export type ModesFile = z.infer<typeof ModesFileSchema>;
export type OpenNivaraMode = z.infer<typeof OpenNivaraModeSchema>;
export type OpenNivaraTheme = z.infer<typeof OpenNivaraThemeSchema>;
export type CommandSnippet = z.infer<typeof CommandSnippetSchema>;
export type MarketplaceRepairReport = z.infer<
	typeof MarketplaceRepairReportSchema
>;
export type MarketplaceStatus = z.infer<typeof MarketplaceStatusSchema>;
export type PackActivationCapabilities = z.infer<
	typeof PackActivationCapabilitiesSchema
>;
export type ModeActivationResult = z.infer<typeof ModeActivationResultSchema>;
export type AddonSettings = z.infer<typeof AddonSettingsSchema>;
export type EffectiveSettingsPreview = z.infer<
	typeof EffectiveSettingsPreviewSchema
>;
export type InstalledThemeSummary = z.infer<typeof InstalledThemeSummarySchema>;
export type ThemeStoreItem = z.infer<typeof ThemeStoreItemSchema>;
export type InstalledTheme = z.infer<typeof InstalledThemeSchema>;
export type AppearanceSettings = z.infer<typeof AppearanceSettingsSchema>;
export type ThemePreview = z.infer<typeof ThemePreviewSchema>;

export async function initMarketplace(): Promise<string> {
	return invoke<string>("marketplace_init");
}

export async function listThemeStoreItems(): Promise<ThemeStoreItem[]> {
	const data = await invoke("theme_store_list");
	return z.array(ThemeStoreItemSchema).parse(data);
}

export async function installBuiltinTheme(
	themeId: string,
): Promise<InstalledTheme> {
	const data = await invoke("theme_install_builtin", { themeId });
	return InstalledThemeSchema.parse(data);
}

export async function installTheme(path: string): Promise<InstalledTheme> {
	const data = await invoke("theme_install_from_path", { path });
	return InstalledThemeSchema.parse(data);
}

export async function uninstallTheme(themeId: string): Promise<void> {
	await invoke("theme_uninstall", { themeId });
}

export async function applyTheme(themeId: string): Promise<void> {
	await invoke("theme_apply", { themeId });
}

export async function resetTheme(): Promise<void> {
	await invoke("theme_reset");
}

export async function getActiveTheme(): Promise<OpenNivaraTheme | null> {
	const data = await invoke("theme_get_active");
	if (!data) return null;
	return OpenNivaraThemeSchema.parse(data);
}

export async function getAppearanceSettings(): Promise<AppearanceSettings> {
	const data = await invoke("theme_get_appearance_settings");
	return AppearanceSettingsSchema.parse(data);
}

export async function listInstalledThemeRecords(): Promise<InstalledTheme[]> {
	const data = await invoke("theme_list_installed");
	return z.array(InstalledThemeSchema).parse(data);
}

export async function previewTheme(themeId: string): Promise<ThemePreview> {
	const data = await invoke("theme_preview", { themeId });
	return ThemePreviewSchema.parse(data);
}

export async function listInstalledPacks(): Promise<InstalledPacksFile> {
	const data = await invoke("marketplace_list_installed_packs");
	return InstalledPacksFileSchema.parse(data);
}

export async function previewPack(path: string): Promise<PackPreview> {
	const data = await invoke("marketplace_preview_pack", { path });
	return PackPreviewSchema.parse(data);
}

export async function installPack(path: string): Promise<InstalledPack> {
	const data = await invoke("marketplace_install_pack", { path });
	return InstalledPackSchema.parse(data);
}

export async function uninstallPack(packId: string): Promise<void> {
	await invoke("marketplace_uninstall_pack", { packId });
}

export async function listBuiltinPacks(): Promise<BuiltinPackSummary[]> {
	const data = await invoke("marketplace_list_builtin_packs");
	return z.array(BuiltinPackSummarySchema).parse(data);
}

export async function installBuiltinPack(
	packId: string,
): Promise<InstalledPack> {
	const data = await invoke("marketplace_install_builtin_pack", { packId });
	return InstalledPackSchema.parse(data);
}

export async function previewInstalledPack(
	packId: string,
): Promise<PackPreview> {
	const data = await invoke("marketplace_preview_installed_pack", { packId });
	return PackPreviewSchema.parse(data);
}

export async function getModes(): Promise<ModesFile> {
	const data = await invoke("marketplace_get_modes");
	return ModesFileSchema.parse(data);
}

export async function setActiveMode(modeId: string): Promise<void> {
	await invoke("marketplace_set_active_mode", { modeId });
}

export async function createMode(mode: any): Promise<void> {
	await invoke("marketplace_create_mode", { mode });
}

export async function addPackToMode(
	modeId: string,
	packId: string,
): Promise<void> {
	await invoke("marketplace_add_pack_to_mode", { modeId, packId });
}

export async function removePackFromMode(
	modeId: string,
	packId: string,
): Promise<void> {
	await invoke("marketplace_remove_pack_from_mode", { modeId, packId });
}

export async function previewBuiltinPack(packId: string): Promise<PackPreview> {
	const data = await invoke("marketplace_preview_builtin_pack", { packId });
	return PackPreviewSchema.parse(data);
}

export async function listInstalledThemes(): Promise<InstalledThemeSummary[]> {
	const data = await invoke("marketplace_list_installed_themes");
	return z.array(InstalledThemeSummarySchema).parse(data);
}

export async function getActiveCommandSnippets(): Promise<CommandSnippet[]> {
	const data = await invoke("marketplace_get_active_command_snippets");
	return z.array(CommandSnippetSchema).parse(data);
}

export async function repairMarketplace(
	dryRun: boolean,
): Promise<MarketplaceRepairReport> {
	const data = await invoke("marketplace_repair", { dryRun });
	return MarketplaceRepairReportSchema.parse(data);
}

export async function getMarketplaceStatus(): Promise<MarketplaceStatus> {
	const data = await invoke("marketplace_status");
	return MarketplaceStatusSchema.parse(data);
}

export async function setPackEnabled(
	packId: string,
	enabled: boolean,
): Promise<void> {
	if (enabled) {
		await invoke("marketplace_enable_pack", { packId });
	} else {
		await invoke("marketplace_disable_pack", { packId });
	}
}

export async function resetMarketplace(): Promise<void> {
	await invoke("marketplace_reset", { confirm: true });
}

export async function getPackActivationCapabilities(
	packId: string,
): Promise<PackActivationCapabilities> {
	const data = await invoke("marketplace_get_pack_activation_capabilities", {
		packId,
	});
	return PackActivationCapabilitiesSchema.parse(data);
}

export async function addPackToModeWithActivation(
	modeId: string,
	packId: string,
	applyTheme: boolean,
	applyStyle: boolean,
): Promise<ModeActivationResult> {
	const data = await invoke("marketplace_add_pack_to_mode_with_activation", {
		modeId,
		packId,
		applyTheme,
		applyStyle,
	});
	return ModeActivationResultSchema.parse(data);
}

export async function createModeFromPack(
	packId: string,
	modeId: string,
	modeName: string,
	activate: boolean,
	applyTheme: boolean,
	applyStyle: boolean,
): Promise<OpenNivaraMode> {
	const data = await invoke("marketplace_create_mode_from_pack", {
		packId,
		modeId,
		modeName,
		activate,
		applyTheme,
		applyStyle,
	});
	return OpenNivaraModeSchema.parse(data);
}

export async function updateModeTheme(
	modeId: string,
	themeId: string | null,
): Promise<void> {
	await invoke("marketplace_update_mode_theme", { modeId, themeId });
}

export async function updateModeStylePack(
	modeId: string,
	stylePackId: string | null,
): Promise<void> {
	await invoke("marketplace_update_mode_style_pack", { modeId, stylePackId });
}

export async function getAddonSettings(): Promise<AddonSettings> {
	const data = await invoke("marketplace_get_addon_settings");
	return AddonSettingsSchema.parse(data);
}

export async function saveAddonSettings(
	settings: AddonSettings,
): Promise<void> {
	await invoke("marketplace_save_addon_settings", { settings });
}

export async function togglePackEnabled(
	packId: string,
	enabled: boolean,
): Promise<void> {
	await invoke("marketplace_toggle_pack_enabled", { packId, enabled });
}

export async function toggleContributionEnabled(
	packId: string,
	contributionType: string,
	contributionId: string,
	enabled: boolean,
): Promise<void> {
	await invoke("marketplace_toggle_contribution_enabled", {
		packId,
		contributionType,
		contributionId,
		enabled,
	});
}

export async function setActiveTheme(
	themeId: string | null,
	sourcePackId: string | null,
): Promise<void> {
	await invoke("marketplace_set_active_theme", { themeId, sourcePackId });
}

export async function migrateAddons(): Promise<void> {
	await invoke("marketplace_migrate_addons");
}

export async function hasLegacyModes(): Promise<boolean> {
	return invoke<boolean>("marketplace_has_legacy_modes");
}

export async function getEffectiveSettingsPreview(): Promise<EffectiveSettingsPreview> {
	const data = await invoke("marketplace_get_effective_settings_preview");
	return EffectiveSettingsPreviewSchema.parse(data);
}
