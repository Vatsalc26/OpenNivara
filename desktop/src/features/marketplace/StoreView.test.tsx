import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test } from "vitest";
import { clearTauriMocks, mockTauriCommand } from "../../test/mockTauri";
import { ThemeProvider } from "../../theme/ThemeProvider";
import { StoreView } from "./StoreView";

const themeStoreFixture = [
	{
		id: "coding_cyan",
		name: "Coding Cyan",
		description: "Sleek high-contrast cyan palette for developer focus.",
		author: "Vatsal Chavda",
		version: "1.0.0",
		source_kind: "builtin",
		installed: true,
		applied: true,
		preview_colors: {
			background: "#0f172a",
			panel: "#111827",
			card: "#1e293b",
			primary: "#06b6d4",
			accent: "#a78bfa",
			success: "#10b981",
			warning: "#f59e0b",
			danger: "#ef4444",
			foreground: "#f8fafc",
			muted: "#64748b",
		},
		safety: {
			data_only: true,
			contains_executable_code: false,
			modifies_tool_security: false,
			requires_network: false,
		},
	},
	{
		id: "calm_focus",
		name: "Calm Focus",
		description: "Low-noise green theme for study sessions.",
		author: "Vatsal Chavda",
		version: "1.0.0",
		source_kind: "builtin",
		installed: false,
		applied: false,
		preview_colors: {
			background: "#0b1120",
			panel: "#102018",
			card: "#12251c",
			primary: "#34d399",
			accent: "#93c5fd",
			success: "#22c55e",
			warning: "#eab308",
			danger: "#f87171",
			foreground: "#ecfdf5",
			muted: "#6b7280",
		},
		safety: {
			data_only: true,
			contains_executable_code: false,
			modifies_tool_security: false,
			requires_network: false,
		},
	},
];

describe("StoreView themes-only product rule", () => {
	beforeEach(() => {
		clearTauriMocks();
		mockTauriCommand("marketplace_init", "Initialized");
		mockTauriCommand("theme_store_list", themeStoreFixture);
		mockTauriCommand("marketplace_list_builtin_packs", []);
		mockTauriCommand("theme_get_active", {
			schema_version: 1,
			id: "coding_cyan",
			name: "Coding Cyan",
			description: "Sleek high-contrast cyan palette for developer focus.",
			colors: themeStoreFixture[0].preview_colors,
			effects: {
				background_gradient: true,
				glow: "medium",
				density: "normal",
			},
		});
	});

	test("renders safe Store navigation without behavior activation controls", async () => {
		render(
			<ThemeProvider>
				<StoreView />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("Coding Cyan")).toBeInTheDocument();
		});

		expect(
			screen.getByRole("button", { name: /^Themes$/i }),
		).toBeInTheDocument();
		expect(
			screen.getByRole("button", { name: /Installed Themes/i }),
		).toBeInTheDocument();
		expect(
			screen.getByRole("button", { name: /Skill Packs/i }),
		).toBeInTheDocument();
		expect(screen.queryByRole("button", { name: /Add-ons/i })).toBeNull();
		expect(screen.queryByRole("button", { name: /Quick Prompts/i })).toBeNull();
	});

	test("does not render behavior-pack language anywhere in Store", async () => {
		render(
			<ThemeProvider>
				<StoreView />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("Calm Focus")).toBeInTheDocument();
		});

		const storeText = document.body.textContent ?? "";
		expect(storeText).not.toMatch(/preferences|contexts|workspace rules/i);
		expect(storeText).not.toMatch(/command snippets|quick prompts/i);
		expect(storeText).not.toMatch(/add[- ]?on behavior|enabled add-ons/i);
		expect(storeText).not.toMatch(/style pack|loadout|mode/i);
	});

	test("theme details show visual preview and safety only", async () => {
		render(
			<ThemeProvider>
				<StoreView />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("Calm Focus")).toBeInTheDocument();
		});

		const calmCard = screen.getByText("Calm Focus").closest("article");
		expect(calmCard).toBeInTheDocument();
		fireEvent.click(
			(calmCard as HTMLElement).querySelector("button") as HTMLButtonElement,
		);

		await waitFor(() => {
			expect(screen.getByText("Theme Details")).toBeInTheDocument();
			expect(screen.getByText("Data-only theme")).toBeInTheDocument();
			expect(screen.getByText("No executable code")).toBeInTheDocument();
			expect(
				screen.getByText("No tool permission changes"),
			).toBeInTheDocument();
			expect(screen.getByText("No network requirement")).toBeInTheDocument();
		});

		const dialogText = document.body.textContent ?? "";
		expect(dialogText).not.toMatch(/LLM impact|effective prompt/i);
		expect(dialogText).not.toMatch(/preferences|contexts|quick prompts/i);
	});

	test("installed and applied states are visible on theme cards", async () => {
		render(
			<ThemeProvider>
				<StoreView />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("Coding Cyan")).toBeInTheDocument();
		});

		expect(screen.getAllByText("Applied").length).toBeGreaterThan(0);
		expect(screen.getByText("Installed")).toBeInTheDocument();
		expect(
			screen.getByRole("button", { name: /Install Theme/i }),
		).toBeInTheDocument();
	});

	test("skill pack details list included skills and skill-level metadata", async () => {
		mockTauriCommand("marketplace_list_builtin_packs", [
			{
				id: "india_engineering_exams",
				name: "India Engineering Exams",
				version: "1.0.0",
				author: "OpenNivara",
				category: "India Exams",
				description: "Data-only skill pack for JEE Main and GATE.",
				risk_level: "low",
			},
		]);
		mockTauriCommand("marketplace_preview_builtin_pack", {
			manifest: {
				schema_version: 1,
				id: "india_engineering_exams",
				name: "India Engineering Exams",
				version: "1.0.0",
				author: "OpenNivara",
				category: "India Exams",
				description: "Data-only skill pack for JEE Main and GATE.",
				compatibility: {
					opennivara_min_version: "0.1.0",
					opennivara_max_version: "",
				},
				contents: {
					preferences: false,
					contexts: false,
					style_presets: false,
					profile_templates: false,
					tool_presets: false,
					workspace_map_rules: false,
					prompt_behaviors: false,
					command_snippets: false,
					theme: false,
					skills: true,
				},
				safety: {
					contains_executable_code: false,
					modifies_tool_permissions: false,
					requires_network: false,
					risk_level: "low",
				},
			},
			source_path: "packs/builtin/india_engineering_exams",
			warnings: [],
			errors: [],
			additions: {
				preferences_count: 0,
				contexts_count: 0,
				style_presets_count: 0,
				themes_count: 0,
				command_snippets_count: 0,
				workspace_rules_count: 0,
				profile_templates_count: 0,
				tool_presets_count: 0,
				skills_count: 1,
			},
			safety_summary: {
				allowed_to_install: true,
				risk_level: "low",
				modifies_tool_permissions: false,
				contains_executable_code: false,
				requires_network: false,
			},
			skill_previews: [
				{
					schema_version: 1,
					id: "jee_main_mock_test_analyzer",
					pack_id: "india_engineering_exams",
					name: "JEE Main Mock Test Analyzer",
					description: "Analyzes JEE Main mock-test performance.",
					enabled: false,
					category: "india_exams",
					route_policy: "auto",
					aliases: ["jee main mock analysis"],
					triggers: ["jee main", "mock analysis"],
					required_any: ["jee", "jee main", "nta"],
					negative_triggers: ["jee advanced paper", "neet mock"],
					examples: ["Analyze my JEE Main mock"],
					min_score: 25,
					prompt: {
						role: "JEE Main mock-test analysis coach",
						instructions: "Analyze subject marks and mistake types.",
						constraints: ["Do not invent official cutoffs."],
					},
					tools: {
						allow: [],
						deny: ["write_file", "run_command", "open_url"],
					},
					safety: {
						risk_level: "low",
						requires_confirmation: false,
						allows_file_write: false,
						allows_shell: false,
						allows_network: false,
						requires_fresh_info: false,
					},
					metadata: {
						audience: ["class_12", "dropper"],
						country: "IN",
						exam: "JEE Main",
						exam_stage: "mock_analysis",
						language_style: ["english", "hinglish_optional"],
						last_reviewed_at: "2026-06-03",
						freshness_sensitive: false,
						official_source_labels: [],
					},
					store_preview: {
						best_for: ["Students who gave a JEE Main mock test"],
						not_for: ["JEE Advanced-only analysis"],
						sample_prompts: ["Analyze my JEE Main mock: P 52 C 68 M 35"],
						what_it_does: ["Finds weak subjects and mistake types"],
						what_it_will_not_do: ["Invent official cutoffs"],
					},
				},
			],
		});

		render(
			<ThemeProvider>
				<StoreView defaultTab="skills" />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("India Engineering Exams")).toBeInTheDocument();
		});

		fireEvent.click(screen.getByRole("button", { name: /Open Details/i }));

		await waitFor(() => {
			expect(
				screen.getByText("JEE Main Mock Test Analyzer"),
			).toBeInTheDocument();
		});
		expect(
			screen.getByText("Students who gave a JEE Main mock test"),
		).toBeInTheDocument();

		fireEvent.click(
			screen.getByRole("button", { name: /Open skill details/i }),
		);

		await waitFor(() => {
			expect(screen.getByText("Trigger Preview")).toBeInTheDocument();
		});
		expect(
			screen.getByText("Finds weak subjects and mistake types"),
		).toBeInTheDocument();
		expect(screen.getByText("jee advanced paper")).toBeInTheDocument();
		expect(screen.getByText("Allowed: none")).toBeInTheDocument();
		expect(
			screen.getByText("Denied: write_file, run_command, open_url"),
		).toBeInTheDocument();
	});
});
