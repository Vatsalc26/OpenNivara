import "@fontsource-variable/geist";
import "../src/index.css";

import type { Preview } from "@storybook/tanstack-react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import MockDate from "mockdate";
import { initialize, mswLoader } from "msw-storybook-addon";
import type { ReactNode } from "react";
import { Toaster } from "sonner";
import { builtinPacksFixture } from "../src/test/fixtures/builtinPacks";
import { effectiveSettingsPreviewFixture } from "../src/test/fixtures/effectiveSettingsPreview";
import { installedPacksFixture } from "../src/test/fixtures/installedPacks";
import { packPreviewsFixture } from "../src/test/fixtures/packPreviews";
import { themesFixture } from "../src/test/fixtures/themes";
import { mockTauriCommand } from "../src/test/mockTauri";
import { ThemeProvider } from "../src/theme/ThemeProvider";
import { mswHandlers } from "./msw-handlers";

(globalThis as any).__VITEST__ = true;

initialize({ onUnhandledRequest: "bypass" });

const fullPreferences = {
	schema_version: 1,
	sections: [
		{
			id: "coding_help",
			enabled: true,
			send_policy: "triggered_strict",
			description: "Prefer small, tested implementation steps.",
			triggers: ["code", "bug"],
			required_any: ["rust", "typescript"],
			negative_triggers: ["restaurant"],
			min_score: 2,
			likes: [{ item: "clear patches", strength: 5 }],
			dislikes: [{ item: "hidden magic", strength: 4 }],
			notes: ["Keep examples practical."],
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
			title: "OpenNivara desktop",
			summary: "Local Tauri assistant context.",
			triggers: ["opennivara", "tauri"],
			required_any: ["opennivara"],
			negative_triggers: [],
			min_score: 1,
			facts: ["Uses Rust and Tauri."],
			rules: ["Keep tools safe."],
		},
	],
};

function setupStoryMocks() {
	mockTauriCommand("check_api_key", false);
	mockTauriCommand("ask_opennivara", {
		session_id: "storybook-session",
		answer: "Storybook deterministic response.",
	});
	mockTauriCommand("get_profile", {
		schema_version: 1,
		identity: { display_name: "Storybook User" },
	});
	mockTauriCommand("get_preferences", fullPreferences);
	mockTauriCommand("save_preferences", {});
	mockTauriCommand("get_contexts", fullContexts);
	mockTauriCommand("save_contexts", {});
	mockTauriCommand("get_style", {});
	mockTauriCommand("save_style", {});
	mockTauriCommand("list_tools", {
		general: { enabled: true, max_tool_rounds: 3, show_tool_activity: true },
		paths: { allowed_roots: [], blocked_patterns: [] },
		tools: {},
	});
	mockTauriCommand("preview_context_for_message", {
		effective_prompt: "Effective story prompt",
		profile_context: "Story profile",
		preference_context: "Story preferences",
		context_context: "Story pinned contexts",
		raw_user_message: "hello",
		addon_context: "Addon context",
		active_addons: ["coding_basics"],
		active_theme: "Coding Cyan",
		theme_source_pack: "coding_basics",
		style_source_pack: "coding_basics",
	});
	mockTauriCommand("pin_context", {});
	mockTauriCommand("unpin_context", {});
	mockTauriCommand("marketplace_status", {
		marketplace_dir: "storybook/marketplace",
		installed_count: 1,
		enabled_count: 1,
		disabled_count: 0,
		modes_count: 1,
		active_mode_id: "default",
		active_mode_name: "Default",
		missing_pack_ids: [],
		builtin_packs_available: ["coding_basics"],
		builtin_resource_path_checked: "storybook/packs",
		builtin_resource_path_exists: true,
		disabled_packs_in_active_mode: [],
	});
	mockTauriCommand("marketplace_init", "ok");
	mockTauriCommand("marketplace_list_installed_packs", installedPacksFixture);
	mockTauriCommand("marketplace_list_builtin_packs", builtinPacksFixture);
	mockTauriCommand(
		"marketplace_preview_builtin_pack",
		packPreviewsFixture.coding_basics,
	);
	mockTauriCommand(
		"marketplace_preview_installed_pack",
		packPreviewsFixture.coding_basics,
	);
	mockTauriCommand(
		"marketplace_install_builtin_pack",
		installedPacksFixture.installed[0],
	);
	mockTauriCommand("marketplace_uninstall_pack", {});
	mockTauriCommand("marketplace_list_installed_themes", themesFixture);
	mockTauriCommand("marketplace_get_active_addon_theme", null);
	mockTauriCommand("marketplace_get_addon_settings", {
		schema_version: 1,
		active_theme_id: null,
		active_theme_source_pack_id: null,
		enabled_packs: ["coding_basics"],
		disabled_contributions: [],
	});
	mockTauriCommand("marketplace_set_active_theme", {});
	mockTauriCommand(
		"marketplace_get_effective_settings_preview",
		effectiveSettingsPreviewFixture,
	);
	mockTauriCommand("marketplace_has_legacy_modes", false);
}

function Providers({ children }: { children: ReactNode }) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false, refetchOnWindowFocus: false } },
	});

	return (
		<QueryClientProvider client={queryClient}>
			<ThemeProvider>
				{children}
				<Toaster richColors position="top-right" />
			</ThemeProvider>
		</QueryClientProvider>
	);
}

const preview: Preview = {
	decorators: [
		(Story) => (
			<Providers>
				<Story />
			</Providers>
		),
	],
	loaders: [mswLoader],
	parameters: {
		msw: { handlers: mswHandlers },
		layout: "fullscreen",
	},
	beforeEach() {
		setupStoryMocks();
		document.documentElement.classList.add("dark");
		localStorage.setItem("theme", "dark");
		MockDate.set("2026-06-02T12:00:00Z");
	},
};

export default preview;
