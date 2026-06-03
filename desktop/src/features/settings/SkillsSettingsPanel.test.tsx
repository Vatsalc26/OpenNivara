import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test } from "vitest";
import { clearTauriMocks, mockTauriCommand } from "../../test/mockTauri";
import { SkillsSettingsPanel } from "./SkillsSettingsPanel";

const skills = [
	{
		id: "upsc_exam_preparation",
		pack_id: "upsc_exam",
		name: "UPSC Exam Preparation",
		description: "Plan UPSC prep.",
		category: "education",
		enabled: false,
		route_policy: "auto",
		risk_level: "low",
		allowed_tools: [],
	},
	{
		id: "notes_reader",
		pack_id: null,
		name: "Notes Reader",
		description: "Read local notes.",
		category: "workspace",
		enabled: true,
		route_policy: "manual_only",
		risk_level: "low",
		allowed_tools: ["read_file"],
	},
];

describe("SkillsSettingsPanel", () => {
	beforeEach(() => {
		clearTauriMocks();
		mockTauriCommand("skills_list", skills);
		mockTauriCommand("skills_set_enabled", null);
		mockTauriCommand("skills_test_route", {
			primary_skill: {
				id: "upsc_exam_preparation",
				pack_id: "upsc_exam",
				name: "UPSC Exam Preparation",
				score: 45,
				reason: "alias phrase +25",
				allowed_tools: [],
				denied_tools: [],
			},
			supporting_skills: [],
			candidates: [
				{
					id: "upsc_exam_preparation",
					name: "UPSC Exam Preparation",
					score: 45,
					accepted: true,
					reason: "alias phrase +25",
				},
			],
			confidence: 0.45,
			reason: "alias phrase +25",
			warnings: [],
		});
	});

	test("renders installed skills and toggles enablement", async () => {
		render(<SkillsSettingsPanel />);

		await waitFor(() => {
			expect(screen.getByText("UPSC Exam Preparation")).toBeInTheDocument();
		});

		expect(screen.getByText("Notes Reader")).toBeInTheDocument();
		fireEvent.click(screen.getByRole("button", { name: "Disabled" }));

		await waitFor(() => {
			expect(screen.getByText("UPSC Exam Preparation")).toBeInTheDocument();
		});
	});

	test("filters skills and displays route-test results", async () => {
		render(<SkillsSettingsPanel />);

		await waitFor(() => {
			expect(screen.getByText("UPSC Exam Preparation")).toBeInTheDocument();
		});

		fireEvent.change(screen.getByPlaceholderText("Search installed skills"), {
			target: { value: "notes" },
		});
		expect(screen.queryByText("UPSC Exam Preparation")).toBeNull();
		expect(screen.getByText("Notes Reader")).toBeInTheDocument();

		fireEvent.change(
			screen.getByPlaceholderText("Type a message to test skill routing"),
			{ target: { value: "make an upsc plan" } },
		);
		fireEvent.click(screen.getByRole("button", { name: "Test Route" }));

		await waitFor(() => {
			expect(
				screen.getByText(/UPSC Exam Preparation \(45\)/),
			).toBeInTheDocument();
		});
	});

	test("renders empty state", async () => {
		mockTauriCommand("skills_list", []);
		render(<SkillsSettingsPanel />);

		await waitFor(() => {
			expect(screen.getByText(/No skills installed yet/)).toBeInTheDocument();
		});
	});
});
