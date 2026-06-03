import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test } from "vitest";
import { clearTauriMocks, mockTauriCommand } from "../../test/mockTauri";
import { ThemeProvider } from "../../theme/ThemeProvider";
import { AppShell } from "./AppShell";

const queryClient = new QueryClient();

describe("AppShell Layout Component Tests", () => {
	beforeEach(() => {
		clearTauriMocks();
		mockTauriCommand("check_api_key", true);
		mockTauriCommand("marketplace_get_addon_settings", {
			schema_version: 1,
			active_theme_id: null,
			active_theme_source_pack_id: null,
			enabled_packs: [],
			disabled_contributions: [],
		});
		mockTauriCommand("marketplace_get_active_addon_theme", null);
	});

	test("1. Sidebar and titlebar render correctly inside AppShell", () => {
		render(
			<QueryClientProvider client={queryClient}>
				<ThemeProvider>
					<AppShell
						activeView="chat"
						onNavigate={() => {}}
						onNewChat={() => {}}
						apiKeyReady={true}
						toolsEnabled={true}
						paletteOpen={false}
						setPaletteOpen={() => {}}
					>
						<div data-testid="chat-content">Main Chat Area</div>
					</AppShell>
				</ThemeProvider>
			</QueryClientProvider>,
		);

		expect(screen.getByTestId("chat-content")).toBeInTheDocument();
		expect(screen.getByText(/Consultation/i)).toBeInTheDocument();
		expect(screen.getByText(/Safe Shell/i)).toBeInTheDocument();
	});
});
