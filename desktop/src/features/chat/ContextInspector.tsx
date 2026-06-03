import { useMachine } from "@xstate/react";
import {
	AlertCircle,
	BookOpen,
	CheckCircle2,
	Eye,
	Palette,
	Pin,
	RefreshCw,
	Sparkles,
	Terminal,
	Trash2,
	User,
	X,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import {
	type Contexts,
	getContexts,
	pinContext,
	previewContextForMessage,
	unpinContext,
} from "@/api/opennivaraClient";
import { Card } from "@/components/ui/card";
import { contextInspectorMachine } from "./contextInspectorMachine";

interface ContextInspectorProps {
	sessionId: string | null;
	currentInputText?: string;
	onClose: () => void;
}

type InspectorTab = "summary" | "contexts" | "skills" | "prompt";

export function ContextInspector({
	sessionId,
	currentInputText = "",
	onClose,
}: ContextInspectorProps) {
	const [testQuery, setTestQuery] = useState<string>("");
	const [allContexts, setAllContexts] = useState<Contexts | null>(null);
	const [activeTab, setActiveTab] = useState<InspectorTab>("summary");

	const [state, send] = useMachine(contextInspectorMachine);
	const isInitialized = useRef(false);

	const fetchAllContexts = async () => {
		try {
			const data = await getContexts();
			setAllContexts(data);
		} catch (err) {
			console.error("Failed to load all contexts list in inspector", err);
		}
	};

	// XState effect handler for resolving preview
	useEffect(() => {
		if (state.value !== "loading") return;

		let active = true;
		const reqId = state.context.requestId;
		const query = state.context.query || testQuery || "hello";

		// 8 seconds timeout protection
		const timer = setTimeout(() => {
			if (active) {
				send({ type: "TIMEOUT", requestId: reqId });
			}
		}, 8000);

		previewContextForMessage(query, sessionId || undefined)
			.then((res) => {
				if (active) {
					clearTimeout(timer);
					send({ type: "RESOLVE", preview: res, requestId: reqId });
				}
			})
			.catch((err) => {
				if (active) {
					clearTimeout(timer);
					send({
						type: "REJECT",
						error: err?.message || String(err),
						requestId: reqId,
					});
				}
			});

		return () => {
			active = false;
			clearTimeout(timer);
		};
	}, [
		state.value,
		state.context.requestId,
		state.context.query,
		sessionId,
		send,
		testQuery,
	]);

	// Sync test query and run initial evaluation on mount
	useEffect(() => {
		if (!isInitialized.current) {
			fetchAllContexts();
			const defaultQuery = currentInputText.trim() || "hello";
			setTestQuery(defaultQuery);
			send({ type: "RUN", query: defaultQuery });
			isInitialized.current = true;
		}
	}, [currentInputText, send]);

	const handleRefresh = () => {
		send({ type: "REFRESH" });
		fetchAllContexts();
	};

	const handlePin = async (contextId: string) => {
		if (!sessionId) {
			toast.error("You must start a chat session first to pin a context!");
			return;
		}
		try {
			await pinContext(sessionId, contextId);
			toast.success(`Context '${contextId}' pinned to active session!`);
			handleRefresh();
		} catch (err: any) {
			toast.error(`Failed to pin context: ${err?.message || err}`);
		}
	};

	const handleUnpin = async (contextId: string) => {
		if (!sessionId) return;
		try {
			await unpinContext(sessionId, contextId);
			toast.success(`Context '${contextId}' unpinned from session.`);
			handleRefresh();
		} catch (err: any) {
			toast.error(`Failed to unpin context: ${err?.message || err}`);
		}
	};

	const preview = state.context.preview;
	const error = state.context.error;
	const isLoading = state.value === "loading";

	return (
		<div className="w-96 flex flex-col h-full bg-secondary/15 border-l border-border/30 backdrop-blur-lg shrink-0 select-none">
			{/* Header */}
			<div className="p-4 border-b border-border/20 flex items-center justify-between shrink-0 bg-secondary/10">
				<div className="flex items-center gap-2">
					<Eye className="h-4 w-4 text-primary" />
					<span className="font-bold text-xs uppercase tracking-wider text-foreground">
						Context Inspector
					</span>
				</div>
				<div className="flex items-center gap-1.5">
					<button
						onClick={handleRefresh}
						disabled={isLoading}
						className="h-7 w-7 rounded-lg hover:bg-secondary/40 text-muted-foreground hover:text-foreground flex items-center justify-center transition-colors cursor-pointer disabled:opacity-50"
						title="Refresh Preview"
					>
						<RefreshCw
							className={`h-3.5 w-3.5 ${isLoading ? "animate-spin" : ""}`}
						/>
					</button>
					<button
						onClick={onClose}
						className="h-7 w-7 rounded-lg hover:bg-secondary/40 text-muted-foreground hover:text-foreground flex items-center justify-center transition-colors cursor-pointer"
					>
						<X className="h-3.5 w-3.5" />
					</button>
				</div>
			</div>

			{/* Query Tester Block */}
			<div className="p-4 border-b border-border/15 shrink-0 space-y-2 bg-secondary/5">
				<label className="text-[10px] font-bold text-muted-foreground uppercase">
					Evaluate Test Prompt
				</label>
				<div className="flex gap-2">
					<input
						type="text"
						value={testQuery}
						onChange={(e) => setTestQuery(e.target.value)}
						onKeyDown={(e) =>
							e.key === "Enter" && send({ type: "RUN", query: testQuery })
						}
						placeholder="Type a test prompt to check boundaries..."
						className="flex-1 bg-background border border-border/40 rounded-lg px-2.5 py-1.5 text-xs focus:border-primary/50 outline-none"
					/>
					<button
						onClick={() => send({ type: "RUN", query: testQuery })}
						className="px-3 py-1.5 rounded-lg bg-primary hover:bg-primary/95 text-primary-foreground text-xs font-semibold uppercase tracking-wider transition-colors cursor-pointer shrink-0"
					>
						Go
					</button>
				</div>
			</div>

			{/* Sub tabs */}
			<div className="flex border-b border-border/15 px-4 py-1.5 gap-1 shrink-0 bg-secondary/5">
				{[
					{ id: "summary", label: "Profile / Prefs", icon: Sparkles },
					{ id: "contexts", label: "Goals & Pinned", icon: BookOpen },
					{ id: "skills", label: "Skills", icon: Sparkles },
					{ id: "prompt", label: "Raw Prompt", icon: Terminal },
				].map((tab) => {
					const Icon = tab.icon;
					const isActive = activeTab === tab.id;
					return (
						<button
							key={tab.id}
							onClick={() => setActiveTab(tab.id as InspectorTab)}
							className={`flex-1 flex items-center justify-center gap-1 py-1 rounded text-[10px] font-bold tracking-wide transition-all cursor-pointer ${
								isActive
									? "bg-primary/10 text-primary"
									: "text-muted-foreground hover:text-foreground"
							}`}
						>
							<Icon className="h-3.5 w-3.5" />
							<span>{tab.label}</span>
						</button>
					);
				})}
			</div>

			{/* Panels */}
			<div className="flex-1 overflow-y-auto p-4 space-y-4">
				{isLoading ? (
					<div className="flex flex-col items-center justify-center py-20 space-y-3">
						<div className="h-6 w-6 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
						<span className="text-[10px] text-muted-foreground font-semibold">
							Running selector evaluation...
						</span>
					</div>
				) : error ? (
					<div className="flex flex-col items-center justify-center py-16 space-y-3 text-center">
						<AlertCircle className="h-8 w-8 text-destructive" />
						<div className="space-y-1">
							<h5 className="text-xs font-bold text-foreground">
								Evaluation Failed
							</h5>
							<p className="text-[10px] text-muted-foreground max-w-[200px] leading-relaxed">
								{error}
							</p>
						</div>
						<button
							onClick={() => send({ type: "RUN", query: testQuery })}
							className="px-3 py-1 rounded bg-secondary hover:bg-secondary/80 text-foreground text-[10px] font-bold uppercase transition-all"
						>
							Retry
						</button>
					</div>
				) : preview ? (
					<>
						{activeTab === "summary" && (
							<div className="space-y-4">
								{/* UI-only active theme */}
								<Card className="p-3 bg-primary/5 border border-primary/20 space-y-2 animate-none">
									<h4 className="text-[10px] font-bold text-primary flex items-center gap-1.5 uppercase tracking-wide">
										<Sparkles className="h-3.5 w-3.5 text-primary" />
										<span>Active Visual Theme</span>
									</h4>
									<div className="text-[10px] text-muted-foreground leading-relaxed">
										<span className="font-bold text-foreground">
											{preview.active_theme?.name || "Default Theme"}
										</span>{" "}
										is UI only and is not sent to the model.
									</div>
								</Card>

								{/* Profile Sent Data */}
								<Card className="p-3 bg-secondary/15 border-border/30 space-y-2">
									<h4 className="text-[10px] font-bold text-primary flex items-center gap-1.5 uppercase">
										<User className="h-3.5 w-3.5" />
										<span>Transmitted Identity Facts</span>
									</h4>
									{preview.profile_sent.length === 0 ? (
										<div className="text-[10px] text-muted-foreground italic p-2 bg-background/25 rounded border border-dashed border-border/30">
											All identity facts omitted. Send toggles disabled.
										</div>
									) : (
										<ul className="text-[11px] text-muted-foreground space-y-1 pl-4.5 list-disc leading-relaxed">
											{preview.profile_sent.map((p, idx) => (
												<li key={idx}>{p}</li>
											))}
										</ul>
									)}
								</Card>

								{/* Style Guidelines */}
								<Card className="p-3 bg-secondary/15 border-border/30 space-y-2">
									<h4 className="text-[10px] font-bold text-primary flex items-center gap-1.5 uppercase">
										<Palette className="h-3.5 w-3.5" />
										<span>Response Styling Directives</span>
									</h4>
									<ul className="text-[11px] text-muted-foreground space-y-1 pl-4.5 list-disc leading-relaxed font-sans">
										{preview.style_sent.map((s, idx) => (
											<li key={idx}>{s}</li>
										))}
									</ul>
								</Card>

								{/* Topic Preferences Triggered */}
								<Card className="p-3 bg-secondary/15 border-border/30 space-y-2.5">
									<h4 className="text-[10px] font-bold text-primary flex items-center gap-1.5 uppercase">
										<Sparkles className="h-3.5 w-3.5" />
										<span>Triggered Preferences Likes</span>
									</h4>
									{preview.preferences_sent.length === 0 ? (
										<div className="text-[10px] text-muted-foreground italic p-2 bg-background/25 rounded border border-dashed border-border/30 leading-relaxed">
											No private preference sections triggered by keywords in: '
											{testQuery}'.
										</div>
									) : (
										<div className="space-y-1.5">
											{preview.preferences_sent.map((pref, idx) => (
												<div
													key={idx}
													className="bg-background/35 p-2 rounded border border-border/20 text-[11px] text-muted-foreground leading-relaxed"
												>
													{pref}
												</div>
											))}
										</div>
									)}
								</Card>
							</div>
						)}

						{activeTab === "contexts" && (
							<div className="space-y-4">
								{/* Pinned Contexts */}
								<Card className="p-3 bg-secondary/15 border-border/30 space-y-2">
									<h4 className="text-[10px] font-bold text-primary flex items-center gap-1.5 uppercase">
										<Pin className="h-3.5 w-3.5 text-primary rotate-45" />
										<span>Session-Pinned Contexts</span>
									</h4>
									{preview.contexts_pinned.length === 0 ? (
										<div className="text-[10px] text-muted-foreground italic p-2 bg-background/25 rounded border border-dashed border-border/30">
											No contexts pinned in the database for this session.
										</div>
									) : (
										<div className="space-y-1.5">
											{preview.contexts_pinned.map((id, idx) => (
												<div
													key={idx}
													className="flex items-center justify-between bg-primary/5 border border-primary/20 p-2 rounded text-[11px]"
												>
													<span className="font-bold text-foreground">
														{id}
													</span>
													<button
														onClick={() => handleUnpin(id)}
														className="text-[10px] text-red-400 hover:underline flex items-center gap-0.5"
													>
														<Trash2 className="h-3 w-3" />
														<span>Unpin</span>
													</button>
												</div>
											))}
										</div>
									)}
								</Card>

								{/* Sent Project & Learning Contexts */}
								<Card className="p-3 bg-secondary/15 border-border/30 space-y-2">
									<h4 className="text-[10px] font-bold text-primary flex items-center gap-1.5 uppercase">
										<CheckCircle2 className="h-3.5 w-3.5 text-emerald-400" />
										<span>Triggered & Always Sent Goals</span>
									</h4>
									{preview.contexts_sent.length === 0 ? (
										<div className="text-[10px] text-muted-foreground italic p-2 bg-background/25 rounded border border-dashed border-border/30 leading-relaxed">
											No learning/project context triggered. Prevents greeting
											bloat.
										</div>
									) : (
										<div className="space-y-1.5">
											{preview.contexts_sent.map((ctx, idx) => (
												<div
													key={idx}
													className="bg-emerald-500/5 border border-emerald-500/20 p-2.5 rounded text-[11px] text-muted-foreground leading-relaxed"
												>
													{ctx}
												</div>
											))}
										</div>
									)}
								</Card>

								{/* Pin triggers list */}
								<Card className="p-3 bg-secondary/15 border-border/30 space-y-2">
									<h4 className="text-[10px] font-bold text-primary flex items-center gap-1.5 uppercase">
										<Sparkles className="h-3.5 w-3.5" />
										<span>Quick-Pin Context entries</span>
									</h4>
									{allContexts?.contexts.length === 0 ? (
										<div className="text-[10px] text-muted-foreground italic p-2 rounded bg-background/20">
											No config contexts available.
										</div>
									) : (
										<div className="space-y-1.5">
											{allContexts?.contexts
												.filter((c) => !preview.contexts_pinned.includes(c.id))
												.map((c) => (
													<div
														key={c.id}
														className="flex items-center justify-between bg-background/30 p-2 rounded border border-border/20 text-[11px]"
													>
														<div className="space-y-0.5 flex-1 pr-3">
															<span className="font-bold text-foreground leading-none">
																{c.title}
															</span>
															<p className="text-[9px] text-muted-foreground font-mono">
																policy: {c.send_policy}
															</p>
														</div>
														<button
															onClick={() => handlePin(c.id)}
															className="text-[10px] text-primary hover:underline flex items-center gap-0.5 font-bold uppercase shrink-0"
														>
															<Pin className="h-3 w-3 rotate-45" />
															<span>Pin</span>
														</button>
													</div>
												))}
										</div>
									)}
								</Card>
							</div>
						)}

						{activeTab === "skills" && (
							<div className="space-y-4">
								<Card className="p-3 bg-secondary/15 border-border/30 space-y-2">
									<h4 className="text-[10px] font-bold text-primary flex items-center gap-1.5 uppercase">
										<Sparkles className="h-3.5 w-3.5" />
										<span>Selected Skills</span>
									</h4>
									{preview.selected_skills.length === 0 ? (
										<div className="text-[10px] text-muted-foreground italic p-2 bg-background/25 rounded border border-dashed border-border/30">
											No skill selected for this message.
										</div>
									) : (
										<div className="space-y-2">
											{preview.selected_skills.map((skill) => (
												<div
													key={skill.id}
													className="rounded-lg border border-primary/20 bg-primary/5 p-2 text-[11px]"
												>
													<div className="font-bold text-foreground">
														{skill.name} ({skill.score})
													</div>
													<div className="text-[10px] text-muted-foreground">
														{skill.reason}
													</div>
													<div className="mt-1 text-[10px] text-muted-foreground">
														Tools:{" "}
														{skill.allowed_tools.length
															? skill.allowed_tools.join(", ")
															: "none"}
													</div>
												</div>
											))}
										</div>
									)}
								</Card>

								<Card className="p-3 bg-secondary/15 border-border/30 space-y-2">
									<h4 className="text-[10px] font-bold text-primary flex items-center gap-1.5 uppercase">
										<Sparkles className="h-3.5 w-3.5" />
										<span>Route Candidates</span>
									</h4>
									{preview.skill_candidates.length === 0 ? (
										<div className="text-[10px] text-muted-foreground italic p-2 bg-background/25 rounded border border-dashed border-border/30">
											No candidate skills matched.
										</div>
									) : (
										<div className="space-y-1.5">
											{preview.skill_candidates.map((candidate) => (
												<div
													key={candidate.id}
													className="rounded border border-border/20 bg-background/30 p-2 text-[10px] text-muted-foreground"
												>
													<span className="font-bold text-foreground">
														{candidate.id}
													</span>{" "}
													score {candidate.score}, accepted{" "}
													{candidate.accepted ? "yes" : "no"}
													<div>{candidate.reason || "No route reason."}</div>
												</div>
											))}
										</div>
									)}
								</Card>
							</div>
						)}

						{activeTab === "prompt" && (
							<div className="space-y-3 h-full flex flex-col">
								<div className="flex items-center justify-between shrink-0">
									<label className="text-[10px] font-bold text-muted-foreground uppercase">
										Effective Prompt
									</label>
									<button
										onClick={() => {
											navigator.clipboard.writeText(preview.final_context_text);
											toast.success("Effective prompt copied to clipboard!");
										}}
										className="text-[10px] text-primary hover:underline font-bold uppercase cursor-pointer"
									>
										Copy All
									</button>
								</div>
								<div className="flex-1 bg-background/60 p-3 rounded-lg border border-border/30 overflow-y-auto text-[10px] font-mono leading-relaxed text-emerald-400/90 max-h-[420px] whitespace-pre-wrap select-text">
									{preview.final_context_text}
								</div>
							</div>
						)}
					</>
				) : (
					<div className="text-center py-20 text-muted-foreground text-xs italic">
						Enter test query above and click evaluation.
					</div>
				)}
			</div>

			{/* Warnings / Diagnostics */}
			{preview?.warnings && preview.warnings.length > 0 && (
				<div className="p-3 bg-red-500/10 border-t border-red-500/20 shrink-0 flex items-start gap-2.5">
					<AlertCircle className="h-4 w-4 text-red-400 shrink-0 mt-0.5" />
					<div className="space-y-1 leading-none">
						<span className="text-[10px] font-bold text-red-400 uppercase tracking-wide">
							Inspector Warnings
						</span>
						<p className="text-[10px] text-muted-foreground leading-relaxed">
							{preview.warnings.join(", ")}
						</p>
					</div>
				</div>
			)}
		</div>
	);
}
