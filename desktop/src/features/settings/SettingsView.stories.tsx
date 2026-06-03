import type { Meta, StoryObj } from "@storybook/tanstack-react";
import { SettingsView } from "./SettingsView";

const paths = {
	profile: "storybook/profile.toml",
	preferences: "storybook/preferences.toml",
	style: "storybook/style.toml",
	tools: "storybook/tools.toml",
	contexts: "storybook/contexts.toml",
	telegram: "storybook/telegram.toml",
};

const meta = {
	component: SettingsView,
	tags: ["ai-generated"],
	args: { paths },
} satisfies Meta<typeof SettingsView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Profile: Story = {};

export const Preferences: Story = {
	args: { paths, defaultTab: "preferences" },
};

export const Appearance: Story = {
	args: { paths, defaultTab: "appearance" },
};
