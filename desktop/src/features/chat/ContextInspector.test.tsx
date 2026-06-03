import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test } from "vitest";
import { clearTauriMocks, mockTauriCommand } from "../../test/mockTauri";
import { ThemeProvider } from "../../theme/ThemeProvider";
import { ContextInspector } from "./ContextInspector";

const mockContextPreview = {
	active_theme: { id: "coding_cyan", name: "Coding Cyan", ui_only: true },
	profile_sent: ["Display Name: DevPro", "Coding Level: Advanced"],
	style_sent: ["Tone: Professional", "Detail: Detailed"],
	preferences_sent: ["Pref Tone triggered"],
	contexts_pinned: ["custom_goal"],
	contexts_sent: ["Always sent goal info"],
	final_context_text: "Final Assembled System Prompt content",
	warnings: ["This is a test warning"],
};

const mockContexts = {
	schema_version: 1,
	contexts: [
		{
			id: "custom_goal",
			enabled: true,
			kind: "goal",
			send_policy: "always",
			title: "Custom Goal Title",
			summary: "Summary of custom goal",
			triggers: [],
			required_any: [],
			negative_triggers: [],
			min_score: 0,
			facts: ["Fact 1"],
			rules: [],
		},
		{
			id: "unpinned_goal",
			enabled: true,
			kind: "goal",
			send_policy: "session_pinned",
			title: "Unpinned Goal Title",
			summary: "Summary of unpinned goal",
			triggers: [],
			required_any: [],
			negative_triggers: [],
			min_score: 0,
			facts: ["Fact 2"],
			rules: [],
		},
	],
};

describe("ContextInspector Unit & Integration Tests", () => {
	beforeEach(() => {
		clearTauriMocks();
		mockTauriCommand("get_contexts", mockContexts);
		mockTauriCommand("marketplace_get_addon_settings", {
			schema_version: 1,
			active_theme_id: "coding_cyan",
			active_theme_source_pack_id: "coding_basics",
			enabled_packs: ["coding_basics"],
			disabled_contributions: [],
		});
		mockTauriCommand("theme_get_active", {
			schema_version: 1,
			id: "coding_cyan",
			colors: { background: "#000", foreground: "#fff" },
		});
	});

	test("1. Renders title and evaluates initial prompt 'hello' successfully", async () => {
		let callCount = 0;
		mockTauriCommand("preview_context_for_message", (args: any) => {
			callCount++;
			expect(args.message).toBe("hello");
			return mockContextPreview;
		});

		render(
			<ThemeProvider>
				<ContextInspector
					sessionId="session_1"
					currentInputText="hello"
					onClose={() => {}}
				/>
			</ThemeProvider>,
		);

		// Spinner should be visible first
		expect(
			screen.getByText("Running selector evaluation..."),
		).toBeInTheDocument();

		await waitFor(() => {
			expect(screen.getByText("Context Inspector")).toBeInTheDocument();
			expect(screen.getByText("Active Visual Theme")).toBeInTheDocument();
			expect(screen.getByText("Coding Cyan")).toBeInTheDocument();
			expect(screen.getByText(/not sent to the model/i)).toBeInTheDocument();
		});

		// Check that preview was NOT called repeatedly in a loop
		expect(callCount).toBe(1);
	});

	test("2. Refresh calls preview exactly once", async () => {
		let callCount = 0;
		mockTauriCommand("preview_context_for_message", () => {
			callCount++;
			return mockContextPreview;
		});

		render(
			<ThemeProvider>
				<ContextInspector
					sessionId="session_1"
					currentInputText="hello"
					onClose={() => {}}
				/>
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("Active Visual Theme")).toBeInTheDocument();
		});
		expect(callCount).toBe(1);

		// Click refresh
		const refreshButton = screen.getByTitle("Refresh Preview");
		fireEvent.click(refreshButton);

		await waitFor(() => {
			expect(callCount).toBe(2);
		});
	});

	test("3. Failed preview shows error state and retry option", async () => {
		mockTauriCommand("preview_context_for_message", () => {
			throw new Error("Evaluation error simulation");
		});

		render(
			<ThemeProvider>
				<ContextInspector
					sessionId="session_1"
					currentInputText="hello"
					onClose={() => {}}
				/>
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("Evaluation Failed")).toBeInTheDocument();
			expect(
				screen.getByText("Evaluation error simulation"),
			).toBeInTheDocument();
			expect(
				screen.getByRole("button", { name: /Retry/i }),
			).toBeInTheDocument();
		});
	});

	test("4. Tabs render and change content correctly", async () => {
		mockTauriCommand("preview_context_for_message", () => mockContextPreview);

		render(
			<ThemeProvider>
				<ContextInspector
					sessionId="session_1"
					currentInputText="hello"
					onClose={() => {}}
				/>
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("Profile / Prefs")).toBeInTheDocument();
			expect(screen.getByText("Goals & Pinned")).toBeInTheDocument();
			expect(screen.getByText("Raw Prompt")).toBeInTheDocument();
		});

		// Switch to Goals & Pinned
		const goalsTab = screen.getByRole("button", { name: /Goals & Pinned/i });
		fireEvent.click(goalsTab);

		await waitFor(() => {
			expect(screen.getByText("Session-Pinned Contexts")).toBeInTheDocument();
			expect(
				screen.getByText("Triggered & Always Sent Goals"),
			).toBeInTheDocument();
			expect(screen.getByText("Quick-Pin Context entries")).toBeInTheDocument();
		});

		// Switch to Raw Prompt
		const promptTab = screen.getByRole("button", { name: /Raw Prompt/i });
		fireEvent.click(promptTab);

		await waitFor(() => {
			expect(screen.getByText("Effective Prompt")).toBeInTheDocument();
			expect(
				screen.getByText("Final Assembled System Prompt content"),
			).toBeInTheDocument();
		});
	});

	test("5. UI does NOT contain store behavior terminology", async () => {
		mockTauriCommand("preview_context_for_message", () => mockContextPreview);

		render(
			<ThemeProvider>
				<ContextInspector
					sessionId="session_1"
					currentInputText="hello"
					onClose={() => {}}
				/>
			</ThemeProvider>,
		);

		await waitFor(() => {
			expect(screen.getByText("Active Visual Theme")).toBeInTheDocument();
		});

		expect(screen.queryByText("Active Mode")).not.toBeInTheDocument();
		expect(screen.queryByText("Packs in this Mode")).not.toBeInTheDocument();
		expect(screen.queryByText("Loadout")).not.toBeInTheDocument();
		expect(screen.queryByText("Enabled Add-ons")).not.toBeInTheDocument();
		expect(screen.queryByText("coding_basics")).not.toBeInTheDocument();
		expect(screen.queryByText("study_coach")).not.toBeInTheDocument();
	});
});
