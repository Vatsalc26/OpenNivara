import { Command } from "cmdk";
import {
	BookOpen,
	Calendar,
	Database,
	Info,
	MessageSquare,
	Palette,
	Plus,
	Search,
	Settings,
	Sliders,
	Sparkles,
	User,
	Wrench,
} from "lucide-react";
import type React from "react";
import { useEffect } from "react";

interface CommandPaletteProps {
	open: boolean;
	setOpen: React.Dispatch<React.SetStateAction<boolean>>;
	onNavigate: (view: string, tab?: any) => void;
	onNewChat: () => void;
	showInspector?: boolean;
	onToggleInspector?: () => void;
}

export function CommandPalette({
	open,
	setOpen,
	onNavigate,
	onNewChat,
	showInspector,
	onToggleInspector,
}: CommandPaletteProps) {
	useEffect(() => {
		const down = (e: KeyboardEvent) => {
			const isCombo = e.key.toLowerCase() === "k" && (e.metaKey || e.ctrlKey);

			if (isCombo) {
				e.preventDefault();
				e.stopPropagation();
				setOpen((prev: boolean) => !prev);
				return;
			}

			if (e.key === "Escape") {
				setOpen(false);
			}
		};

		// Use capturing mode to ensure we intercept keydowns inside input elements
		document.addEventListener("keydown", down, true);
		return () => document.removeEventListener("keydown", down, true);
	}, [setOpen]);

	if (!open) return null;

	return (
		<div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 font-sans select-none">
			{/* Overlay click to close */}
			<div className="absolute inset-0" onClick={() => setOpen(false)} />

			<div className="relative w-full max-w-lg bg-popover border border-border rounded-2xl shadow-2xl overflow-hidden">
				<Command
					label="Global Command Menu"
					className="flex flex-col h-full text-foreground"
				>
					<div className="flex items-center border-b border-border px-4 py-3 gap-3">
						<Search className="h-4 w-4 text-muted-foreground/80 shrink-0" />
						<Command.Input
							autoFocus
							placeholder="Type a command or search panel..."
							className="w-full bg-transparent border-0 outline-none text-foreground placeholder-muted-foreground text-sm py-1 font-sans"
						/>
						<span className="text-[10px] bg-secondary text-muted-foreground font-bold px-2 py-0.5 rounded border border-border shrink-0 uppercase select-none">
							ESC
						</span>
					</div>

					<Command.List className="max-h-72 overflow-y-auto p-2.5 space-y-1">
						<Command.Empty className="py-6 text-center text-xs text-muted-foreground/60 select-none">
							No results found.
						</Command.Empty>

						<div className="text-[9px] font-extrabold text-muted-foreground/60 uppercase tracking-widest px-2.5 py-1.5 select-none">
							Navigation Actions
						</div>

						<Command.Item
							onSelect={() => {
								onNavigate("chat");
								setOpen(false);
							}}
							className="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold text-muted-foreground hover:text-foreground hover:bg-secondary cursor-pointer transition-colors select-none"
						>
							<MessageSquare className="h-4 w-4 text-primary shrink-0" />
							<span>Go to Chat & Consultation</span>
						</Command.Item>

						<Command.Item
							onSelect={() => {
								onNavigate("sessions");
								setOpen(false);
							}}
							className="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold text-muted-foreground hover:text-foreground hover:bg-secondary cursor-pointer transition-colors select-none"
						>
							<Calendar className="h-4 w-4 text-amber-400 shrink-0" />
							<span>Go to Session History Log</span>
						</Command.Item>

						<Command.Item
							onSelect={() => {
								onNavigate("tools");
								setOpen(false);
							}}
							className="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold text-muted-foreground hover:text-foreground hover:bg-secondary cursor-pointer transition-colors select-none"
						>
							<Wrench className="h-4 w-4 text-emerald-400 shrink-0" />
							<span>Go to Tool Security Policies</span>
						</Command.Item>

						<Command.Item
							onSelect={() => {
								onNavigate("workspace");
								setOpen(false);
							}}
							className="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold text-muted-foreground hover:text-foreground hover:bg-secondary cursor-pointer transition-colors select-none"
						>
							<Database className="h-4 w-4 text-violet-400 shrink-0" />
							<span>Go to Workspace Landmarks Map</span>
						</Command.Item>

						<Command.Item
							onSelect={() => {
								onNavigate("settings", "profile");
								setOpen(false);
							}}
							className="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold text-muted-foreground hover:text-foreground hover:bg-secondary cursor-pointer transition-colors select-none"
						>
							<Sliders className="h-4 w-4 text-pink-400 shrink-0" />
							<span>Go to OpenNivara Settings Hub</span>
						</Command.Item>

						<div className="h-px bg-border my-2 mx-2" />

						<div className="text-[9px] font-extrabold text-muted-foreground/60 uppercase tracking-widest px-2.5 py-1.5 select-none">
							Settings Sub-panels
						</div>

						<Command.Item
							onSelect={() => {
								onNavigate("settings", "profile");
								setOpen(false);
							}}
							className="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold text-muted-foreground hover:text-foreground hover:bg-secondary cursor-pointer transition-colors select-none"
						>
							<User className="h-4 w-4 text-sky-400 shrink-0" />
							<span>Open Settings: User Identity</span>
						</Command.Item>

						<Command.Item
							onSelect={() => {
								onNavigate("settings", "style");
								setOpen(false);
							}}
							className="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold text-muted-foreground hover:text-foreground hover:bg-secondary cursor-pointer transition-colors select-none"
						>
							<Palette className="h-4 w-4 text-teal-400 shrink-0" />
							<span>Open Settings: Response Style Guidelines</span>
						</Command.Item>

						<Command.Item
							onSelect={() => {
								onNavigate("settings", "preferences");
								setOpen(false);
							}}
							className="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold text-muted-foreground hover:text-foreground hover:bg-secondary cursor-pointer transition-colors select-none"
						>
							<Settings className="h-4 w-4 text-amber-400 shrink-0" />
							<span>Open Settings: Topic Preferences Triggers</span>
						</Command.Item>

						<Command.Item
							onSelect={() => {
								onNavigate("settings", "contexts");
								setOpen(false);
							}}
							className="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold text-muted-foreground hover:text-foreground hover:bg-secondary cursor-pointer transition-colors select-none"
						>
							<BookOpen className="h-4 w-4 text-primary shrink-0" />
							<span>Open Settings: Project Goals & Contexts</span>
						</Command.Item>

						<Command.Item
							onSelect={() => {
								onNavigate("settings", "paths");
								setOpen(false);
							}}
							className="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold text-muted-foreground hover:text-foreground hover:bg-secondary cursor-pointer transition-colors select-none"
						>
							<Info className="h-4 w-4 text-purple-400 shrink-0" />
							<span>Open Settings: Config Files Explorer</span>
						</Command.Item>

						<div className="h-px bg-border my-2 mx-2" />

						<div className="text-[9px] font-extrabold text-muted-foreground/60 uppercase tracking-widest px-2.5 py-1.5 select-none">
							Specialized Actions
						</div>

						<Command.Item
							onSelect={() => {
								onNewChat();
								setOpen(false);
							}}
							className="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold text-muted-foreground hover:text-foreground hover:bg-secondary cursor-pointer transition-colors select-none"
						>
							<Plus className="h-4 w-4 text-emerald-400 shrink-0" />
							<span>Initialize Clean Consultation Chat</span>
						</Command.Item>

						<Command.Item
							onSelect={() => {
								onNavigate("chat");
								if (onToggleInspector && !showInspector) {
									onToggleInspector();
								}
								setOpen(false);
							}}
							className="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold text-muted-foreground hover:text-foreground hover:bg-secondary cursor-pointer transition-colors select-none"
						>
							<Sparkles className="h-4 w-4 text-primary shrink-0 animate-pulse" />
							<span>Open Chat Context Inspector Drawer</span>
						</Command.Item>
					</Command.List>
				</Command>
			</div>
		</div>
	);
}
