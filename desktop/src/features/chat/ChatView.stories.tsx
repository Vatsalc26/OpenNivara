import type { Meta, StoryObj } from "@storybook/tanstack-react";
import { ChatView } from "./ChatView";

const meta = {
	component: ChatView,
	tags: ["ai-generated"],
	args: {
		currentSessionId: null,
		onSessionCreated: () => {},
	},
} satisfies Meta<typeof ChatView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Empty: Story = {};

export const WithMessages: Story = {
	args: {
		initialMessages: [
			{ role: "user", content: "Explain this crate", timestamp: new Date() },
			{
				role: "model",
				content: "This crate exposes a Tauri desktop assistant.",
				timestamp: new Date(),
			},
		],
	},
};

export const InspectorOpen: Story = {
	args: {
		showInspector: true,
		currentSessionId: "storybook-session",
	},
};
