import { safeInvoke } from "./tauriBridge";

const invoke = safeInvoke;

import type {
	ApiKeyStatus,
	EngineResponse,
	FirstRunInput,
	FirstRunStatus,
} from "@/generated/backendTypes";
import { StyleSchema } from "./schemas";

export type AskResponse = EngineResponse;

export interface Session {
	id: string;
	title: string;
	created_at: string;
	updated_at: string;
	status: string;
	source_created: string;
	active: boolean;
}

export interface DbMessage {
	id: string;
	session_id: string;
	role: string;
	source: string;
	content: string;
	created_at: string;
	metadata_json: string | null;
}

export interface ToolSettings {
	enabled: boolean;
	requires_confirmation: boolean;
	max_bytes?: number;
}

export interface ToolsConfig {
	general: {
		enabled: boolean;
		max_tool_rounds: number;
		show_tool_activity: boolean;
	};
	paths: {
		allowed_roots: string[];
		blocked_patterns: string[];
	};
	tools: Record<string, ToolSettings>;
}

export async function tauriAskOpenNivara(
	message: string,
	sessionId?: string,
	uiSelectedSkillId?: string,
	pinSelectedSkill?: boolean,
): Promise<AskResponse> {
	return invoke<AskResponse>("ask_opennivara", {
		message,
		sessionId: sessionId ?? null,
		uiSelectedSkillId: uiSelectedSkillId ?? null,
		pinSelectedSkill: pinSelectedSkill ?? false,
	});
}

export async function tauriListSessions(): Promise<Session[]> {
	return invoke<Session[]>("list_sessions");
}

export async function tauriGetSessionMessages(
	sessionId: string,
): Promise<DbMessage[]> {
	return invoke<DbMessage[]>("get_session_messages", { sessionId });
}

export async function tauriListTools(): Promise<ToolsConfig> {
	return invoke<ToolsConfig>("list_tools");
}

export async function tauriGetToolsPath(): Promise<string> {
	return invoke<string>("get_tools_path");
}

export async function tauriGetProfilePath(): Promise<string> {
	return invoke<string>("get_profile_path");
}

export async function tauriGetPreferencesPath(): Promise<string> {
	return invoke<string>("get_preferences_path");
}

export async function tauriGetStylePath(): Promise<string> {
	return invoke<string>("get_style_path");
}

export async function tauriGetContextsPath(): Promise<string> {
	return invoke<string>("get_contexts_path");
}

export async function tauriGetTelegramPath(): Promise<string> {
	return invoke<string>("get_telegram_path");
}

export async function tauriMapSummary(): Promise<string> {
	return invoke<string>("map_summary");
}

export interface Profile {
	schema_version: number;
	identity: {
		display_name: string;
		full_name: string;
		gender: string;
		pronouns: string;
		date_of_birth: string;
		timezone: string;
	};
	location: {
		country: string;
		state_or_region: string;
		city: string;
		living_situation: string;
	};
	languages: {
		preferred_human_language: string;
		other_human_languages: string[];
	};
	technical: {
		coding_level: string;
		preferred_coding_languages: string[];
		current_os: string;
		main_editor: string;
		secondary_editor: string;
		terminal: string;
	};
	personal: {
		occupation_or_role: string;
		education_level: string;
		interests: string[];
	};
	privacy: {
		send_identity: boolean;
		send_location: boolean;
		send_gender: boolean;
		send_technical: boolean;
		send_personal: boolean;
	};
}

export interface Style {
	schema_version: number;
	communication: {
		tone: string;
		detail_level: string;
		use_examples: boolean;
		use_step_by_step: boolean;
		avoid_unexplained_jargon: boolean;
		ask_fewer_questions: boolean;
		prefer_actionable_answers: boolean;
	};
	coding: {
		show_simple_solution_first: boolean;
		explain_after_code: boolean;
		prefer_mvp_architecture: boolean;
		avoid_overengineering: boolean;
		use_beginner_comments: boolean;
	};
	formatting: {
		use_markdown: boolean;
		use_short_sections: boolean;
		include_next_step: boolean;
		avoid_long_walls_of_text: boolean;
	};
	behavior: {
		be_honest_about_uncertainty: boolean;
		do_not_pretend_to_have_done_things: boolean;
		do_not_reveal_private_context_unless_relevant: boolean;
	};
}

export interface PreferenceItem {
	item: string;
	strength: number;
}

export interface PreferenceSection {
	id: string;
	enabled: boolean;
	send_policy: string;
	description?: string;
	triggers: string[];
	required_any: string[];
	negative_triggers: string[];
	min_score: number;
	likes: PreferenceItem[];
	dislikes: PreferenceItem[];
	notes: string[];
}

export interface Preferences {
	schema_version: number;
	sections: PreferenceSection[];
}

export interface ContextEntry {
	id: string;
	enabled: boolean;
	kind: string;
	send_policy: string;
	title: string;
	summary: string;
	triggers: string[];
	required_any: string[];
	negative_triggers: string[];
	min_score: number;
	facts: string[];
	rules: string[];
}

export interface Contexts {
	schema_version: number;
	contexts: ContextEntry[];
}

export interface ContextPreview {
	profile_sent: string[];
	style_sent: string[];
	preferences_sent: string[];
	contexts_sent: string[];
	contexts_pinned: string[];
	contexts_not_sent: string[];
	selected_skills: Array<{
		id: string;
		pack_id?: string | null;
		name: string;
		score: number;
		reason: string;
		allowed_tools: string[];
		denied_tools: string[];
	}>;
	skill_candidates: Array<{
		id: string;
		name: string;
		score: number;
		accepted: boolean;
		reason: string;
	}>;
	skill_warnings: string[];
	warnings: string[];
	final_context_text: string;
	active_theme?: {
		id: string;
		name: string;
		ui_only: true;
	} | null;
}

export async function tauriGetProfile(): Promise<Profile> {
	return invoke<Profile>("get_profile");
}

export async function tauriSaveProfile(profile: Profile): Promise<void> {
	return invoke<void>("save_profile", { profile });
}

export async function tauriGetStyle(): Promise<Style> {
	const raw = await invoke<unknown>("get_style");
	try {
		return StyleSchema.parse(raw) as Style;
	} catch (err: any) {
		console.error("Style parsing failed:", err);
		throw new Error("Style config shape does not match frontend schema.");
	}
}

export async function tauriSaveStyle(style: Style): Promise<void> {
	return invoke<void>("save_style", { style });
}

export async function tauriGetPreferences(): Promise<Preferences> {
	return invoke<Preferences>("get_preferences");
}

export async function tauriSavePreferences(
	preferences: Preferences,
): Promise<void> {
	return invoke<void>("save_preferences", { preferences });
}

export async function tauriGetContexts(): Promise<Contexts> {
	return invoke<Contexts>("get_contexts");
}

export async function tauriSaveContexts(contexts: Contexts): Promise<void> {
	return invoke<void>("save_contexts", { contexts });
}

export async function tauriPreviewContextForMessage(
	message: string,
	sessionId?: string,
	uiSelectedSkillId?: string,
): Promise<ContextPreview> {
	return invoke<ContextPreview>("preview_context_for_message", {
		message,
		sessionId: sessionId ?? null,
		uiSelectedSkillId: uiSelectedSkillId ?? null,
	});
}

export async function tauriPinContext(
	sessionId: string,
	contextId: string,
): Promise<void> {
	return invoke<void>("pin_context", { sessionId, contextId });
}

export async function tauriUnpinContext(
	sessionId: string,
	contextId: string,
): Promise<void> {
	return invoke<void>("unpin_context", { sessionId, contextId });
}

export async function tauriPinSkill(
	sessionId: string,
	skillId: string,
): Promise<void> {
	return invoke<void>("pin_skill", { sessionId, skillId });
}

export async function tauriUnpinSkill(
	sessionId: string,
	skillId: string,
): Promise<void> {
	return invoke<void>("unpin_skill", { sessionId, skillId });
}

export async function tauriListPinnedSkills(
	sessionId: string,
): Promise<string[]> {
	return invoke<string[]>("list_pinned_skills", { sessionId });
}

export async function tauriCheckApiKey(): Promise<boolean> {
	return invoke<boolean>("check_api_key");
}

export async function tauriCheckGeminiKey(): Promise<ApiKeyStatus> {
	return invoke<ApiKeyStatus>("check_gemini_key");
}

export async function tauriSaveGeminiKey(secret: string): Promise<void> {
	return invoke<void>("save_gemini_key", { secret });
}

export async function tauriFirstRunStatus(): Promise<FirstRunStatus> {
	return invoke<FirstRunStatus>("first_run_status");
}

export async function tauriInitializeCleanFirstRun(
	input: FirstRunInput,
): Promise<FirstRunStatus> {
	return invoke<FirstRunStatus>("initialize_clean_first_run", { input });
}

export type { ApiKeyStatus, FirstRunInput, FirstRunStatus };
