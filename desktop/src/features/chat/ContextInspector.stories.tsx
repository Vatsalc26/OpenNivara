import type { Meta, StoryObj } from "@storybook/tanstack-react";
import { ContextInspector } from "./ContextInspector";

const meta = {
	component: ContextInspector,
	tags: ["ai-generated"],
	args: {
		sessionId: "storybook-session",
		currentInputText: "hello",
		onClose: () => {},
	},
} satisfies Meta<typeof ContextInspector>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Idle: Story = {};

export const WithCurrentInput: Story = {
	args: { currentInputText: "hello from storybook" },
};

export const NoSession: Story = {
	args: { sessionId: null },
};
