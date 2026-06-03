import {
	AlertCircle,
	ArrowRight,
	BookOpen,
	Check,
	Clock,
	Copy,
	Database,
	FileCode,
	FileText,
	FolderOpen,
	Settings,
	Sparkles,
	Terminal,
} from "lucide-react";
import type React from "react";
import { useState } from "react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";

interface WorkspaceMapViewProps {
	summary: string | null;
	isLoading: boolean;
}

export function WorkspaceMapView({
	summary,
	isLoading,
}: WorkspaceMapViewProps) {
	const [copiedCommand, setCopiedCommand] = useState(false);

	if (isLoading) {
		return (
			<div className="flex flex-col items-center justify-center h-full text-center space-y-4">
				<Database className="h-8 w-8 text-primary animate-spin" />
				<p className="text-sm text-muted-foreground font-semibold">
					Retrieving workspace index summary...
				</p>
			</div>
		);
	}

	// Parse lines of the text-rendered summary to present it beautifully in visual cards!
	const parseSummary = (text: string | null) => {
		if (!text) return null;

		const lines = text.split("\n");
		let rootPath = "Unknown";
		let lastScan = "Never";
		let totalFiles = "0";
		let totalFolders = "0";
		let blockedItems = "0";
		let ignoredItems = "0";
		const categories: { label: string; count: string }[] = [];
		let keyLandmarks = "";

		lines.forEach((line) => {
			const trimmed = line.trim();
			if (trimmed.startsWith("Root Directory:")) {
				rootPath = trimmed.replace("Root Directory:", "").trim();
			} else if (trimmed.startsWith("Last Scan Time:")) {
				lastScan = trimmed.replace("Last Scan Time:", "").trim();
			} else if (trimmed.startsWith("Total Files:")) {
				totalFiles = trimmed.replace("Total Files:", "").trim();
			} else if (trimmed.startsWith("Total Folders:")) {
				totalFolders = trimmed.replace("Total Folders:", "").trim();
			} else if (trimmed.startsWith("Blocked Items:")) {
				blockedItems = trimmed.replace("Blocked Items:", "").trim();
			} else if (trimmed.startsWith("Ignored Items:")) {
				ignoredItems = trimmed.replace("Ignored Items:", "").trim();
			} else if (trimmed.startsWith("Key Landmarks Identified:")) {
				keyLandmarks = trimmed.replace("Key Landmarks Identified:", "").trim();
			} else if (
				trimmed &&
				!trimmed.startsWith("===") &&
				!trimmed.startsWith("Category Breakdown:") &&
				!trimmed.startsWith("Key Landmarks") &&
				!trimmed.startsWith("=================")
			) {
				const parts = trimmed.split(":");
				if (parts.length >= 2) {
					categories.push({
						label: parts[0].trim(),
						count: parts[1].trim(),
					});
				}
			}
		});

		return {
			rootPath,
			lastScan,
			totalFiles,
			totalFolders,
			blockedItems,
			ignoredItems,
			categories,
			keyLandmarks,
		};
	};

	const parsed = parseSummary(summary);

	const handleCopyCommand = () => {
		navigator.clipboard.writeText("opennivara map-scan");
		setCopiedCommand(true);
		toast.success("Command copied to clipboard!");
		setTimeout(() => setCopiedCommand(false), 2000);
	};

	const getLandmarkMeta = (filename: string) => {
		const lower = filename.toLowerCase();
		if (lower.includes("cargo.toml")) {
			return {
				label: "Cargo Config",
				icon: Settings,
				color: "text-amber-400 border-amber-500/25 bg-amber-500/5",
				desc: "Defines package dependencies and crate settings.",
			};
		} else if (lower.includes("readme.md")) {
			return {
				label: "Documentation",
				icon: BookOpen,
				color: "text-emerald-400 border-emerald-500/25 bg-emerald-500/5",
				desc: "Introductory developer guide & project specs.",
			};
		} else if (lower.includes("main.rs") || lower.includes("lib.rs")) {
			return {
				label: "Rust Entrypoint",
				icon: FileCode,
				color: "text-cyan-400 border-cyan-500/25 bg-cyan-500/5",
				desc: "Main binary compiler or library code link.",
			};
		} else if (lower.includes("package.json")) {
			return {
				label: "Node Config",
				icon: Settings,
				color: "text-rose-400 border-rose-500/25 bg-rose-500/5",
				desc: "NPM script configurations & dependencies map.",
			};
		} else if (
			lower.endsWith(".rs") ||
			lower.endsWith(".ts") ||
			lower.endsWith(".js") ||
			lower.endsWith(".py")
		) {
			return {
				label: "Primary Source",
				icon: FileCode,
				color: "text-purple-400 border-purple-500/25 bg-purple-500/5",
				desc: "Core implementation logic files.",
			};
		} else {
			return {
				label: "Landmark Target",
				icon: FileText,
				color: "text-zinc-400 border-zinc-500/20 bg-zinc-500/5",
				desc: "Key reference checkpoint mapping landmark.",
			};
		}
	};

	const landmarkList = parsed?.keyLandmarks
		? parsed.keyLandmarks
				.split(",")
				.map((s) => s.trim())
				.filter(Boolean)
		: [];

	return (
		<div className="flex flex-col h-full bg-background/20 p-6 space-y-6 overflow-y-auto">
			{/* View Title */}
			<div className="space-y-1 shrink-0">
				<h2 className="text-xl font-bold tracking-wide font-heading flex items-center gap-2">
					<Database className="h-5 w-5 text-primary" />
					<span>Workspace SQLite Map</span>
				</h2>
				<p className="text-xs text-muted-foreground leading-relaxed">
					Aggregated analytics of directory structure scans stored inside your
					localized SQLite workspace database.
				</p>
			</div>

			{!parsed ? (
				<div className="flex-1 flex items-center justify-center p-4">
					<Card className="p-8 max-w-lg bg-secondary/15 border-border/40 text-center flex flex-col items-center justify-center space-y-6 glow-effect">
						<div className="w-14 h-14 rounded-full bg-amber-500/10 flex items-center justify-center border border-amber-500/20 shadow-inner">
							<AlertCircle className="h-7 w-7 text-amber-400" />
						</div>

						<div className="space-y-2 max-w-sm">
							<h3 className="font-bold text-base text-zinc-100 font-heading">
								Workspace SQLite Map Empty
							</h3>
							<p className="text-xs text-muted-foreground leading-normal">
								OpenNivara indexes files, folder structures, and key landmark
								items to assist in semantic search rounds. No map index was
								found.
							</p>
						</div>

						<div className="w-full bg-zinc-950/60 rounded-xl p-4 border border-zinc-900 space-y-4 text-left">
							<span className="text-[10px] font-extrabold uppercase tracking-widest text-zinc-500 block border-b border-zinc-900 pb-2">
								Scan Initiation Instructions
							</span>

							<ul className="space-y-3">
								{[
									"Open your target project directory in your terminal emulator.",
									"Run the OpenNivara CLI indexing command below.",
									"Once finished, refresh this page to inspect your project maps.",
								].map((step, idx) => (
									<li
										key={idx}
										className="flex gap-2 text-xs leading-relaxed text-zinc-300"
									>
										<span className="text-primary font-bold">{idx + 1}.</span>
										<span>{step}</span>
									</li>
								))}
							</ul>

							<div className="pt-2">
								<div className="flex items-center justify-between bg-muted/40 p-2.5 rounded-lg border border-border/30 font-mono text-xs select-all">
									<div className="flex items-center gap-2 text-primary font-bold">
										<Terminal className="h-4 w-4 shrink-0" />
										<span>opennivara map-scan</span>
									</div>
									<button
										onClick={handleCopyCommand}
										className="h-7 w-7 rounded bg-secondary hover:bg-secondary/85 text-zinc-400 flex items-center justify-center cursor-pointer transition-colors border border-border/35"
										title="Copy command"
									>
										{copiedCommand ? (
											<Check className="h-3.5 w-3.5 text-emerald-400" />
										) : (
											<Copy className="h-3.5 w-3.5" />
										)}
									</button>
								</div>
							</div>
						</div>
					</Card>
				</div>
			) : (
				<div className="space-y-6 pb-6">
					{/* Main metrics grid */}
					<div className="grid grid-cols-2 md:grid-cols-4 gap-4">
						{[
							{
								label: "Scanned Files",
								value: parsed.totalFiles,
								sub: "Total code records",
								iconColor: "text-cyan-400",
							},
							{
								label: "Scanned Folders",
								value: parsed.totalFolders,
								sub: "Directories indexed",
								iconColor: "text-purple-400",
							},
							{
								label: "Blocked Items",
								value: parsed.blockedItems,
								sub: "Safety boundaries",
								iconColor: "text-rose-400",
							},
							{
								label: "Ignored Matches",
								value: parsed.ignoredItems,
								sub: "Gitignore patterns",
								iconColor: "text-zinc-500",
							},
						].map((card, idx) => (
							<Card
								key={idx}
								className="p-4 bg-secondary/25 border-border/50 flex flex-col justify-between min-h-[90px] glow-effect"
							>
								<span className="text-[10px] text-muted-foreground font-bold uppercase tracking-wider">
									{card.label}
								</span>
								<div className="space-y-0.5 mt-2">
									<p className="text-2xl font-extrabold font-heading text-primary leading-none">
										{card.value}
									</p>
									<span className="text-[9px] text-muted-foreground/80 font-medium tracking-wide block">
										{card.sub}
									</span>
								</div>
							</Card>
						))}
					</div>

					{/* Anchor folder details card */}
					<Card className="p-4 bg-secondary/20 border-border/50 space-y-3.5">
						<div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 border-b border-border/20 pb-3">
							<div className="flex items-center gap-2">
								<FolderOpen className="h-4.5 w-4.5 text-primary shrink-0" />
								<span className="font-bold text-xs uppercase tracking-wider text-zinc-200">
									Anchor Workspace Directory
								</span>
							</div>
							<div className="flex items-center gap-1.5 text-[10px] font-semibold text-muted-foreground">
								<Clock className="h-3.5 w-3.5" />
								<span>Last Scanned: {parsed.lastScan}</span>
							</div>
						</div>
						<div className="text-xs font-mono bg-zinc-950/45 p-3 rounded-lg border border-border/20 truncate select-all leading-relaxed text-zinc-300">
							{parsed.rootPath}
						</div>
					</Card>

					{/* Landmarks visual grid section */}
					<div className="space-y-3">
						<div className="flex items-center justify-between border-b border-border/20 pb-2">
							<h3 className="font-bold text-xs uppercase tracking-wider text-zinc-200 flex items-center gap-2">
								<Sparkles className="h-4.5 w-4.5 text-primary shrink-0" />
								<span>Key Workspace Landmarks ({landmarkList.length})</span>
							</h3>
							<Badge
								variant="outline"
								className="text-[9px] font-bold border-emerald-500/30 text-emerald-400 bg-emerald-500/5 px-2 py-0.5 rounded-full select-none"
							>
								Auto-Discovered
							</Badge>
						</div>
						<p className="text-[10px] text-muted-foreground leading-normal">
							Primary project structures detected to serve as immediate context
							guides for LLM planning pipelines.
						</p>

						{landmarkList.length === 0 ? (
							<Card className="p-6 text-center bg-secondary/10 border-border/30 text-xs text-muted-foreground italic">
								No key landmarks discovered in database scan.
							</Card>
						) : (
							<div className="grid grid-cols-1 md:grid-cols-3 gap-4">
								{landmarkList.map((filename, i) => {
									const meta = getLandmarkMeta(filename);
									const Icon = meta.icon;
									const handleCopyLandmark = (e: React.MouseEvent) => {
										e.stopPropagation();
										navigator.clipboard.writeText(filename);
										toast.success(`Copied landmark path: ${filename}`);
									};

									return (
										<Card
											key={i}
											className="p-4 bg-secondary/15 border-border/40 flex flex-col justify-between gap-3 group hover:bg-secondary/25 transition-all duration-300 hover:border-border"
										>
											<div className="space-y-2">
												<div className="flex items-start justify-between gap-2">
													<Badge
														variant="outline"
														className={`${meta.color} text-[8px] font-extrabold uppercase px-2 py-0.5 rounded`}
													>
														{meta.label}
													</Badge>
													<button
														onClick={handleCopyLandmark}
														className="h-6 w-6 rounded hover:bg-secondary text-zinc-400 flex items-center justify-center shrink-0 border border-transparent hover:border-border/30 transition-all opacity-0 group-hover:opacity-100 cursor-pointer"
														title="Copy path"
													>
														<Copy className="h-3 w-3" />
													</button>
												</div>

												<div className="flex items-center gap-2">
													<Icon className="h-4 w-4 text-primary shrink-0" />
													<span
														className="font-mono text-xs font-bold text-zinc-100 truncate group-hover:text-primary transition-colors select-all"
														title={filename}
													>
														{filename.split("/").pop()?.split("\\").pop() ||
															filename}
													</span>
												</div>

												<p className="text-[10px] text-muted-foreground leading-normal leading-snug">
													{meta.desc}
												</p>
											</div>

											<div className="text-[9px] font-mono text-muted-foreground/60 border-t border-border/10 pt-2 flex items-center justify-between shrink-0">
												<span className="truncate max-w-[80%]" title={filename}>
													{filename}
												</span>
												<ArrowRight className="h-3 w-3 shrink-0 opacity-0 group-hover:opacity-100 group-hover:translate-x-0.5 transition-all text-primary" />
											</div>
										</Card>
									);
								})}
							</div>
						)}
					</div>

					{/* Categories card */}
					<Card className="p-4 bg-secondary/20 border-border/50 space-y-4">
						<h3 className="font-bold text-xs uppercase tracking-wider text-zinc-200 flex items-center gap-2 border-b border-border/20 pb-2.5">
							<Database className="h-4.5 w-4.5 text-primary shrink-0" />
							<span>Category File Breakdown</span>
						</h3>
						{parsed.categories.length === 0 ? (
							<p className="text-xs text-muted-foreground italic">
								No category breakdown details detected.
							</p>
						) : (
							<div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
								{parsed.categories.map((cat, i) => (
									<div
										key={i}
										className="flex items-center justify-between p-2.5 bg-zinc-950/40 rounded-lg border border-border/20 hover:bg-secondary/10 transition-colors"
									>
										<span className="text-xs font-semibold text-zinc-300 truncate max-w-[80%]">
											{cat.label}
										</span>
										<Badge
											variant="outline"
											className="text-xs font-bold font-mono text-primary bg-primary/5 px-2 py-0.5 border border-primary/20 rounded-lg"
										>
											{cat.count}
										</Badge>
									</div>
								))}
							</div>
						)}
					</Card>
				</div>
			)}
		</div>
	);
}
