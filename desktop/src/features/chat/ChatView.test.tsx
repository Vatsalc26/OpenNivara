import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { clearTauriMocks, mockTauriCommand } from "../../test/mockTauri";
import { ThemeProvider } from "../../theme/ThemeProvider";
import { ChatView } from "./ChatView";

vi.mock("@/components/ui/scroll-area", () => ({
	ScrollArea: ({ children, className, ...props }: any) => (
		<div className={className} {...props}>
			{children}
		</div>
	),
	ScrollBar: () => null,
}));

describe("ChatView Unit Tests", () => {
	beforeEach(() => {
		clearTauriMocks();
		mockTauriCommand("check_api_key", true);
		mockTauriCommand("get_contexts", { schema_version: 1, contexts: [] });
		mockTauriCommand("marketplace_status", {
			marketplace_dir: "C:\\mock\\store",
			installed_count: 0,
			enabled_count: 0,
			disabled_count: 0,
			modes_count: 0,
			active_mode_id: "default",
			active_mode_name: "Default",
			active_theme_id: null,
			active_theme_name: null,
			missing_pack_ids: [],
			disabled_packs_in_active_mode: [],
			builtin_packs_available: [],
			builtin_resource_path_checked: "C:\\mock\\builtin",
			builtin_resource_path_exists: true,
		});
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
	});

	test("1. Renders empty chat consultation state correctly", () => {
		render(
			<ThemeProvider>
				<ChatView currentSessionId={null} onSessionCreated={() => {}} />
			</ThemeProvider>,
		);

		expect(screen.getByText("Consult with OpenNivara")).toBeInTheDocument();
		expect(
			screen.getByPlaceholderText(
				"Ask OpenNivara a question about your files or workspace...",
			),
		).toBeInTheDocument();
	});

	test("2. Typing and sending a message calls ask_opennivara tauri command", async () => {
		let askOpenNivaraCalled = false;
		mockTauriCommand("ask_opennivara", (args: any) => {
			askOpenNivaraCalled = true;
			expect(args.message).toBe("Explain the project workspace structure");
			return {
				session_id: "new_session_id",
				answer: "Here is the structure description",
			};
		});

		render(
			<ThemeProvider>
				<ChatView currentSessionId={null} onSessionCreated={() => {}} />
			</ThemeProvider>,
		);

		const textarea = screen.getByPlaceholderText(
			"Ask OpenNivara a question about your files or workspace...",
		);
		fireEvent.change(textarea, {
			target: { value: "Explain the project workspace structure" },
		});

		// Click send button
		const sendButton = screen
			.getAllByRole("button")
			.find((btn) => btn.querySelector(".lucide-send"));
		if (sendButton) {
			fireEvent.click(sendButton);
		}

		await waitFor(() => {
			expect(askOpenNivaraCalled).toBe(true);
			expect(
				screen.getByText("Here is the structure description"),
			).toBeInTheDocument();
		});
	});

	test("3. Missing API key alerts user appropriately", async () => {
		mockTauriCommand("check_api_key", false);
		mockTauriCommand("ask_opennivara", () => {
			throw new Error("Missing API Key");
		});

		render(
			<ThemeProvider>
				<ChatView currentSessionId={null} onSessionCreated={() => {}} />
			</ThemeProvider>,
		);

		const textarea = screen.getByPlaceholderText(
			"Ask OpenNivara a question about your files or workspace...",
		);
		fireEvent.change(textarea, { target: { value: "Hello" } });

		const sendButton = screen
			.getAllByRole("button")
			.find((btn) => btn.querySelector(".lucide-send"));
		if (sendButton) {
			fireEvent.click(sendButton);
		}

		await waitFor(() => {
			const el = screen.queryAllByText(
				(_, el) => el?.textContent?.includes("Missing API Key") ?? false,
			);
			expect(el.length).toBeGreaterThan(0);
		});
	});

	test("4. Context Inspector side panel opens from chat", async () => {
		mockTauriCommand("preview_context_for_message", {
			active_mode: "Default",
			active_packs: [],
			active_theme: "Default",
			profile_sent: [],
			style_sent: [],
			preferences_sent: [],
			contexts_pinned: [],
			contexts_sent: [],
			final_context_text: "System prompt contents",
		});

		render(
			<ThemeProvider>
				<ChatView currentSessionId="session_123" onSessionCreated={() => {}} />
			</ThemeProvider>,
		);

		const inspectBtn = screen.getByRole("button", { name: /Inspect Context/i });
		fireEvent.click(inspectBtn);

		await waitFor(() => {
			expect(screen.getByText("Context Inspector")).toBeInTheDocument();
		});
	});
});
