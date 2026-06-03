import type {
	ContextCompilerInput,
	ContextCompilerOutput,
	CreateSavedPlace,
	MemoryExtractionProposal,
	MemoryGraphContext,
	MemoryGraphStatus,
	MemoryItem,
	MemorySearchQuery,
	MemorySearchResult,
	MemorySettings,
	MemoryStatus,
	MemoryTask,
	RuntimeContext,
	SavedPlace,
} from "@/generated/backendTypes";
import {
	ContextCompilerOutputSchema,
	MemoryExtractionProposalSchema,
	MemoryGraphContextSchema,
	MemoryGraphStatusSchema,
	MemoryItemSchema,
	MemorySearchResultSchema,
	MemorySettingsSchema,
	MemoryStatusSchema,
	RuntimeContextSchema,
	SavedPlaceSchema,
} from "./memorySchemas";
import { safeInvoke } from "./tauriBridge";

export async function getMemoryStatus(): Promise<MemoryStatus> {
	const raw = await safeInvoke<unknown>("memory_status");
	return MemoryStatusSchema.parse(raw) as MemoryStatus;
}

export async function getMemorySettings(): Promise<MemorySettings> {
	const raw = await safeInvoke<unknown>("memory_get_settings");
	return MemorySettingsSchema.parse(raw) as MemorySettings;
}

export async function saveMemorySettings(
	settings: MemorySettings,
): Promise<void> {
	await safeInvoke<void>("memory_save_settings", { settings });
}

export async function listMemoryItems(limit = 100): Promise<MemoryItem[]> {
	const raw = await safeInvoke<unknown[]>("memory_list_items", { limit });
	return raw.map((item) => MemoryItemSchema.parse(item) as MemoryItem);
}

export async function searchMemory(
	query: MemorySearchQuery,
): Promise<MemorySearchResult[]> {
	const raw = await safeInvoke<unknown[]>("memory_search", { query });
	return raw.map(
		(result) => MemorySearchResultSchema.parse(result) as MemorySearchResult,
	);
}

export async function getRuntimeContext(
	allowExactLocation = false,
): Promise<RuntimeContext> {
	const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
	const raw = await safeInvoke<unknown>("runtime_get_context", {
		timezone,
		allowExactLocation,
	});
	return RuntimeContextSchema.parse(raw) as RuntimeContext;
}

export async function listSavedPlaces(): Promise<SavedPlace[]> {
	const raw = await safeInvoke<unknown[]>("location_list_saved_places");
	return raw.map((place) => SavedPlaceSchema.parse(place) as SavedPlace);
}

export async function saveSavedPlace(
	input: CreateSavedPlace,
): Promise<SavedPlace> {
	const raw = await safeInvoke<unknown>("location_save_place", { input });
	return SavedPlaceSchema.parse(raw) as SavedPlace;
}

export async function deleteSavedPlace(placeId: string): Promise<void> {
	await safeInvoke<void>("location_delete_saved_place", { placeId });
}

export async function getMemoryGraphStatus(): Promise<MemoryGraphStatus> {
	const raw = await safeInvoke<unknown>("memory_graph_status");
	return MemoryGraphStatusSchema.parse(raw) as MemoryGraphStatus;
}

export async function rebuildMemoryGraph(): Promise<MemoryGraphStatus> {
	const raw = await safeInvoke<unknown>("memory_graph_rebuild");
	return MemoryGraphStatusSchema.parse(raw) as MemoryGraphStatus;
}

export async function getMemoryGraphContext(
	memoryId: string,
	maxDepth = 2,
): Promise<MemoryGraphContext> {
	const raw = await safeInvoke<unknown>("memory_graph_memory_context", {
		memoryId,
		maxDepth,
	});
	return MemoryGraphContextSchema.parse(raw) as MemoryGraphContext;
}

export async function deleteMemoryItem(memoryId: string): Promise<void> {
	await safeInvoke<void>("memory_delete_item", { memoryId });
}

export async function retractMemoryItem(
	memoryId: string,
	reason: string,
): Promise<void> {
	await safeInvoke<void>("memory_retract_item", { memoryId, reason });
}

export async function listMemoryProposals(): Promise<
	MemoryExtractionProposal[]
> {
	const raw = await safeInvoke<unknown[]>("memory_list_proposals");
	return raw.map(
		(proposal) =>
			MemoryExtractionProposalSchema.parse(
				proposal,
			) as MemoryExtractionProposal,
	);
}

export async function extractMemoryProposalsForMessage(
	message: string,
): Promise<MemoryExtractionProposal[]> {
	const raw = await safeInvoke<unknown[]>(
		"memory_extract_proposals_for_message",
		{ message, sessionId: null, mode: null },
	);
	return raw.map(
		(proposal) =>
			MemoryExtractionProposalSchema.parse(
				proposal,
			) as MemoryExtractionProposal,
	);
}

export async function approveMemoryProposal(proposalId: string): Promise<void> {
	await safeInvoke<void>("memory_approve_proposal", { proposalId });
}

export async function rejectMemoryProposal(proposalId: string): Promise<void> {
	await safeInvoke<void>("memory_reject_proposal", { proposalId });
}

export async function listMemoryTasks(status?: string): Promise<MemoryTask[]> {
	return safeInvoke<MemoryTask[]>("memory_list_tasks", {
		status: status ?? null,
	});
}

export async function updateMemoryTaskStatus(
	memoryId: string,
	status: string,
): Promise<void> {
	await safeInvoke<void>("memory_update_task_status", { memoryId, status });
}

export async function compileMemoryContext(
	input: ContextCompilerInput,
): Promise<ContextCompilerOutput> {
	const raw = await safeInvoke<unknown>("memory_compile_context", { input });
	return ContextCompilerOutputSchema.parse(
		raw,
	) as unknown as ContextCompilerOutput;
}
