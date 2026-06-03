import { Palette } from "lucide-react";
import type { OpenNivaraTheme } from "@/api/marketplaceClient";

interface ThemePreviewProps {
	theme: OpenNivaraTheme;
	isActive?: boolean;
	onApply?: () => void;
}

export function ThemePreview({
	theme,
	isActive = false,
	onApply,
}: ThemePreviewProps) {
	const colors = theme.colors;

	return (
		<div className="bg-card border border-border rounded-2xl p-4.5 flex flex-col justify-between gap-4 select-none relative hover:border-primary/30 transition-colors">
			<div className="space-y-3">
				{/* Title & Badge */}
				<div className="flex items-center justify-between gap-2">
					<h4 className="font-extrabold text-xs text-foreground flex items-center gap-1.5 leading-none">
						<Palette className="h-3.5 w-3.5 text-primary" />
						<span>{theme.name}</span>
					</h4>
					{isActive && (
						<span className="text-[8px] bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 font-extrabold px-1.5 py-0.5 rounded uppercase leading-none">
							Applied
						</span>
					)}
				</div>
				<p className="text-[10px] text-muted-foreground leading-normal line-clamp-2">
					{theme.description}
				</p>
			</div>

			{/* Colors Grid Preview */}
			<div className="space-y-1.5">
				<div className="text-[8px] font-extrabold text-muted-foreground/80 uppercase tracking-widest leading-none">
					Palette Swatch
				</div>
				<div className="flex items-center gap-1 rounded-lg bg-secondary p-1 border border-border/50">
					{[
						{ color: colors.background, title: "Background" },
						{ color: colors.panel, title: "Panel" },
						{ color: colors.card, title: "Card" },
						{ color: colors.primary, title: "Primary" },
						{ color: colors.accent, title: "Accent" },
						{ color: colors.foreground, title: "Foreground" },
						{ color: colors.success, title: "Success" },
						{ color: colors.warning, title: "Warning" },
						{ color: colors.danger, title: "Danger" },
					].map((item, idx) => (
						<div
							key={idx}
							className="h-5 flex-1 rounded-md border border-background shadow-inner"
							style={{ backgroundColor: item.color }}
							title={`${item.title}: ${item.color}`}
						/>
					))}
				</div>
			</div>

			{/* Apply Button */}
			{onApply && !isActive && (
				<button
					onClick={onApply}
					className="w-full flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-xl bg-secondary hover:bg-secondary/80 text-foreground font-bold text-xs tracking-wider uppercase transition-colors cursor-pointer"
				>
					<span>Apply Theme</span>
				</button>
			)}
		</div>
	);
}
