import { beforeEach, describe, expect, test } from "vitest";
import { clearTauriMocks, mockTauriCommand } from "../test/mockTauri";
import {
	getSkill,
	listSkills,
	setSkillEnabled,
	testSkillRoute,
} from "./skillClient";

const skillSummary = {
	id: "upsc_exam_preparation",
	pack_id: "upsc_exam",
	name: "UPSC Exam Preparation",
	description: "Plan UPSC prep.",
	category: "education",
	enabled: false,
	route_policy: "auto",
	risk_level: "low",
	allowed_tools: [],
	denied_tools: [],
	exam: null,
	exam_stage: null,
	audience: [],
	language_style: [],
	freshness_sensitive: false,
	official_source_labels: [],
	best_for: [],
	not_for: [],
};

const skillManifest = {
	schema_version: 1,
	...skillSummary,
	aliases: ["upsc prep"],
	triggers: ["upsc"],
	required_any: ["upsc"],
	negative_triggers: [],
	examples: ["make an upsc plan"],
	min_score: 10,
	prompt: {
		role: "Study coach",
		instructions: "Help plan study.",
		constraints: [],
	},
	tools: {
		allow: [],
		deny: [],
	},
	safety: {
		risk_level: "low",
		requires_confirmation: false,
		allows_file_write: false,
		allows_shell: false,
		allows_network: false,
		requires_fresh_info: true,
	},
	metadata: {
		audience: ["aspirant"],
		country: "IN",
		exam: "UPSC CSE",
		exam_stage: "foundation",
		language_style: ["english"],
		freshness_sensitive: true,
		official_source_labels: ["UPSC"],
		last_reviewed_at: "2026-06-03",
	},
	store_preview: {
		best_for: ["Foundation planning"],
		not_for: ["Official notice replacement"],
		sample_prompts: ["make an upsc plan"],
		what_it_does: ["Create a study plan"],
		what_it_will_not_do: ["Replace official notices"],
	},
};

describe("skillClient command contracts", () => {
	beforeEach(() => {
		clearTauriMocks();
	});

	test("read commands parse skill responses", async () => {
		mockTauriCommand("skills_list", [skillSummary]);
		mockTauriCommand("skills_get", skillManifest);
		mockTauriCommand("skills_test_route", {
			primary_skill: {
				id: "upsc_exam_preparation",
				pack_id: "upsc_exam",
				name: "UPSC Exam Preparation",
				score: 45,
				reason: "trigger word +5",
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
					reason: "trigger word +5",
				},
			],
			confidence: 0.45,
			reason: "trigger word +5",
			warnings: [],
		});

		expect(await listSkills()).toEqual([skillSummary]);
		expect(await getSkill("upsc_exam_preparation")).toMatchObject({
			id: "upsc_exam_preparation",
			prompt: { role: "Study coach" },
			safety: { requires_fresh_info: true },
			metadata: {
				audience: ["aspirant"],
				language_style: ["english"],
			},
			store_preview: {
				what_it_does: ["Create a study plan"],
				not_for: ["Official notice replacement"],
			},
		});
		expect(await testSkillRoute("make an upsc plan")).toMatchObject({
			primary_skill: { id: "upsc_exam_preparation" },
		});
	});

	test("setSkillEnabled sends exact payload", async () => {
		const calls: any[] = [];
		mockTauriCommand("skills_set_enabled", (args: any) => {
			calls.push(args);
			return null;
		});

		await setSkillEnabled("upsc_exam_preparation", true);
		await setSkillEnabled("upsc_exam_preparation", false);

		expect(calls).toEqual([
			{ skillId: "upsc_exam_preparation", enabled: true },
			{ skillId: "upsc_exam_preparation", enabled: false },
		]);
	});
});
