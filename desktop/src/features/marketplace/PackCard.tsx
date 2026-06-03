import { FolderSync, Info, Sparkles, Trash2, User } from "lucide-react";
import type { InstalledPack, PackPreview } from "@/api/marketplaceClient";
import { Card } from "@/components/ui/card";

interface PackCardProps {
	pack: InstalledPack | PackPreview["manifest"];
	isInstalled?: boolean;
	onPreview?: () => void;
	onUninstall?: () => void;
	onAddToMode?: () => void;
	onToggleEnable?: (enabled: boolean) => void;
	activeModeIsDefault?: boolean;
}

export function PackCard({
	pack,
	isInstalled = false,
	onPreview,
	onUninstall,
	onAddToMode,
	onToggleEnable,
	activeModeIsDefault = false,
}: PackCardProps) {
	// Extract info regardless of whether it's InstalledPack or PackManifest
	const _id = pack.id;
	const name = pack.name;
	const version = pack.version;

	// Manifest fields exist on PackManifest, or default otherwise
	const author = "author" in pack ? pack.author : "OpenNivara";
	const category = "category" in pack ? pack.category : "Custom";
	const description =
		"description" in pack ? pack.description : "Custom installed pack.";
	const safety = "safety" in pack ? pack.safety : { risk_level: "low" };

	const riskColor =
		safety.risk_level === "low"
			? "bg-emerald-500/10 text-emerald-400 border-emerald-500/20"
			: safety.risk_level === "medium"
				? "bg-amber-500/10 text-amber-400 border-amber-500/20"
				: "bg-rose-500/10 text-rose-400 border-rose-500/20";

	const isDisabled = isInstalled && "enabled" in pack && !pack.enabled;

	return (
		<Card
			className={`group relative overflow-hidden bg-card hover:bg-card/85 border border-border hover:border-primary/30 p-5 rounded-2xl transition-all duration-300 flex flex-col justify-between shadow hover:shadow-primary/10 select-none ${isDisabled ? "opacity-65 saturate-50 hover:opacity-100 hover:saturate-100" : ""}`}
		>
			{/* Visual background accents */}
			<div className="absolute -top-12 -right-12 h-24 w-24 rounded-full bg-primary/5 blur-xl group-hover:bg-primary/10 transition-colors duration-300" />

			<div className="space-y-3 relative">
				{/* Category & Safety Badge */}
				<div className="flex items-center justify-between gap-2">
					<div className="flex items-center gap-1.5">
						<span className="text-[9px] font-extrabold uppercase tracking-widest text-primary bg-primary/10 px-2 py-0.5 rounded border border-primary/20">
							{category}
						</span>
						{isInstalled && "enabled" in pack && !pack.enabled && (
							<span className="text-[9px] font-extrabold uppercase tracking-wide px-1.5 py-0.5 rounded border bg-rose-500/10 text-rose-400 border-rose-500/20">
								Disabled
							</span>
						)}
					</div>
					<span
						className={`text-[9px] font-extrabold uppercase tracking-wide px-1.5 py-0.5 rounded border ${riskColor}`}
					>
						Risk: {safety.risk_level}
					</span>
				</div>

				{/* Title */}
				<div className="space-y-1">
					<h3 className="font-extrabold text-sm text-foreground group-hover:text-primary transition-colors leading-tight flex items-center gap-1.5">
						<Sparkles className="h-3.5 w-3.5 text-primary shrink-0" />
						<span>{name}</span>
					</h3>
					<p className="text-[10px] text-muted-foreground font-semibold flex items-center gap-1">
						<User className="h-3 w-3" />
						<span>
							by {author} | v{version}
						</span>
					</p>
				</div>

				{/* Description */}
				<p className="text-muted-foreground text-xs font-medium leading-relaxed line-clamp-3 pt-1">
					{description}
				</p>
			</div>

			{/* Buttons panel */}
			<div className="flex items-center gap-2 pt-4 border-t border-border/40 mt-4 relative">
				{onPreview && (
					<button
						onClick={onPreview}
						className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl bg-secondary hover:bg-secondary/80 text-foreground hover:text-foreground font-bold text-xs transition-colors cursor-pointer"
					>
						<Info className="h-3.5 w-3.5" />
						<span>Inspect</span>
					</button>
				)}

				{isInstalled ? (
					<>
						{onToggleEnable && "enabled" in pack && (
							<button
								onClick={() => onToggleEnable(!pack.enabled)}
								className={`flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl border font-bold text-xs transition-all cursor-pointer ${
									!pack.enabled
										? "bg-amber-500/5 hover:bg-amber-500/10 border-amber-500/20 text-amber-500 hover:text-amber-400"
										: "bg-emerald-500/5 hover:bg-emerald-500/10 border-emerald-500/20 text-emerald-400 hover:text-emerald-300"
								}`}
								title={pack.enabled ? "Disable Pack" : "Enable Pack"}
							>
								<span>{pack.enabled ? "Enabled" : "Disabled"}</span>
							</button>
						)}
						{onAddToMode && (
							<button
								onClick={onAddToMode}
								className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl bg-primary/10 hover:bg-primary/20 border border-primary/20 hover:border-primary/40 text-primary font-bold text-xs transition-all cursor-pointer"
							>
								<FolderSync className="h-3.5 w-3.5" />
								<span>
									{activeModeIsDefault ? "Choose Mode" : "Add to Active Mode"}
								</span>
							</button>
						)}
						{onUninstall && (
							<button
								onClick={onUninstall}
								className="h-8.5 w-8.5 flex items-center justify-center rounded-xl bg-rose-500/5 hover:bg-rose-500/10 border border-rose-500/10 hover:border-rose-500/30 text-rose-400 hover:text-rose-300 transition-colors cursor-pointer shrink-0"
								title="Uninstall Pack"
							>
								<Trash2 className="h-4 w-4" />
							</button>
						)}
					</>
				) : (
					<div className="text-[10px] text-muted-foreground font-bold italic w-full text-center py-1">
						Built-in sample pack
					</div>
				)}
			</div>
		</Card>
	);
}
