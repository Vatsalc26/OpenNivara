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
});
