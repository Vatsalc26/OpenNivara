import type { Meta, StoryObj } from "@storybook/tanstack-react";
import { StoreView } from "./StoreView";

const meta = {
	component: StoreView,
	tags: ["ai-generated"],
} satisfies Meta<typeof StoreView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Themes: Story = {
	args: { defaultTab: "themes" },
};

export const Installed: Story = {
	args: { defaultTab: "installed" },
};
