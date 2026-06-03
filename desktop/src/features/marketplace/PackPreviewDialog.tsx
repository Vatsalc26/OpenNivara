import {
	AlertCircle,
	AlertTriangle,
	Download,
	ShieldAlert,
	ShieldCheck,
	Sparkles,
	X,
} from "lucide-react";
import type { PackPreview } from "@/api/marketplaceClient";

interface PackPreviewDialogProps {
	preview: PackPreview | null;
	isOpen: boolean;
	onClose: () => void;
	onInstall: () => void;
	isInstalling?: boolean;
}

export function PackPreviewDialog({
	preview,
	isOpen,
	onClose,
	onInstall,
	isInstalling = false,
}: PackPreviewDialogProps) {
	if (!isOpen || !preview) return null;

	const manifest = preview.manifest;
	const safety = preview.safety_summary;

	const riskColor =
		safety.risk_level === "low"
			? "text-emerald-400 bg-emerald-500/10 border-emerald-500/25"
			: safety.risk_level === "medium"
				? "text-amber-400 bg-amber-500/10 border-amber-500/25"
				: "text-rose-400 bg-rose-500/10 border-rose-500/25";

	return (
		<div className="fixed inset-0 z-50 flex items-center justify-center p-4">
			{/* Dark overlay backdrop */}
			<div
				className="absolute inset-0 bg-background/80 backdrop-blur-md transition-opacity duration-300"
				onClick={onClose}
			/>

			{/* Dialog body */}
			<div className="relative bg-card border border-border w-full max-w-xl rounded-2xl overflow-hidden shadow-2xl flex flex-col max-h-[90vh] transition-all duration-300 select-none scale-100">
				{/* Header */}
				<div className="p-5 border-b border-border bg-card flex items-center justify-between shrink-0">
					<div className="flex items-center gap-2.5">
						<div className="h-8 w-8 rounded-lg bg-primary/10 border border-primary/20 flex items-center justify-center">
							<Sparkles className="h-4.5 w-4.5 text-primary animate-pulse" />
						</div>
						<div>
							<h2 className="font-extrabold text-sm text-foreground uppercase tracking-wide">
								Pack Safety Inspection
							</h2>
							<p className="text-[10px] text-muted-foreground/80 font-bold uppercase tracking-wider">
								Review permissions & content additions
							</p>
						</div>
					</div>
					<button
						onClick={onClose}
						className="h-8 w-8 rounded-lg bg-secondary hover:bg-secondary/80 text-muted-foreground hover:text-foreground flex items-center justify-center transition-colors cursor-pointer"
					>
						<X className="h-4 w-4" />
					</button>
				</div>

				{/* Content Body */}
				<div className="flex-1 overflow-y-auto p-6 space-y-5">
					{/* Metadata Card */}
					<div className="bg-secondary/20 border border-border/50 p-4 rounded-xl space-y-2">
						<div className="flex justify-between items-center">
							<span className="text-[10px] bg-primary/10 text-primary font-extrabold border border-primary/20 px-2 py-0.5 rounded uppercase">
								{manifest.category}
							</span>
							<span
								className={`text-[10px] font-extrabold border px-2 py-0.5 rounded uppercase ${riskColor}`}
							>
								Risk: {safety.risk_level}
							</span>
						</div>
						<h3 className="font-extrabold text-base text-foreground">
							{manifest.name}
						</h3>
						<p className="text-xs text-muted-foreground leading-relaxed font-semibold">
							{manifest.description}
						</p>
						<div className="text-[10px] text-muted-foreground/80 font-bold tracking-wider uppercase pt-1">
							Author: {manifest.author} | Version: {manifest.version}
						</div>
					</div>

					{/* Additions count summary */}
					<div className="space-y-2.5">
						<h4 className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
							Resource Additions Summary
						</h4>
						<div className="grid grid-cols-2 gap-2">
							{[
								{
									label: "Preferences Trigger Lists",
									count: preview.additions.preferences_count,
								},
								{
									label: "Goal Context Entries",
									count: preview.additions.contexts_count,
								},
								{
									label: "Response Style Guidelines",
									count: preview.additions.style_presets_count,
								},
								{
									label: "Visual Color Themes",
									count: preview.additions.themes_count,
								},
								{
									label: "Command Input Snippets",
									count: preview.additions.command_snippets_count,
								},
								{
									label: "Workspace Landmark Rules",
									count: preview.additions.workspace_rules_count,
								},
							].map((item, idx) => (
								<div
									key={idx}
									className="bg-card border border-border p-2.5 rounded-xl flex items-center justify-between"
								>
									<span className="text-[11px] font-semibold text-muted-foreground">
										{item.label}
									</span>
									<span className="text-xs font-extrabold text-primary bg-primary/10 px-2 py-0.5 rounded">
										+{item.count}
									</span>
								</div>
							))}
						</div>
					</div>

					{/* Safety & Compliance Checklist */}
					<div className="space-y-2.5">
						<h4 className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
							Security Compliance Checks
						</h4>
						<div className="space-y-2">
							<div className="flex items-center justify-between p-3 rounded-xl bg-card border border-border">
								<div className="flex items-center gap-2">
									{safety.contains_executable_code ? (
										<ShieldAlert className="h-4.5 w-4.5 text-rose-400 shrink-0" />
									) : (
										<ShieldCheck className="h-4.5 w-4.5 text-emerald-400 shrink-0" />
									)}
									<span className="text-xs font-semibold text-foreground">
										Contains Native Binary/Code
									</span>
								</div>
								<span
									className={`text-[10px] font-extrabold px-2 py-0.5 rounded border uppercase ${
										safety.contains_executable_code
											? "bg-rose-500/10 text-rose-400 border-rose-500/20"
											: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20"
									}`}
								>
									{safety.contains_executable_code
										? "Violates Safety"
										: "Complies"}
								</span>
							</div>

							<div className="flex items-center justify-between p-3 rounded-xl bg-card border border-border">
								<div className="flex items-center gap-2">
									{safety.modifies_tool_permissions ? (
										<ShieldAlert className="h-4.5 w-4.5 text-rose-400 shrink-0" />
									) : (
										<ShieldCheck className="h-4.5 w-4.5 text-emerald-400 shrink-0" />
									)}
									<span className="text-xs font-semibold text-foreground">
										Modifies Tool Security Policies
									</span>
								</div>
								<span
									className={`text-[10px] font-extrabold px-2 py-0.5 rounded border uppercase ${
										safety.modifies_tool_permissions
											? "bg-rose-500/10 text-rose-400 border-rose-500/20"
											: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20"
									}`}
								>
									{safety.modifies_tool_permissions
										? "Violates Safety"
										: "Complies"}
								</span>
							</div>

							<div className="flex items-center justify-between p-3 rounded-xl bg-card border border-border">
								<div className="flex items-center gap-2">
									<ShieldCheck className="h-4.5 w-4.5 text-emerald-400 shrink-0" />
									<span className="text-xs font-semibold text-foreground">
										Requires Network Access
									</span>
								</div>
								<span
									className={`text-[10px] font-extrabold px-2 py-0.5 rounded border uppercase bg-secondary text-muted-foreground border-border`}
								>
									{safety.requires_network ? "Yes" : "No"}
								</span>
							</div>
						</div>
					</div>

					{/* Warnings list if any */}
					{(preview.warnings.length > 0 || preview.errors.length > 0) && (
						<div className="p-3.5 bg-rose-500/5 border border-rose-500/15 rounded-xl space-y-1.5">
							<div className="flex items-center gap-1.5 text-rose-400 font-extrabold text-[10px] uppercase">
								<AlertTriangle className="h-4 w-4 text-rose-400 shrink-0" />
								<span>Diagnostics & Warnings</span>
							</div>
							<ul className="text-[10px] text-muted-foreground space-y-1 leading-relaxed pl-4 list-disc font-medium">
								{preview.errors.map((err, idx) => (
									<li key={`err-${idx}`} className="text-rose-400 font-bold">
										[BLOCKING] {err}
									</li>
								))}
								{preview.warnings.map((warn, idx) => (
									<li key={`warn-${idx}`}>{warn}</li>
								))}
							</ul>
						</div>
					)}
				</div>

				{/* Footer Buttons */}
				<div className="p-5 border-t border-border bg-card shrink-0 flex gap-3">
					<button
						onClick={onClose}
						className="flex-1 py-2.5 rounded-xl bg-secondary hover:bg-secondary/80 text-foreground font-bold text-xs transition-colors cursor-pointer text-center"
					>
						Cancel
					</button>

					{safety.allowed_to_install ? (
						<button
							onClick={onInstall}
							disabled={isInstalling}
							className="flex-1 py-2.5 rounded-xl bg-primary hover:bg-primary/90 text-primary-foreground font-extrabold text-xs tracking-wider uppercase transition-colors cursor-pointer text-center flex items-center justify-center gap-1.5 disabled:opacity-50"
						>
							<Download className="h-4 w-4" />
							<span>{isInstalling ? "Installing..." : "Install Pack"}</span>
						</button>
					) : (
						<div className="flex-1 py-2.5 rounded-xl bg-secondary text-rose-400 border border-rose-500/20 font-bold text-xs uppercase tracking-wide flex items-center justify-center gap-1.5 cursor-not-allowed">
							<AlertCircle className="h-4 w-4 text-rose-400 shrink-0" />
							<span>Blocked by Policy</span>
						</div>
					)}
				</div>
			</div>
		</div>
	);
}
