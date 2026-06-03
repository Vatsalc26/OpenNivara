import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { clearTauriMocks } from "../../test/mockTauri";
import { ThemeProvider } from "../../theme/ThemeProvider";
import { TitleBar } from "./TitleBar";

const mockWindow = vi.hoisted(() => ({
	isMaximized: vi.fn(),
	minimize: vi.fn(),
	toggleMaximize: vi.fn(),
	close: vi.fn(),
}));

vi.mock("@tauri-apps/api/window", () => ({
	getCurrentWindow: () => mockWindow,
}));

describe("TitleBar Header Component Tests", () => {
	beforeEach(() => {
		clearTauriMocks();
		delete (window as any).__TAURI_INTERNALS__;
		mockWindow.isMaximized.mockReset();
		mockWindow.minimize.mockReset();
		mockWindow.toggleMaximize.mockReset();
		mockWindow.close.mockReset();
		mockWindow.isMaximized.mockResolvedValue(false);
		mockWindow.minimize.mockResolvedValue(undefined);
		mockWindow.toggleMaximize.mockResolvedValue(undefined);
		mockWindow.close.mockResolvedValue(undefined);
	});

	test.each([
		["chat", "Consultation & Chat"],
		["sessions", "Session Log History"],
		["tools", "Tool Security & Permissions"],
		["workspace", "Workspace Knowledge Map"],
		["settings", "OpenNivara Hub Settings"],
		["marketplace", "OpenNivara Store"],
		["unknown", "Assistant"],
	])("1. Renders page title for %s", (activeView, title) => {
		const { unmount } = render(
			<ThemeProvider>
				<TitleBar activeView={activeView} />
			</ThemeProvider>,
		);

		expect(screen.getByText("OPENNIVARA")).toBeInTheDocument();
		expect(screen.getByText(title)).toBeInTheDocument();
		unmount();
	});

	test("2. Calls Tauri window controls and updates maximize title", async () => {
		(window as any).__TAURI_INTERNALS__ = {};
		mockWindow.isMaximized
			.mockResolvedValueOnce(false)
			.mockResolvedValueOnce(true);

		render(
			<ThemeProvider>
				<TitleBar activeView="chat" />
			</ThemeProvider>,
		);

		fireEvent.click(screen.getByTitle("Minimize"));
		fireEvent.click(screen.getByTitle("Maximize"));
		fireEvent.click(screen.getByTitle("Close"));

		await waitFor(() => {
			expect(mockWindow.minimize).toHaveBeenCalled();
			expect(mockWindow.toggleMaximize).toHaveBeenCalled();
			expect(mockWindow.close).toHaveBeenCalled();
			expect(screen.getByTitle("Restore Down")).toBeInTheDocument();
		});
	});
});
