import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
	Brain,
	Calendar,
	Database,
	MessageSquare,
	Plus,
	Shield,
	ShoppingBag,
	Sliders,
	Sparkles,
	Terminal,
	Wrench,
} from "lucide-react";
import { toast } from "sonner";
import { getModes, setActiveMode } from "@/api/marketplaceClient";
import { useOpenNivaraTheme } from "@/theme/ThemeProvider";

interface SidebarProps {
	activeView: string;
	onNavigate: (view: string) => void;
	onNewChat: () => void;
	apiKeyReady: boolean;
	toolsEnabled: boolean;
}

export function Sidebar({
	activeView,
	onNavigate,
	onNewChat,
	apiKeyReady,
	toolsEnabled: _toolsEnabled,
}: SidebarProps) {
	const queryClient = useQueryClient();
	const { refreshTheme } = useOpenNivaraTheme();

	const { data: modesFile = null } = useQuery({
		queryKey: ["modes"],
		queryFn: getModes,
	});

	const _activeMode = modesFile?.modes.find(
		(m) => m.id === modesFile?.active_mode,
	);

	const _handleSetActiveMode = async (modeId: string) => {
		try {
			await setActiveMode(modeId);
			queryClient.invalidateQueries({ queryKey: ["modes"] });
			// Invalidate preview_context queries to update Inspector
			queryClient.invalidateQueries({ queryKey: ["preview_context"] });

			// Update theme variables instantly using centralized theme manager
			await refreshTheme();

			// Find mode name for toast
			const targetMode = modesFile?.modes.find((m) => m.id === modeId);
			toast.success(`${targetMode ? targetMode.name : "Mode"} activated!`);
		} catch (err: any) {
			toast.error(`Failed to switch active mode: ${err?.message || err}`);
		}
	};

	const items = [
		{ id: "chat", label: "Chat", icon: MessageSquare },
		{ id: "sessions", label: "Sessions", icon: Calendar },
		{ id: "tools", label: "Tools", icon: Wrench },
		{ id: "workspace", label: "Workspace", icon: Database },
		{ id: "memory", label: "Memory", icon: Brain },
		{ id: "settings", label: "Settings", icon: Sliders },
		{ id: "marketplace", label: "Store", icon: ShoppingBag },
	];

	return (
		<aside className="w-56 bg-sidebar border-r border-sidebar-border flex flex-col shrink-0 text-muted-foreground font-sans select-none animate-none">
			{/* Brand logo header */}
			<div className="p-5 border-b border-sidebar-border flex items-center gap-3">
				<div className="h-8 w-8 rounded-xl bg-primary/10 border border-primary/20 flex items-center justify-center shrink-0">
					<Sparkles className="h-4 w-4 text-primary animate-pulse" />
				</div>
				<div className="flex flex-col">
					<span className="font-extrabold text-[11px] tracking-widest text-foreground uppercase font-heading leading-tight">
						OPENNIVARA
					</span>
					<span className="text-[9px] text-muted-foreground font-bold uppercase tracking-wider">
						Command Center
					</span>
				</div>
			</div>

			{/* New chat action button */}
			<div className="px-4 py-4 border-b border-sidebar-border/50">
				<button
					onClick={onNewChat}
					className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-xl border border-primary/20 bg-primary/5 hover:bg-primary/10 text-primary font-semibold text-xs tracking-wider uppercase transition-all duration-300 shadow hover:shadow-lg focus:outline-none cursor-pointer"
				>
					<Plus className="h-3.5 w-3.5" />
					<span>New Chat</span>
				</button>
			</div>

			{/* Navigation list */}
			<nav className="flex-1 px-2.5 py-3 space-y-1 overflow-y-auto">
				{items.map((item) => {
					const Icon = item.icon;
					const isActive = activeView === item.id;
					return (
						<button
							key={item.id}
							onClick={() => onNavigate(item.id)}
							className={`w-full flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-semibold tracking-wide transition-all duration-200 cursor-pointer ${
								isActive
									? "bg-sidebar-accent text-sidebar-primary border-l-2 border-sidebar-primary animate-none"
									: "text-muted-foreground hover:text-foreground hover:bg-sidebar-accent/40"
							}`}
						>
							<Icon
								className={`h-4.5 w-4.5 shrink-0 ${isActive ? "text-sidebar-primary" : "text-muted-foreground/80"}`}
							/>
							<span>{item.label}</span>
						</button>
					);
				})}
			</nav>

			{/* Sidebar Footer badges */}
			<div className="p-3 border-t border-sidebar-border bg-sidebar/45 space-y-2.5">
				<div className="flex flex-col gap-1.5 text-[9px] font-bold text-muted-foreground/80 tracking-wider uppercase">
					<div className="flex items-center justify-between px-1.5 py-0.5 rounded hover:bg-sidebar-accent/30 transition-colors">
						<span className="flex items-center gap-1.5">
							<Shield className="h-3 w-3 text-emerald-400 shrink-0" />
							<span>Safe Shell</span>
						</span>
						<span className="text-[8px] bg-emerald-500/10 text-emerald-400 font-extrabold px-1.5 py-0.5 rounded border border-emerald-500/20 select-none">
							Active
						</span>
					</div>

					<div className="flex items-center justify-between px-1.5 py-0.5 rounded hover:bg-sidebar-accent/30 transition-colors">
						<span className="flex items-center gap-1.5">
							<Terminal className="h-3 w-3 text-muted-foreground/80 shrink-0" />
							<span>API Status</span>
						</span>
						<span
							className={`text-[8px] font-extrabold px-1.5 py-0.5 rounded border select-none ${apiKeyReady ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/20" : "bg-red-500/10 text-red-400 border-red-500/20"}`}
						>
							{apiKeyReady ? "Ready" : "Missing"}
						</span>
					</div>
				</div>

				{/* Command palette keyboard shortcut helper */}
				<div className="flex items-center justify-center gap-1 text-[9px] text-muted-foreground/60 font-semibold border border-sidebar-border/50 bg-sidebar p-1.5 rounded-lg select-none">
					<span>Press</span>
					<kbd className="px-1.5 py-0.5 bg-sidebar-accent text-muted-foreground/80 rounded border border-sidebar-border font-mono text-[8px] font-extrabold select-none">
						Ctrl + K
					</kbd>
					<span>for Command Palette</span>
				</div>
			</div>
		</aside>
	);
}
