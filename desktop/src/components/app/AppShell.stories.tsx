import type { Meta, StoryObj } from "@storybook/tanstack-react";
import { useState } from "react";
import { AppShell } from "./AppShell";

const meta = {
	component: AppShell,
	tags: ["ai-generated"],
	args: {
		activeView: "chat",
		onNavigate: () => {},
		onNewChat: () => {},
		apiKeyReady: true,
		toolsEnabled: true,
		paletteOpen: false,
		setPaletteOpen: () => {},
		children: <div className="p-6">Content frame</div>,
	},
} satisfies Meta<typeof AppShell>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ChatShell: Story = {
	render: () => {
		const [paletteOpen, setPaletteOpen] = useState(false);
		return (
			<AppShell
				activeView="chat"
				onNavigate={() => {}}
				onNewChat={() => {}}
				apiKeyReady
				toolsEnabled
				paletteOpen={paletteOpen}
				setPaletteOpen={setPaletteOpen}
			>
				<div className="p-6">Chat content frame</div>
			</AppShell>
		);
	},
};

export const SettingsShellApiMissing: Story = {
	render: () => {
		const [paletteOpen, setPaletteOpen] = useState(false);
		return (
			<AppShell
				activeView="settings"
				onNavigate={() => {}}
				onNewChat={() => {}}
				apiKeyReady={false}
				toolsEnabled
				paletteOpen={paletteOpen}
				setPaletteOpen={setPaletteOpen}
			>
				<div className="p-6">Settings content frame</div>
			</AppShell>
		);
	},
};
