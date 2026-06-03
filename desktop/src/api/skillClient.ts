import { z } from "zod";
import {
	RouteDecisionSchema,
	SkillManifestSchema,
	SkillSummarySchema,
} from "./skillSchemas";
import { safeInvoke } from "./tauriBridge";

const invoke = safeInvoke;

export type SkillSummary = z.infer<typeof SkillSummarySchema>;
export type SkillManifest = z.infer<typeof SkillManifestSchema>;
export type RouteDecision = z.infer<typeof RouteDecisionSchema>;

export async function listSkills(): Promise<SkillSummary[]> {
	const data = await invoke("skills_list");
	return z.array(SkillSummarySchema).parse(data);
}

export async function getSkill(skillId: string): Promise<SkillManifest> {
	const data = await invoke("skills_get", { skillId });
	return SkillManifestSchema.parse(data);
}

export async function setSkillEnabled(
	skillId: string,
	enabled: boolean,
): Promise<void> {
	await invoke("skills_set_enabled", { skillId, enabled });
}

export async function testSkillRoute(message: string): Promise<RouteDecision> {
	const data = await invoke("skills_test_route", { message });
	return RouteDecisionSchema.parse(data);
}
