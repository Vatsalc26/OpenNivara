import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";
import { CommandPalette } from "./CommandPalette";

describe("CommandPalette", () => {
	test("does not render when closed and toggles with keyboard shortcut", () => {
		const setOpen = vi.fn();

		const { rerender } = render(
			<CommandPalette
				open={false}
				setOpen={setOpen}
				onNavigate={() => {}}
				onNewChat={() => {}}
			/>,
		);

		expect(
			screen.queryByLabelText("Global Command Menu"),
		).not.toBeInTheDocument();

		fireEvent.keyDown(document, { key: "k", ctrlKey: true });
		expect(setOpen).toHaveBeenCalledWith(expect.any(Function));

		rerender(
			<CommandPalette
				open={true}
				setOpen={setOpen}
				onNavigate={() => {}}
				onNewChat={() => {}}
			/>,
		);

		fireEvent.keyDown(document, { key: "Escape" });
		expect(setOpen).toHaveBeenCalledWith(false);
	});

	test("navigates, starts new chat, toggles inspector, and closes after actions", () => {
		const navigations: Array<[string, any?]> = [];
		const setOpen = vi.fn();
		const onNewChat = vi.fn();
		const onToggleInspector = vi.fn();

		render(
			<CommandPalette
				open={true}
				setOpen={setOpen}
				onNavigate={(view, tab) => navigations.push([view, tab])}
				onNewChat={onNewChat}
				showInspector={false}
				onToggleInspector={onToggleInspector}
			/>,
		);

		fireEvent.click(screen.getByText("Go to OpenNivara Settings Hub"));
		expect(navigations).toContainEqual(["settings", "profile"]);
		expect(setOpen).toHaveBeenCalledWith(false);

		fireEvent.click(screen.getByText("Go to Chat & Consultation"));
		fireEvent.click(screen.getByText("Go to Session History Log"));
		fireEvent.click(screen.getByText("Go to Tool Security Policies"));
		fireEvent.click(screen.getByText("Go to Workspace Landmarks Map"));
		fireEvent.click(
			screen.getByText("Open Settings: Response Style Guidelines"),
		);
		fireEvent.click(screen.getByText("Open Settings: User Identity"));
		fireEvent.click(
			screen.getByText("Open Settings: Topic Preferences Triggers"),
		);
		fireEvent.click(
			screen.getByText("Open Settings: Project Goals & Contexts"),
		);
		fireEvent.click(screen.getByText("Open Settings: Config Files Explorer"));

		expect(navigations).toContainEqual(["chat", undefined]);
		expect(navigations).toContainEqual(["sessions", undefined]);
		expect(navigations).toContainEqual(["tools", undefined]);
		expect(navigations).toContainEqual(["workspace", undefined]);
		expect(navigations).toContainEqual(["settings", "style"]);
		expect(navigations).toContainEqual(["settings", "preferences"]);
		expect(navigations).toContainEqual(["settings", "contexts"]);
		expect(navigations).toContainEqual(["settings", "paths"]);

		fireEvent.click(screen.getByText("Initialize Clean Consultation Chat"));
		expect(onNewChat).toHaveBeenCalled();

		fireEvent.click(screen.getByText("Open Chat Context Inspector Drawer"));
		expect(navigations).toContainEqual(["chat", undefined]);
		expect(onToggleInspector).toHaveBeenCalled();
	});
});
