import type React from "react";
import type { ReactNode } from "react";
import { CommandPalette } from "./CommandPalette";
import { Sidebar } from "./Sidebar";
import { TitleBar } from "./TitleBar";

interface AppShellProps {
	children: ReactNode;
	activeView: string;
	onNavigate: (view: string, tab?: any) => void;
	onNewChat: () => void;
	apiKeyReady: boolean;
	toolsEnabled: boolean;
	paletteOpen: boolean;
	setPaletteOpen: React.Dispatch<React.SetStateAction<boolean>>;
	showInspector?: boolean;
	onToggleInspector?: () => void;
}

export function AppShell({
	children,
	activeView,
	onNavigate,
	onNewChat,
	apiKeyReady,
	toolsEnabled,
	paletteOpen,
	setPaletteOpen,
	showInspector,
	onToggleInspector,
}: AppShellProps) {
	return (
		<div className="flex flex-col h-screen w-screen bg-background text-foreground overflow-hidden font-sans select-none">
			{/* Draggable Custom title bar */}
			<TitleBar activeView={activeView} />

			{/* Main Container */}
			<div className="flex flex-1 min-h-0 overflow-hidden">
				{/* Sidebar panel */}
				<Sidebar
					activeView={activeView}
					onNavigate={onNavigate}
					onNewChat={onNewChat}
					apiKeyReady={apiKeyReady}
					toolsEnabled={toolsEnabled}
				/>

				{/* Content Frame */}
				<main className="flex-1 flex flex-col min-w-0 bg-background">
					<div className="flex-1 overflow-hidden relative flex flex-col bg-background">
						{children}
					</div>
				</main>
			</div>

			{/* Global Command palette shortcut modal */}
			<CommandPalette
				open={paletteOpen}
				setOpen={setPaletteOpen}
				onNavigate={onNavigate}
				onNewChat={onNewChat}
				showInspector={showInspector}
				onToggleInspector={onToggleInspector}
			/>
		</div>
	);
}
