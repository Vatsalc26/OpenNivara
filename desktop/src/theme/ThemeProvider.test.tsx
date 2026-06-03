import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test } from "vitest";
import { clearTauriMocks, mockTauriCommand } from "../test/mockTauri";
import { ThemeProvider, useOpenNivaraTheme } from "./ThemeProvider";

function TestComponent() {
	const { activeTheme, refreshTheme } = useOpenNivaraTheme();
	return (
		<div>
			<span data-testid="theme-name">
				{activeTheme ? activeTheme.name : "Default Theme"}
			</span>
			<button data-testid="refresh-btn" onClick={refreshTheme}>
				Refresh
			</button>
		</div>
	);
}

describe("ThemeProvider Regression Tests", () => {
	beforeEach(() => {
		clearTauriMocks();
	});

	test("1. ThemeProvider loads active theme from theme_get_active", async () => {
		let themeGetActiveCalled = false;

		mockTauriCommand("theme_get_active", () => {
			themeGetActiveCalled = true;
			return {
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
			};
		});

		render(
			<ThemeProvider>
				<TestComponent />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByTestId("theme-name")).toHaveTextContent("Coding Cyan");
		});

		expect(themeGetActiveCalled).toBe(true);
	});

	test("2. Applying Coding Cyan updates document CSS variables", async () => {
		mockTauriCommand("theme_get_active", () => {
			return {
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
			};
		});

		render(
			<ThemeProvider>
				<TestComponent />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByTestId("theme-name")).toHaveTextContent("Coding Cyan");
		});

		const rootStyle = document.documentElement.style;
		expect(rootStyle.getPropertyValue("--background")).toBe("#0f172a");
		expect(rootStyle.getPropertyValue("--primary")).toBe("#06b6d4");
		expect(rootStyle.getPropertyValue("--opennivara-bg-glow-1")).toContain(
			"#a78bfa",
		);
	});

	test("3. Applying default theme clears custom CSS variables", async () => {
		mockTauriCommand("theme_get_active", () => {
			return {
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
			};
		});

		render(
			<ThemeProvider>
				<TestComponent />
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByTestId("theme-name")).toHaveTextContent("Coding Cyan");
		});

		mockTauriCommand("theme_get_active", () => {
			return null;
		});

		fireEvent.click(screen.getByTestId("refresh-btn"));

		await waitFor(() => {
			expect(screen.getByTestId("theme-name")).toHaveTextContent(
				"Default Theme",
			);
		});

		const rootStyle = document.documentElement.style;
		expect(rootStyle.getPropertyValue("--background")).toBe("");
		expect(rootStyle.getPropertyValue("--primary")).toBe("");
		expect(rootStyle.getPropertyValue("--opennivara-bg-glow-1")).toBe("");
	});
});
