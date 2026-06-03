import type { Meta, StoryObj } from "@storybook/tanstack-react";
import { expect } from "storybook/test";
import { Button } from "./button";

const meta = {
	component: Button,
	tags: ["ai-generated"],
	args: { children: "Run check" },
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {};

export const Secondary: Story = {
	args: { variant: "secondary", children: "Open settings" },
};

export const Disabled: Story = {
	args: { disabled: true, children: "Saving" },
};

export const CssCheck: Story = {
	args: { children: "Styled primary" },
	play: async ({ canvas }) => {
		const button = canvas.getByRole("button", { name: /styled primary/i });
		await expect(button.dataset.variant).toBe("default");
		await expect(button.dataset.size).toBe("default");
		await expect(button.className).toContain("bg-primary");
	},
};
