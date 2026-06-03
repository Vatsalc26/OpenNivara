import {
	Archive,
	ChevronRight,
	Clock,
	Laptop,
	MessageSquare,
	Search,
	Send,
	Sparkles,
	Terminal,
	X,
} from "lucide-react";
import { useState } from "react";
import type { Session } from "@/api/tauriClient";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";

interface SessionsListProps {
	sessions: Session[];
	activeSessionId: string | null;
	onSelectSession: (session: Session) => void;
}

export function SessionsList({
	sessions,
	activeSessionId,
	onSelectSession,
}: SessionsListProps) {
	const [searchTerm, setSearchTerm] = useState("");
	const [sourceFilter, setSourceFilter] = useState<
		"all" | "cli" | "telegram" | "desktop"
	>("all");

	const formatDate = (dateStr: string) => {
		try {
			const d = new Date(dateStr);
			return d.toLocaleDateString("en-US", {
				month: "short",
				day: "numeric",
				year: "numeric",
				hour: "2-digit",
				minute: "2-digit",
			});
		} catch {
			return dateStr;
		}
	};

	// Filter sessions by search term and source type
	const filteredSessions = sessions.filter((session) => {
		const titleMatch = (session.title || "New Conversation")
			.toLowerCase()
			.includes(searchTerm.toLowerCase());
		const idMatch = session.id.toLowerCase().includes(searchTerm.toLowerCase());
		const textMatch = titleMatch || idMatch;

		const source = (session.source_created || "").toLowerCase();
		const sourceMatch =
			sourceFilter === "all" ||
			(sourceFilter === "cli" && source.includes("cli")) ||
			(sourceFilter === "telegram" && source.includes("telegram")) ||
			(sourceFilter === "desktop" && source.includes("desktop"));

		return textMatch && sourceMatch;
	});

	const getSourceDetails = (source: string) => {
		const s = source.toLowerCase();
		if (s.includes("cli")) {
			return {
				label: "CLI Terminal",
				icon: Terminal,
				color: "text-cyan-400 border-cyan-500/20 bg-cyan-500/5",
			};
		} else if (s.includes("telegram")) {
			return {
				label: "Telegram Bot",
				icon: Send,
				color: "text-sky-400 border-sky-500/20 bg-sky-500/5",
			};
		} else {
			return {
				label: "Desktop App",
				icon: Laptop,
				color: "text-purple-400 border-purple-500/20 bg-purple-500/5",
			};
		}
	};

	return (
		<div className="flex flex-col h-full bg-background/20 p-6 space-y-4 overflow-hidden">
			{/* Page Header */}
			<div className="space-y-1 shrink-0">
				<h2 className="text-xl font-bold tracking-wide font-heading flex items-center gap-2">
					<MessageSquare className="h-5 w-5 text-primary" />
					<span>Conversation Sessions</span>
				</h2>
				<p className="text-xs text-muted-foreground leading-relaxed">
					Historical timeline of conversations synchronized from the SQLite
					sessions database.
				</p>
			</div>

			{/* Filter and Search Bar Row */}
			<div className="space-y-3 shrink-0 bg-secondary/10 p-4 rounded-xl border border-border/30">
				<div className="relative">
					<Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
					<input
						type="text"
						value={searchTerm}
						onChange={(e) => setSearchTerm(e.target.value)}
						placeholder="Search by title, description or session ID..."
						className="w-full bg-background/50 border border-border/50 rounded-lg pl-9 pr-9 py-2 text-xs text-zinc-200 focus:border-primary/50 outline-none transition-colors"
					/>
					{searchTerm && (
						<button
							onClick={() => setSearchTerm("")}
							className="absolute right-3 top-2.5 hover:text-foreground text-muted-foreground cursor-pointer transition-colors"
						>
							<X className="h-4 w-4" />
						</button>
					)}
				</div>

				{/* Source filtering tabs */}
				<div className="flex flex-wrap items-center gap-1.5 pt-1">
					<span className="text-[10px] font-extrabold text-muted-foreground uppercase tracking-wider mr-2 select-none">
						Filter Origin:
					</span>
					{[
						{ id: "all", label: "All Sources" },
						{ id: "cli", label: "CLI" },
						{ id: "telegram", label: "Telegram" },
						{ id: "desktop", label: "Desktop" },
					].map((tab) => {
						const isActive = sourceFilter === tab.id;
						return (
							<button
								key={tab.id}
								onClick={() => setSourceFilter(tab.id as any)}
								className={`px-3 py-1 rounded-md text-[10px] font-bold uppercase tracking-wider transition-all border cursor-pointer ${
									isActive
										? "bg-primary/10 text-primary border-primary/20"
										: "text-muted-foreground hover:text-foreground border-transparent hover:bg-secondary/40"
								}`}
							>
								{tab.label}
							</button>
						);
					})}
				</div>
			</div>

			{/* Sessions list */}
			<ScrollArea className="flex-1 -mx-2 px-2">
				{filteredSessions.length === 0 ? (
					<div className="flex flex-col items-center justify-center py-20 text-center space-y-4">
						<div className="w-12 h-12 rounded-xl bg-muted/20 flex items-center justify-center border border-border/25">
							<Archive className="h-6 w-6 text-muted-foreground/60" />
						</div>
						<div className="space-y-1">
							<p className="text-xs text-zinc-300 font-bold uppercase tracking-wide">
								No Sessions Match Your Filters
							</p>
							<p className="text-[10px] text-muted-foreground leading-normal max-w-xs">
								Try modifying your text search query or switching the origin
								filter setting.
							</p>
						</div>
					</div>
				) : (
					<div className="space-y-3 pb-6">
						{filteredSessions.map((session) => {
							const isSelected = session.id === activeSessionId;
							const sourceMeta = getSourceDetails(session.source_created);
							const SourceIcon = sourceMeta.icon;

							return (
								<Card
									key={session.id}
									onClick={() => onSelectSession(session)}
									className={`p-4 cursor-pointer border-border/40 hover:border-border transition-all duration-300 flex items-center justify-between group select-none ${
										isSelected
											? "bg-secondary/45 border-primary/45 glow-effect"
											: "bg-secondary/15 hover:bg-secondary/25"
									}`}
								>
									<div className="flex items-start gap-4 flex-1 min-w-0">
										{/* Dynamic Icon showing Active or message bubble */}
										<div
											className={`h-9 w-9 rounded-lg flex items-center justify-center shrink-0 border ${
												isSelected
													? "bg-primary/10 border-primary/30 text-primary"
													: "bg-zinc-950/40 border-border/20 text-muted-foreground group-hover:text-foreground"
											} transition-colors`}
										>
											{isSelected ? (
												<Sparkles className="h-4.5 w-4.5 animate-pulse" />
											) : (
												<MessageSquare className="h-4.5 w-4.5" />
											)}
										</div>

										<div className="space-y-2 flex-1 min-w-0">
											{/* Session Title & Active status */}
											<div className="flex items-center flex-wrap gap-2">
												<span className="font-bold text-xs truncate tracking-wide text-zinc-100 group-hover:text-primary transition-colors">
													{session.title || "New Conversation"}
												</span>
												<Badge
													variant={session.active ? "outline" : "secondary"}
													className={`text-[8px] px-1.5 py-0 rounded font-extrabold uppercase tracking-wider ${
														session.active
															? "bg-emerald-500/10 text-emerald-400 border-emerald-500/25"
															: "bg-zinc-900 border-zinc-800 text-zinc-500"
													}`}
												>
													{session.active ? "Active" : "Closed"}
												</Badge>
											</div>

											{/* Info Metadata */}
											<div className="flex flex-wrap items-center gap-y-1.5 gap-x-4 text-[10px] text-zinc-400 font-semibold">
												<span className="flex items-center gap-1">
													<Clock className="h-3.5 w-3.5 text-zinc-500" />
													<span>Updated: {formatDate(session.updated_at)}</span>
												</span>

												<Badge
													variant="outline"
													className={`${sourceMeta.color} flex items-center gap-1 text-[8px] font-extrabold px-1.5 py-0.5 rounded`}
												>
													<SourceIcon className="h-3 w-3" />
													<span>{sourceMeta.label}</span>
												</Badge>
											</div>
										</div>
									</div>

									<ChevronRight className="h-4 w-4 text-muted-foreground group-hover:text-primary group-hover:translate-x-0.5 transition-all shrink-0 ml-4" />
								</Card>
							);
						})}
					</div>
				)}
			</ScrollArea>
		</div>
	);
}
