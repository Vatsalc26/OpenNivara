import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { clearTauriMocks, mockTauriCommand } from "../../test/mockTauri";
import { ThemeProvider } from "../../theme/ThemeProvider";
import { Sidebar } from "./Sidebar";

function renderSidebar(
	props: Partial<React.ComponentProps<typeof Sidebar>> = {},
) {
	const queryClient = new QueryClient();
	const defaults = {
		activeView: "chat",
		onNavigate: vi.fn(),
		onNewChat: vi.fn(),
		apiKeyReady: true,
		toolsEnabled: true,
	};

	const merged = { ...defaults, ...props };

	render(
		<QueryClientProvider client={queryClient}>
			<ThemeProvider>
				<Sidebar {...merged} />
			</ThemeProvider>
		</QueryClientProvider>,
	);

	return merged;
}

describe("Sidebar Navigation Component Tests", () => {
	beforeEach(() => {
		clearTauriMocks();
		mockTauriCommand("marketplace_get_addon_settings", {
			schema_version: 1,
			active_theme_id: null,
			active_theme_source_pack_id: null,
			enabled_packs: [],
			disabled_contributions: [],
		});
		mockTauriCommand("marketplace_get_active_addon_theme", null);
	});

	test("1. Navigation link clicks trigger view navigation callbacks", () => {
		const props = renderSidebar();

		// Click Settings button
		const settingsBtn = screen.getByRole("button", { name: /Settings/i });
		fireEvent.click(settingsBtn);

		expect(props.onNavigate).toHaveBeenCalledWith("settings");
	});

	test("2. Renders all navigation targets, new chat, and API missing state", () => {
		const props = renderSidebar({
			activeView: "marketplace",
			apiKeyReady: false,
		});

		for (const label of [
			"Chat",
			"Sessions",
			"Tools",
			"Workspace",
			"Settings",
			"Store",
		]) {
			expect(screen.getByRole("button", { name: label })).toBeInTheDocument();
		}

		fireEvent.click(screen.getByRole("button", { name: /New Chat/i }));
		fireEvent.click(screen.getByRole("button", { name: "Store" }));

		expect(props.onNewChat).toHaveBeenCalled();
		expect(props.onNavigate).toHaveBeenCalledWith("marketplace");
		expect(screen.getByText("Missing")).toBeInTheDocument();
	});
});
