import { render, screen } from "@testing-library/react";
import { describe, expect, test } from "vitest";
import {
	Card,
	CardAction,
	CardContent,
	CardDescription,
	CardFooter,
	CardHeader,
	CardTitle,
} from "./card";

describe("Card primitives", () => {
	test("renders all card slots and forwards classes", () => {
		render(
			<Card size="sm" className="outer-card">
				<CardHeader className="header">
					<CardTitle className="title">Card title</CardTitle>
					<CardDescription className="description">
						Card description
					</CardDescription>
					<CardAction className="action">Action</CardAction>
				</CardHeader>
				<CardContent className="content">Card content</CardContent>
				<CardFooter className="footer">Card footer</CardFooter>
			</Card>,
		);

		expect(screen.getByText("Card title")).toHaveAttribute(
			"data-slot",
			"card-title",
		);
		expect(screen.getByText("Card description")).toHaveAttribute(
			"data-slot",
			"card-description",
		);
		expect(screen.getByText("Action")).toHaveAttribute(
			"data-slot",
			"card-action",
		);
		expect(screen.getByText("Card content")).toHaveAttribute(
			"data-slot",
			"card-content",
		);
		expect(screen.getByText("Card footer")).toHaveAttribute(
			"data-slot",
			"card-footer",
		);
		expect(
			screen.getByText("Card title").closest("[data-slot=card]"),
		).toHaveAttribute("data-size", "sm");
	});
});
