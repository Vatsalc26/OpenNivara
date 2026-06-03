import {
	AlertTriangle,
	CheckCircle2,
	FileCode,
	FolderOpen,
	Info,
	Lock,
	Shield,
	ShieldAlert,
	Unlock,
	Wrench,
	XCircle,
} from "lucide-react";
import { toast } from "sonner";
import type { ToolsConfig } from "@/api/tauriClient";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";

interface ToolStatusListProps {
	config: ToolsConfig | null;
	configPath: string | null;
}

export function ToolStatusList({ config, configPath }: ToolStatusListProps) {
	if (!config) {
		return (
			<div className="flex flex-col items-center justify-center h-full text-center space-y-4">
				<Wrench className="h-8 w-8 text-primary animate-spin" />
				<p className="text-sm text-muted-foreground font-semibold">
					Reading tools configurations...
				</p>
			</div>
		);
	}

	// Define metadata helper for risk levels of tools
	const getToolRisk = (name: string): "low" | "medium" | "high" => {
		switch (name) {
			case "get_current_dir":
			case "file_exists":
				return "low";
			case "list_dir":
			case "read_file":
			case "map_summary":
			case "map_tree":
			case "map_search":
			case "map_get_node":
				return "medium";
			default:
				return "high";
		}
	};

	const riskBadges = {
		low: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
		medium: "bg-amber-500/10 text-amber-400 border-amber-500/20",
		high: "bg-rose-500/10 text-rose-400 border-rose-500/20",
	};

	const riskCardBorders = {
		low: "hover:border-emerald-500/30",
		medium: "hover:border-amber-500/30",
		high: "hover:border-rose-500/30",
	};

	// Group tools by risk level
	const toolsList = Object.entries(config.tools);
	const lowRisk = toolsList.filter(([name]) => getToolRisk(name) === "low");
	const mediumRisk = toolsList.filter(
		([name]) => getToolRisk(name) === "medium",
	);
	const highRisk = toolsList.filter(([name]) => getToolRisk(name) === "high");

	// Calculate statistics
	const totalToolsCount = toolsList.length;
	const enabledToolsCount = toolsList.filter(([_, t]) => t.enabled).length;
	const disabledToolsCount = totalToolsCount - enabledToolsCount;
	const autoExecuteCount = toolsList.filter(
		([_, t]) => t.enabled && !t.requires_confirmation,
	).length;
	const approvalRequiredCount = toolsList.filter(
		([_, t]) => t.enabled && t.requires_confirmation,
	).length;

	const handleCopyPath = () => {
		if (configPath) {
			navigator.clipboard.writeText(configPath);
			toast.success("Tools config path copied to clipboard!");
		}
	};

	const renderToolCard = (
		name: string,
		settings: any,
		risk: "low" | "medium" | "high",
	) => {
		return (
			<Card
				key={name}
				className={`p-4 bg-secondary/15 border-border/40 flex flex-col justify-between space-y-3 group hover:bg-secondary/25 transition-all duration-300 ${riskCardBorders[risk]}`}
			>
				<div className="space-y-2">
					<div className="flex items-start justify-between gap-3">
						<span className="font-bold text-xs tracking-wide font-mono text-zinc-100 truncate select-all group-hover:text-primary transition-colors">
							{name}
						</span>
						<Badge
							variant="outline"
							className={`${riskBadges[risk]} capitalize text-[9px] px-2 py-0.5 rounded-full font-bold`}
						>
							{risk} Risk
						</Badge>
					</div>

					<div className="flex items-center gap-4 text-[10px] font-semibold text-zinc-400">
						<span className="flex items-center gap-1">
							{settings.enabled ? (
								<CheckCircle2 className="h-3.5 w-3.5 text-emerald-400" />
							) : (
								<XCircle className="h-3.5 w-3.5 text-rose-400" />
							)}
							{settings.enabled ? "Enabled" : "Disabled"}
						</span>

						<span className="flex items-center gap-1">
							{settings.requires_confirmation ? (
								<Lock className="h-3.5 w-3.5 text-amber-400" />
							) : (
								<Unlock className="h-3.5 w-3.5 text-emerald-400" />
							)}
							{settings.requires_confirmation
								? "Requires Approval"
								: "Auto-Execute"}
						</span>
					</div>
				</div>

				{settings.max_bytes && (
					<div className="text-[10px] font-mono text-muted-foreground/60 border-t border-border/10 pt-2 flex items-center gap-1 select-none">
						<FileCode className="h-3.5 w-3.5 text-zinc-500" />
						Max Preview: {settings.max_bytes.toLocaleString()} bytes
					</div>
				)}
			</Card>
		);
	};

	return (
		<div className="flex flex-col h-full bg-background/20 p-6 space-y-6 overflow-hidden">
			{/* Header section */}
			<div className="space-y-1 border-b border-border/20 pb-4 shrink-0 flex flex-col md:flex-row md:items-center md:justify-between gap-4">
				<div>
					<h2 className="text-xl font-bold tracking-wide font-heading flex items-center gap-2">
						<Shield className="h-5 w-5 text-primary" />
						<span>Tool Security Management</span>
					</h2>
					<p className="text-xs text-muted-foreground leading-relaxed">
						Overview of local command tools permissions and safety filters
						defined inside your global `tools.toml`.
					</p>
				</div>
				{configPath && (
					<button
						onClick={handleCopyPath}
						className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-border/40 bg-secondary/10 hover:bg-secondary/20 transition-all font-mono text-[10px] text-zinc-400 cursor-pointer max-w-sm truncate self-start md:self-auto select-none"
						title="Click to copy full path"
					>
						<FolderOpen className="h-3.5 w-3.5 text-primary shrink-0" />
						<span className="truncate">tools.toml</span>
					</button>
				)}
			</div>

			<ScrollArea className="flex-1 -mx-2 px-2">
				<div className="space-y-6 pb-6">
					{/* Top Row Statistics Cards */}
					<div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
						{[
							{
								label: "Total Registered",
								value: totalToolsCount,
								sub: `${enabledToolsCount} active, ${disabledToolsCount} inactive`,
								icon: Wrench,
								iconColor: "text-zinc-400",
							},
							{
								label: "Safety Strictness",
								value: config.general.enabled ? "ACTIVE" : "DISABLED",
								sub: `Tool support is ${config.general.enabled ? "on" : "off"}`,
								icon: Shield,
								iconColor: config.general.enabled
									? "text-emerald-400"
									: "text-rose-400",
							},
							{
								label: "Auto-Execute Tools",
								value: autoExecuteCount,
								sub: "Zero latency prompts",
								icon: Unlock,
								iconColor: "text-emerald-400",
							},
							{
								label: "Requires Approval",
								value: approvalRequiredCount,
								sub: "User confirmation required",
								icon: Lock,
								iconColor: "text-amber-400",
							},
						].map((stat, idx) => {
							const Icon = stat.icon;
							return (
								<Card
									key={idx}
									className="p-4 bg-secondary/25 border-border/50 flex flex-col justify-between min-h-[90px] glow-effect"
								>
									<div className="flex items-center justify-between gap-2">
										<span className="text-[10px] text-muted-foreground font-bold uppercase tracking-wider">
											{stat.label}
										</span>
										<Icon
											className={`h-4.5 w-4.5 ${stat.iconColor} shrink-0`}
										/>
									</div>
									<div className="space-y-0.5 mt-2">
										<p className="text-xl font-extrabold font-heading text-primary leading-none">
											{stat.value}
										</p>
										<span className="text-[9px] text-muted-foreground/80 font-medium tracking-wide block">
											{stat.sub}
										</span>
									</div>
								</Card>
							);
						})}
					</div>

					{/* Paths restrictions cards */}
					<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
						<Card className="p-4 bg-secondary/20 border-border/50 space-y-3.5">
							<div className="flex items-center gap-2 border-b border-border/20 pb-2">
								<FolderOpen className="h-4.5 w-4.5 text-primary shrink-0" />
								<h3 className="font-bold text-xs uppercase tracking-wider text-zinc-200">
									Allowed Workspace Roots
								</h3>
							</div>
							<ul className="space-y-2">
								{config.paths.allowed_roots.map((root, i) => (
									<li
										key={i}
										className="text-xs font-mono bg-zinc-950/40 p-2 rounded-lg border border-border/20 flex items-center gap-2 overflow-hidden text-ellipsis whitespace-nowrap select-all hover:bg-zinc-900/35 transition-colors"
									>
										<CheckCircle2 className="h-3.5 w-3.5 text-emerald-400 shrink-0" />
										<span className="truncate text-zinc-300 font-medium">
											{root}
										</span>
									</li>
								))}
							</ul>
						</Card>

						<Card className="p-4 bg-secondary/20 border-border/50 space-y-3.5">
							<div className="flex items-center gap-2 border-b border-border/20 pb-2">
								<ShieldAlert className="h-4.5 w-4.5 text-rose-400 shrink-0" />
								<h3 className="font-bold text-xs uppercase tracking-wider text-zinc-200">
									Blocked Safety Keywords
								</h3>
							</div>
							<p className="text-[10px] text-muted-foreground leading-normal">
								Files matching these keywords or absolute paths will be rejected
								immediately before accessing disk tools.
							</p>
							<div className="flex flex-wrap gap-2">
								{config.paths.blocked_patterns.map((pat, i) => (
									<Badge
										key={i}
										variant="outline"
										className="text-[10px] font-mono font-bold bg-rose-500/5 text-rose-400 border-rose-500/25 px-2.5 py-1 rounded-full select-all"
									>
										{pat}
									</Badge>
								))}
							</div>
						</Card>
					</div>

					{/* Risk-grouped Assistive Tools */}
					<div className="space-y-6">
						{/* High risk tools section */}
						{highRisk.length > 0 && (
							<div className="space-y-3">
								<div className="flex items-center justify-between border-b border-border/20 pb-2">
									<h3 className="font-bold text-xs uppercase tracking-wider text-rose-400 flex items-center gap-2">
										<AlertTriangle className="h-4 w-4 shrink-0" />
										<span>High Risk Action Tools</span>
									</h3>
									<Badge
										variant="outline"
										className="text-[9px] font-bold border-rose-500/30 text-rose-400 bg-rose-500/5 px-2 py-0.5 rounded-full select-none"
									>
										Requires Extreme Care
									</Badge>
								</div>
								<p className="text-[10px] text-muted-foreground leading-relaxed bg-rose-500/5 border border-rose-500/10 p-3 rounded-lg">
									<strong className="text-rose-400">Security Warning:</strong>{" "}
									These tools run shell processes, execute build commands, or
									edit system configurations. Ensure that "Requires Approval" is
									turned on for all automated execution agents.
								</p>
								<div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
									{highRisk.map(([name, settings]) =>
										renderToolCard(name, settings, "high"),
									)}
								</div>
							</div>
						)}

						{/* Medium risk tools section */}
						{mediumRisk.length > 0 && (
							<div className="space-y-3">
								<div className="flex items-center justify-between border-b border-border/20 pb-2">
									<h3 className="font-bold text-xs uppercase tracking-wider text-amber-400 flex items-center gap-2">
										<Info className="h-4 w-4 shrink-0 text-amber-400" />
										<span>Medium Risk Reading Tools</span>
									</h3>
									<Badge
										variant="outline"
										className="text-[9px] font-bold border-amber-500/30 text-amber-400 bg-amber-500/5 px-2 py-0.5 rounded-full select-none"
									>
										File Inspections
									</Badge>
								</div>
								<div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
									{mediumRisk.map(([name, settings]) =>
										renderToolCard(name, settings, "medium"),
									)}
								</div>
							</div>
						)}

						{/* Low risk tools section */}
						{lowRisk.length > 0 && (
							<div className="space-y-3">
								<div className="flex items-center justify-between border-b border-border/20 pb-2">
									<h3 className="font-bold text-xs uppercase tracking-wider text-emerald-400 flex items-center gap-2">
										<CheckCircle2 className="h-4 w-4 shrink-0 text-emerald-400" />
										<span>Low Risk Discovery Tools</span>
									</h3>
									<Badge
										variant="outline"
										className="text-[9px] font-bold border-emerald-500/30 text-emerald-400 bg-emerald-500/5 px-2 py-0.5 rounded-full select-none"
									>
										Basic Assertions
									</Badge>
								</div>
								<div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
									{lowRisk.map(([name, settings]) =>
										renderToolCard(name, settings, "low"),
									)}
								</div>
							</div>
						)}
					</div>
				</div>
			</ScrollArea>
		</div>
	);
}
