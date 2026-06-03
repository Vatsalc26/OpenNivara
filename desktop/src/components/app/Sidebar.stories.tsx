import type { Meta, StoryObj } from "@storybook/tanstack-react";
import { expect } from "storybook/test";
import { Sidebar } from "./Sidebar";

const meta = {
	component: Sidebar,
	tags: ["ai-generated"],
	args: {
		activeView: "chat",
		onNavigate: () => {},
		onNewChat: () => {},
		apiKeyReady: true,
		toolsEnabled: true,
	},
} satisfies Meta<typeof Sidebar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Ready: Story = {
	play: async ({ canvas }) => {
		await expect(canvas.getByText("Ready")).toBeVisible();
	},
};

export const ApiMissing: Story = {
	args: { apiKeyReady: false, activeView: "settings" },
};
