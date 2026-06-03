import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { LucideIcon } from "lucide-react";
import {
	Brain,
	CheckCircle2,
	Clock,
	Database,
	MapPin,
	Network,
	RefreshCw,
	Search,
	Shield,
	Sparkles,
	Trash2,
	XCircle,
} from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";
import {
	approveMemoryProposal,
	compileMemoryContext,
	deleteMemoryItem,
	deleteSavedPlace,
	extractMemoryProposalsForMessage,
	getMemoryGraphContext,
	getMemoryGraphStatus,
	getMemorySettings,
	getMemoryStatus,
	getRuntimeContext,
	listMemoryItems,
	listMemoryProposals,
	listMemoryTasks,
	listSavedPlaces,
	rebuildMemoryGraph,
	rejectMemoryProposal,
	retractMemoryItem,
	saveMemorySettings,
	saveSavedPlace,
	searchMemory,
	updateMemoryTaskStatus,
} from "@/api/memoryClient";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import type {
	ContextCompilerOutput,
	CreateSavedPlace,
	MemoryGraphContext,
	MemoryGraphStatus,
	MemoryItem,
	MemoryMode,
	MemorySearchResult,
	MemorySettings,
	SavedPlace,
} from "@/generated/backendTypes";

type MemoryTab =
	| "timeline"
	| "search"
	| "review"
	| "tasks"
	| "graph"
	| "places"
	| "privacy";
type BoolSettingKey =
	| "pause_memory"
	| "private_chat"
	| "allow_location_memories"
	| "sensitive_approval_required";

const tabFromRoute: Record<string, MemoryTab> = {
	timeline: "timeline",
	search: "search",
	review: "review",
	people: "timeline",
	tasks: "tasks",
	graph: "graph",
	places: "places",
	privacy: "privacy",
};

interface MemoryViewProps {
	defaultTab?: string;
}

export function MemoryView({ defaultTab = "timeline" }: MemoryViewProps) {
	const queryClient = useQueryClient();
	const [activeTab, setActiveTab] = useState<MemoryTab>(
		tabFromRoute[defaultTab] ?? "timeline",
	);
	const [searchText, setSearchText] = useState("");
	const [proposalText, setProposalText] = useState("");
	const [compilerText, setCompilerText] = useState("did I buy bread Tuesday?");
	const [compilerOutput, setCompilerOutput] =
		useState<ContextCompilerOutput | null>(null);
	const [selectedMemoryId, setSelectedMemoryId] = useState<string | null>(null);

	const { data: status } = useQuery({
		queryKey: ["memory", "status"],
		queryFn: getMemoryStatus,
	});
	const { data: settings } = useQuery({
		queryKey: ["memory", "settings"],
		queryFn: getMemorySettings,
	});
	const { data: items = [] } = useQuery({
		queryKey: ["memory", "items"],
		queryFn: () => listMemoryItems(100),
	});
	const { data: proposals = [] } = useQuery({
		queryKey: ["memory", "proposals"],
		queryFn: listMemoryProposals,
	});
	const { data: tasks = [] } = useQuery({
		queryKey: ["memory", "tasks"],
		queryFn: () => listMemoryTasks(),
	});
	const { data: places = [] } = useQuery({
		queryKey: ["memory", "places"],
		queryFn: listSavedPlaces,
	});
	const { data: graphStatus } = useQuery({
		queryKey: ["memory", "graph", "status"],
		queryFn: getMemoryGraphStatus,
	});
	const graphContextQuery = useQuery({
		queryKey: ["memory", "graph", "context", selectedMemoryId],
		queryFn: () => getMemoryGraphContext(selectedMemoryId as string),
		enabled: activeTab === "graph" && Boolean(selectedMemoryId),
	});

	const searchQuery = useQuery({
		queryKey: ["memory", "search", searchText],
		queryFn: () =>
			searchMemory({
				query: searchText.trim() || null,
				memory_type: null,
				status: null,
				domain: null,
				facet_type: null,
				label: null,
				limit: 20,
			}),
		enabled: activeTab === "search" && searchText.trim().length > 0,
	});

	const refreshMemory = () => {
		queryClient.invalidateQueries({ queryKey: ["memory"] });
	};

	const settingsMutation = useMutation({
		mutationFn: saveMemorySettings,
		onSuccess: () => {
			toast.success("Memory settings saved");
			refreshMemory();
		},
		onError: (err: any) => toast.error(err?.message || String(err)),
	});

	const deleteMutation = useMutation({
		mutationFn: deleteMemoryItem,
		onSuccess: () => {
			toast.success("Memory deleted");
			refreshMemory();
		},
		onError: (err: any) => toast.error(err?.message || String(err)),
	});

	const retractMutation = useMutation({
		mutationFn: (memoryId: string) =>
			retractMemoryItem(memoryId, "Retracted from Memory view"),
		onSuccess: () => {
			toast.success("Memory retracted");
			refreshMemory();
		},
		onError: (err: any) => toast.error(err?.message || String(err)),
	});

	const proposalMutation = useMutation({
		mutationFn: extractMemoryProposalsForMessage,
		onSuccess: (created) => {
			toast.success(`${created.length} proposal(s) extracted`);
			setProposalText("");
			refreshMemory();
		},
		onError: (err: any) => toast.error(err?.message || String(err)),
	});

	const approveMutation = useMutation({
		mutationFn: approveMemoryProposal,
		onSuccess: () => {
			toast.success("Proposal approved");
			refreshMemory();
		},
		onError: (err: any) => toast.error(err?.message || String(err)),
	});

	const rejectMutation = useMutation({
		mutationFn: rejectMemoryProposal,
		onSuccess: () => {
			toast.success("Proposal rejected");
			refreshMemory();
		},
		onError: (err: any) => toast.error(err?.message || String(err)),
	});

	const taskMutation = useMutation({
		mutationFn: ({
			memoryId,
			nextStatus,
		}: {
			memoryId: string;
			nextStatus: string;
		}) => updateMemoryTaskStatus(memoryId, nextStatus),
		onSuccess: () => {
			toast.success("Task updated");
			refreshMemory();
		},
		onError: (err: any) => toast.error(err?.message || String(err)),
	});

	const graphRebuildMutation = useMutation({
		mutationFn: rebuildMemoryGraph,
		onSuccess: () => {
			toast.success("Memory graph rebuilt");
			refreshMemory();
		},
		onError: (err: any) => toast.error(err?.message || String(err)),
	});

	const savePlaceMutation = useMutation({
		mutationFn: saveSavedPlace,
		onSuccess: () => {
			toast.success("Saved place added");
			refreshMemory();
		},
		onError: (err: any) => toast.error(err?.message || String(err)),
	});

	const deletePlaceMutation = useMutation({
		mutationFn: deleteSavedPlace,
		onSuccess: () => {
			toast.success("Saved place deleted");
			refreshMemory();
		},
		onError: (err: any) => toast.error(err?.message || String(err)),
	});

	const compilerMutation = useMutation({
		mutationFn: async () => {
			const runtimeContext = await getRuntimeContext(
				settings?.allow_location_memories ?? false,
			);
			return compileMemoryContext({
				user_message: compilerText,
				session_id: null,
				message_id: null,
				runtime_context: runtimeContext,
				model_context_limit: 1800,
				reserved_output_tokens: 400,
				privacy_mode: settings?.mode ?? "ask_before_saving",
				enabled_sources: ["memory", "tasks"],
				current_workspace_context: null,
				current_route_context: null,
				manual_context_overrides: [],
			});
		},
		onSuccess: setCompilerOutput,
		onError: (err: any) => toast.error(err?.message || String(err)),
	});

	const pendingProposals = useMemo(
		() => proposals.filter((proposal) => proposal.status === "pending"),
		[proposals],
	);

	return (
		<div className="h-full min-h-0 overflow-y-auto bg-background">
			<section className="border-b border-border/30 bg-secondary/10 px-6 py-5">
				<div className="flex flex-wrap items-start justify-between gap-4">
					<div className="space-y-2">
						<div className="flex items-center gap-2 text-primary">
							<Brain className="h-5 w-5" />
							<h1 className="text-lg font-bold tracking-tight text-foreground">
								Life Memory
							</h1>
						</div>
						<p className="max-w-2xl text-xs leading-relaxed text-muted-foreground">
							Local SQLite memory with explicit review, privacy controls, and
							auditable prompt inclusion. Stored memories stay out of prompts
							unless the compiler decides they are relevant.
						</p>
					</div>
					<div className="grid grid-cols-3 gap-2 text-xs">
						<Metric label="Items" value={status?.item_count ?? 0} />
						<Metric label="Review" value={status?.proposal_count ?? 0} />
						<Metric
							label="Vector"
							value={status?.vector_enabled ? "On" : "Off"}
						/>
					</div>
				</div>
				<div className="mt-3 flex items-center gap-2 text-[10px] text-muted-foreground">
					<Database className="h-3.5 w-3.5" />
					<span className="truncate">
						{status?.db_path ?? "Initializing memory database..."}
					</span>
				</div>
			</section>

			<Tabs
				value={activeTab}
				onValueChange={(value) => setActiveTab(value as MemoryTab)}
				className="p-5"
			>
				<TabsList className="w-full max-w-3xl">
					<TabsTrigger value="timeline">Timeline</TabsTrigger>
					<TabsTrigger value="search">Search</TabsTrigger>
					<TabsTrigger value="review">Review</TabsTrigger>
					<TabsTrigger value="tasks">Tasks</TabsTrigger>
					<TabsTrigger value="graph">Graph</TabsTrigger>
					<TabsTrigger value="places">Places</TabsTrigger>
					<TabsTrigger value="privacy">Privacy</TabsTrigger>
				</TabsList>

				<TabsContent value="timeline" className="mt-4">
					<div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_380px]">
						<div className="space-y-3">
							{items.length === 0 ? (
								<EmptyState
									icon={Brain}
									title="No stored memories yet"
									body="Approved proposals and explicit saves appear here."
								/>
							) : (
								items.map((item) => (
									<MemoryItemRow
										key={item.id}
										item={item}
										onRetract={() => retractMutation.mutate(item.id)}
										onDelete={() => deleteMutation.mutate(item.id)}
									/>
								))
							)}
						</div>
						<CompilerAuditPanel
							compilerText={compilerText}
							setCompilerText={setCompilerText}
							output={compilerOutput}
							isPending={compilerMutation.isPending}
							onRun={() => compilerMutation.mutate()}
						/>
					</div>
				</TabsContent>

				<TabsContent value="search" className="mt-4 space-y-4">
					<div className="flex max-w-2xl gap-2">
						<Input
							value={searchText}
							onChange={(event) => setSearchText(event.target.value)}
							placeholder="Search memories, tasks, plans..."
						/>
						<Button
							onClick={() => searchQuery.refetch()}
							disabled={!searchText.trim()}
						>
							<Search className="h-4 w-4" />
							Search
						</Button>
					</div>
					<SearchResults results={searchQuery.data ?? []} />
				</TabsContent>

				<TabsContent value="review" className="mt-4 space-y-4">
					<Card className="max-w-3xl p-4">
						<div className="mb-2 flex items-center gap-2 text-sm font-bold">
							<Sparkles className="h-4 w-4 text-primary" />
							Extract Review Proposal
						</div>
						<Textarea
							value={proposalText}
							onChange={(event) => setProposalText(event.target.value)}
							placeholder="Example: remind me to buy bread tomorrow"
							className="min-h-24"
						/>
						<div className="mt-3 flex justify-end">
							<Button
								onClick={() => proposalMutation.mutate(proposalText)}
								disabled={!proposalText.trim() || proposalMutation.isPending}
							>
								Create Proposal
							</Button>
						</div>
					</Card>
					<div className="grid gap-3">
						{pendingProposals.length === 0 ? (
							<EmptyState
								icon={CheckCircle2}
								title="Review queue is clear"
								body="Pending extraction proposals will wait here before becoming memory."
							/>
						) : (
							pendingProposals.map((proposal) => (
								<Card key={proposal.id} className="p-4">
									<div className="flex flex-wrap justify-between gap-3">
										<div className="min-w-0 space-y-2">
											<div className="text-xs font-bold text-foreground">
												{proposal.id}
											</div>
											<pre className="max-h-44 overflow-auto rounded-md bg-background/60 p-3 text-[11px] text-muted-foreground">
												{formatProposal(proposal.proposal_json)}
											</pre>
										</div>
										<div className="flex shrink-0 gap-2">
											<Button
												size="sm"
												onClick={() => approveMutation.mutate(proposal.id)}
											>
												<CheckCircle2 className="h-4 w-4" />
												Approve
											</Button>
											<Button
												size="sm"
												variant="outline"
												onClick={() => rejectMutation.mutate(proposal.id)}
											>
												<XCircle className="h-4 w-4" />
												Reject
											</Button>
										</div>
									</div>
								</Card>
							))
						)}
					</div>
				</TabsContent>

				<TabsContent value="tasks" className="mt-4 space-y-3">
					{tasks.length === 0 ? (
						<EmptyState
							icon={Clock}
							title="No memory tasks"
							body="Task memories appear here with due dates and completion state."
						/>
					) : (
						tasks.map((task) => (
							<Card
								key={task.memory_id}
								className="flex items-center justify-between gap-3 p-4"
							>
								<div>
									<div className="text-sm font-bold">{task.memory_id}</div>
									<div className="text-xs text-muted-foreground">
										{task.status} {task.due_at ? `- due ${task.due_at}` : ""}
									</div>
								</div>
								<Button
									size="sm"
									variant={task.status === "completed" ? "outline" : "default"}
									onClick={() =>
										taskMutation.mutate({
											memoryId: task.memory_id,
											nextStatus:
												task.status === "completed" ? "planned" : "completed",
										})
									}
								>
									{task.status === "completed" ? "Reopen" : "Complete"}
								</Button>
							</Card>
						))
					)}
				</TabsContent>

				<TabsContent value="graph" className="mt-4">
					<GraphPanel
						status={graphStatus}
						items={items}
						selectedMemoryId={selectedMemoryId}
						setSelectedMemoryId={setSelectedMemoryId}
						context={graphContextQuery.data}
						isLoadingContext={graphContextQuery.isLoading}
						isRebuilding={graphRebuildMutation.isPending}
						onRebuild={() => graphRebuildMutation.mutate()}
					/>
				</TabsContent>

				<TabsContent value="places" className="mt-4">
					<PlacesPanel
						places={places}
						onSave={(place) => savePlaceMutation.mutate(place)}
						onDelete={(placeId) => deletePlaceMutation.mutate(placeId)}
					/>
				</TabsContent>

				<TabsContent value="privacy" className="mt-4">
					<PrivacyPanel
						settings={settings}
						onSave={(next) => settingsMutation.mutate(next)}
					/>
				</TabsContent>
			</Tabs>
		</div>
	);
}

function Metric({ label, value }: { label: string; value: number | string }) {
	return (
		<div className="rounded-lg border border-border/30 bg-background/50 px-3 py-2 text-center">
			<div className="text-sm font-bold text-foreground">{value}</div>
			<div className="text-[10px] uppercase text-muted-foreground">{label}</div>
		</div>
	);
}

function MemoryItemRow({
	item,
	onRetract,
	onDelete,
}: {
	item: MemoryItem;
	onRetract: () => void;
	onDelete: () => void;
}) {
	return (
		<Card className="p-4">
			<div className="flex flex-wrap justify-between gap-3">
				<div className="min-w-0 space-y-1">
					<div className="flex flex-wrap items-center gap-2">
						<span className="rounded-md bg-primary/10 px-2 py-0.5 text-[10px] font-bold uppercase text-primary">
							{item.memory_type}
						</span>
						<span className="rounded-md bg-secondary px-2 py-0.5 text-[10px] font-bold uppercase text-muted-foreground">
							{item.status}
						</span>
					</div>
					<h3 className="text-sm font-bold text-foreground">{item.title}</h3>
					<p className="text-xs leading-relaxed text-muted-foreground">
						{item.summary}
					</p>
					<div className="text-[10px] text-muted-foreground">
						confidence {Math.round(item.confidence * 100)}% - observed{" "}
						{item.observed_at}
					</div>
				</div>
				<div className="flex shrink-0 gap-2">
					<Button size="sm" variant="outline" onClick={onRetract}>
						Retract
					</Button>
					<Button size="icon-sm" variant="destructive" onClick={onDelete}>
						<Trash2 className="h-4 w-4" />
					</Button>
				</div>
			</div>
		</Card>
	);
}

function SearchResults({ results }: { results: MemorySearchResult[] }) {
	if (results.length === 0) {
		return (
			<EmptyState
				icon={Search}
				title="No search results"
				body="FTS and structured filters will explain why a result was selected."
			/>
		);
	}
	return (
		<div className="grid gap-3">
			{results.map((result) => (
				<Card key={result.item.id} className="p-4">
					<div className="flex flex-wrap justify-between gap-3">
						<div>
							<div className="text-sm font-bold">{result.item.title}</div>
							<div className="text-xs text-muted-foreground">
								{result.item.summary}
							</div>
						</div>
						<div className="text-right text-[10px] uppercase text-muted-foreground">
							<div>{result.answerability}</div>
							<div>{result.reason}</div>
						</div>
					</div>
				</Card>
			))}
		</div>
	);
}

function GraphPanel({
	status,
	items,
	selectedMemoryId,
	setSelectedMemoryId,
	context,
	isLoadingContext,
	isRebuilding,
	onRebuild,
}: {
	status?: MemoryGraphStatus;
	items: MemoryItem[];
	selectedMemoryId: string | null;
	setSelectedMemoryId: (memoryId: string) => void;
	context?: MemoryGraphContext;
	isLoadingContext: boolean;
	isRebuilding: boolean;
	onRebuild: () => void;
}) {
	const effectiveMemoryId = selectedMemoryId ?? items[0]?.id ?? null;

	return (
		<div className="grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
			<Card className="p-4">
				<div className="mb-3 flex items-center justify-between gap-3">
					<div className="flex items-center gap-2 text-sm font-bold">
						<Network className="h-4 w-4 text-primary" />
						Graph Index
					</div>
					<Button
						size="sm"
						variant="outline"
						onClick={onRebuild}
						disabled={isRebuilding}
					>
						<RefreshCw
							className={`h-4 w-4 ${isRebuilding ? "animate-spin" : ""}`}
						/>
						Rebuild
					</Button>
				</div>
				<div className="grid grid-cols-3 gap-2">
					<Metric label="Nodes" value={status?.node_count ?? 0} />
					<Metric label="Edges" value={status?.edge_count ?? 0} />
					<Metric label="Index" value={status?.index_count ?? 0} />
				</div>
				{status?.validation_errors?.length ? (
					<div className="mt-3 rounded-md border border-destructive/30 bg-destructive/10 p-3 text-xs text-destructive">
						{status.validation_errors.join("\n")}
					</div>
				) : (
					<div className="mt-3 rounded-md border border-border/30 bg-background/50 p-3 text-xs text-muted-foreground">
						Graph consistency is clean for the indexed SQLite rows.
					</div>
				)}
				<div className="mt-4 space-y-2">
					<div className="text-xs font-bold text-foreground">Memory Focus</div>
					{items.slice(0, 12).map((item) => (
						<button
							key={item.id}
							onClick={() => setSelectedMemoryId(item.id)}
							className={`w-full rounded-md border px-3 py-2 text-left text-xs ${
								effectiveMemoryId === item.id
									? "border-primary bg-primary/10 text-primary"
									: "border-border/30 bg-background/35 text-muted-foreground"
							}`}
						>
							<div className="truncate font-bold">{item.title}</div>
							<div className="truncate">{item.id}</div>
						</button>
					))}
				</div>
			</Card>
			<Card className="p-4">
				<div className="mb-3 text-sm font-bold">Neighborhood</div>
				{!effectiveMemoryId ? (
					<EmptyState
						icon={Network}
						title="No memory selected"
						body="Add or approve a memory, rebuild the graph, then inspect its neighbors."
					/>
				) : isLoadingContext ? (
					<div className="text-xs text-muted-foreground">
						Loading graph context...
					</div>
				) : context ? (
					<div className="space-y-4">
						<div className="grid gap-2 sm:grid-cols-2">
							{context.nodes.map((node) => (
								<div
									key={node.id}
									className="rounded-md border border-border/30 bg-background/40 p-3 text-xs"
								>
									<div className="font-bold text-foreground">{node.label}</div>
									<div className="text-muted-foreground">
										{node.node_type} - {node.source_table}
									</div>
								</div>
							))}
						</div>
						<div className="space-y-2">
							<div className="text-xs font-bold text-foreground">Edges</div>
							{context.edges.length === 0 ? (
								<div className="text-xs text-muted-foreground">
									No graph edges are indexed for this memory yet.
								</div>
							) : (
								context.edges.map((edge) => (
									<div
										key={edge.id}
										className="rounded-md bg-secondary/20 px-3 py-2 text-[11px] text-muted-foreground"
									>
										{`${edge.edge_type}: ${edge.from_node_id} -> ${edge.to_node_id}`}
									</div>
								))
							)}
						</div>
					</div>
				) : (
					<EmptyState
						icon={Network}
						title="Graph context unavailable"
						body="Rebuild the graph index to derive nodes and edges from SQLite."
					/>
				)}
			</Card>
		</div>
	);
}

function PlacesPanel({
	places,
	onSave,
	onDelete,
}: {
	places: SavedPlace[];
	onSave: (place: CreateSavedPlace) => void;
	onDelete: (placeId: string) => void;
}) {
	const [label, setLabel] = useState("");
	const [city, setCity] = useState("");
	const [timezone, setTimezone] = useState(
		Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC",
	);

	const submit = () => {
		if (!label.trim()) return;
		onSave({
			label: label.trim(),
			place_type: "saved_place",
			latitude: null,
			longitude: null,
			address: null,
			city: city.trim() || null,
			region: null,
			country: null,
			timezone: timezone.trim() || null,
			details_json: "{}",
		});
		setLabel("");
		setCity("");
	};

	return (
		<div className="grid gap-4 lg:grid-cols-[360px_minmax(0,1fr)]">
			<Card className="p-4">
				<div className="mb-3 flex items-center gap-2 text-sm font-bold">
					<MapPin className="h-4 w-4 text-primary" />
					Saved Place
				</div>
				<div className="space-y-3">
					<Input
						value={label}
						onChange={(event) => setLabel(event.target.value)}
						placeholder="Home, Office, Gym..."
					/>
					<Input
						value={city}
						onChange={(event) => setCity(event.target.value)}
						placeholder="City"
					/>
					<Input
						value={timezone}
						onChange={(event) => setTimezone(event.target.value)}
						placeholder="Timezone"
					/>
					<Button onClick={submit} disabled={!label.trim()}>
						Save Place
					</Button>
				</div>
				<p className="mt-3 text-xs leading-relaxed text-muted-foreground">
					Saved places are local hints. Exact current location is included in
					prompts only when memory settings allow it and the compiler finds it
					relevant.
				</p>
			</Card>
			<div className="space-y-3">
				{places.length === 0 ? (
					<EmptyState
						icon={MapPin}
						title="No saved places"
						body="Add places for timezone and route context without needing live tracking."
					/>
				) : (
					places.map((place) => (
						<Card
							key={place.id}
							className="flex items-center justify-between gap-3 p-4"
						>
							<div className="min-w-0">
								<div className="text-sm font-bold text-foreground">
									{place.label}
								</div>
								<div className="text-xs text-muted-foreground">
									{[place.city, place.region, place.country]
										.filter(Boolean)
										.join(", ") || "No address details"}
								</div>
								<div className="text-[10px] text-muted-foreground">
									{place.timezone ?? "No timezone"} - {place.place_type}
								</div>
							</div>
							<Button
								size="sm"
								variant="outline"
								onClick={() => onDelete(place.id)}
							>
								Delete
							</Button>
						</Card>
					))
				)}
			</div>
		</div>
	);
}

function CompilerAuditPanel({
	compilerText,
	setCompilerText,
	output,
	isPending,
	onRun,
}: {
	compilerText: string;
	setCompilerText: (value: string) => void;
	output: ContextCompilerOutput | null;
	isPending: boolean;
	onRun: () => void;
}) {
	return (
		<Card className="h-fit p-4">
			<div className="mb-3 flex items-center justify-between gap-3">
				<div className="flex items-center gap-2 text-sm font-bold">
					<Shield className="h-4 w-4 text-primary" />
					Context Compiler Audit
				</div>
				<Button
					size="sm"
					variant="outline"
					onClick={onRun}
					disabled={isPending}
				>
					<RefreshCw className={`h-4 w-4 ${isPending ? "animate-spin" : ""}`} />
					Run
				</Button>
			</div>
			<Input
				value={compilerText}
				onChange={(event) => setCompilerText(event.target.value)}
				placeholder="Test a prompt against memory selection"
			/>
			{output && (
				<div className="mt-4 space-y-3 text-xs">
					<div className="rounded-md border border-border/30 bg-background/50 p-3">
						<div className="mb-1 font-bold text-foreground">Decision</div>
						<div className="text-muted-foreground">
							{output.intent.labels.join(", ")} - {output.intent.reason}
						</div>
						<div className="mt-2 text-[11px] text-muted-foreground">
							Runtime: {output.runtime_decision}
							<br />
							Location: {output.location_decision}
						</div>
					</div>
					<div className="rounded-md border border-border/30 bg-background/50 p-3">
						<div className="mb-1 font-bold text-foreground">Memory Brief</div>
						<pre className="max-h-40 overflow-auto whitespace-pre-wrap text-[11px] text-muted-foreground">
							{output.memory_brief || "No memory included."}
						</pre>
					</div>
					<div className="grid grid-cols-3 gap-2">
						<Metric
							label="Context"
							value={output.token_budget_report.estimated_prompt_tokens}
						/>
						<Metric
							label="Limit"
							value={output.token_budget_report.model_context_limit}
						/>
						<Metric label="IDs" value={output.included_memory_ids.length} />
					</div>
					<div className="rounded-md border border-border/30 bg-background/50 p-3">
						<div className="mb-1 font-bold text-foreground">Graph Edges</div>
						<div className="text-[11px] text-muted-foreground">
							{output.included_graph_edge_ids.length > 0
								? output.included_graph_edge_ids.join(", ")
								: "No graph edges included."}
						</div>
					</div>
					<details className="rounded-md border border-border/30 bg-background/50 p-3">
						<summary className="cursor-pointer text-xs font-bold text-foreground">
							Audit JSON
						</summary>
						<pre className="mt-2 max-h-40 overflow-auto whitespace-pre-wrap text-[11px] text-muted-foreground">
							{formatProposal(output.audit.compiled_context_json)}
						</pre>
					</details>
				</div>
			)}
		</Card>
	);
}

function PrivacyPanel({
	settings,
	onSave,
}: {
	settings?: MemorySettings;
	onSave: (settings: MemorySettings) => void;
}) {
	const [draft, setDraft] = useState<MemorySettings | null>(null);
	const active = draft ?? settings ?? defaultSettings();

	const setMode = (mode: MemoryMode) => {
		setDraft({ ...active, mode });
	};

	const toggle = (key: BoolSettingKey) => {
		setDraft({ ...active, [key]: !active[key] });
	};

	return (
		<Card className="max-w-3xl p-4">
			<div className="mb-4 flex items-center gap-2">
				<Shield className="h-4 w-4 text-primary" />
				<h2 className="text-sm font-bold">Memory Privacy</h2>
			</div>
			<div className="grid gap-3 sm:grid-cols-2">
				{[
					["off", "Off"],
					["ask_before_saving", "Ask before saving"],
					["auto_save_low_risk", "Auto-save low risk"],
					["full_life_journal", "Full life journal"],
				].map(([mode, label]) => (
					<button
						key={mode}
						onClick={() => setMode(mode as MemoryMode)}
						className={`rounded-lg border p-3 text-left text-xs transition-colors ${
							active.mode === mode
								? "border-primary bg-primary/10 text-primary"
								: "border-border/40 bg-background/40 text-muted-foreground hover:text-foreground"
						}`}
					>
						{label}
					</button>
				))}
			</div>
			<div className="mt-4 grid gap-2 text-xs">
				<ToggleRow
					label="Pause all memory activity"
					checked={active.pause_memory}
					onClick={() => toggle("pause_memory")}
				/>
				<ToggleRow
					label="Private chat: never include or save"
					checked={active.private_chat}
					onClick={() => toggle("private_chat")}
				/>
				<ToggleRow
					label="Allow location memories"
					checked={active.allow_location_memories}
					onClick={() => toggle("allow_location_memories")}
				/>
				<ToggleRow
					label="Require approval for sensitive memories"
					checked={active.sensitive_approval_required}
					onClick={() => toggle("sensitive_approval_required")}
				/>
			</div>
			<div className="mt-4 flex justify-end">
				<Button onClick={() => onSave(active)}>Save Privacy Settings</Button>
			</div>
		</Card>
	);
}

function ToggleRow({
	label,
	checked,
	onClick,
}: {
	label: string;
	checked: boolean;
	onClick: () => void;
}) {
	return (
		<button
			onClick={onClick}
			className="flex items-center justify-between rounded-lg border border-border/30 bg-background/35 px-3 py-2 text-left"
		>
			<span>{label}</span>
			<span
				className={`rounded-full px-2 py-0.5 text-[10px] font-bold uppercase ${
					checked
						? "bg-primary/10 text-primary"
						: "bg-secondary text-muted-foreground"
				}`}
			>
				{checked ? "On" : "Off"}
			</span>
		</button>
	);
}

function EmptyState({
	icon: Icon,
	title,
	body,
}: {
	icon: LucideIcon;
	title: string;
	body: string;
}) {
	return (
		<div className="rounded-lg border border-dashed border-border/40 bg-secondary/10 p-8 text-center">
			<Icon className="mx-auto mb-3 h-6 w-6 text-muted-foreground" />
			<div className="text-sm font-bold text-foreground">{title}</div>
			<div className="mt-1 text-xs text-muted-foreground">{body}</div>
		</div>
	);
}

function formatProposal(json: string) {
	try {
		return JSON.stringify(JSON.parse(json), null, 2);
	} catch {
		return json;
	}
}

function defaultSettings(): MemorySettings {
	return {
		schema_version: 1,
		mode: "ask_before_saving",
		pause_memory: false,
		private_chat: false,
		allow_location_memories: false,
		sensitive_approval_required: true,
	};
}
