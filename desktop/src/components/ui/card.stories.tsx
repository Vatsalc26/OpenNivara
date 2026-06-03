import type { Meta, StoryObj } from "@storybook/tanstack-react";
import { Button } from "./button";
import {
	Card,
	CardAction,
	CardContent,
	CardDescription,
	CardFooter,
	CardHeader,
	CardTitle,
} from "./card";

const meta = {
	component: Card,
	tags: ["ai-generated"],
} satisfies Meta<typeof Card>;

export default meta;
type Story = StoryObj<typeof meta>;

export const SettingsPanel: Story = {
	render: () => (
		<Card className="max-w-md">
			<CardHeader>
				<CardTitle>Preference Section</CardTitle>
				<CardDescription>All selector fields are visible.</CardDescription>
				<CardAction>
					<Button size="sm">Edit</Button>
				</CardAction>
			</CardHeader>
			<CardContent>Triggers, strengths, dislikes, and notes.</CardContent>
			<CardFooter>Saved locally</CardFooter>
		</Card>
	),
};

export const CompactStatus: Story = {
	render: () => (
		<Card size="sm" className="max-w-sm">
			<CardHeader>
				<CardTitle>Safe Shell</CardTitle>
				<CardDescription>Dangerous tools disabled.</CardDescription>
			</CardHeader>
		</Card>
	),
};
