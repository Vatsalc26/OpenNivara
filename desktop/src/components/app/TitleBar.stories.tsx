import type { Meta, StoryObj } from "@storybook/tanstack-react";
import { TitleBar } from "./TitleBar";

const meta = {
	component: TitleBar,
	tags: ["ai-generated"],
} satisfies Meta<typeof TitleBar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Chat: Story = {
	args: { activeView: "chat" },
};

export const Store: Story = {
	args: { activeView: "marketplace" },
};
