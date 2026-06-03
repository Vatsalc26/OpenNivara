import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Sparkles, Square, X } from "lucide-react";
import type React from "react";
import { useEffect, useState } from "react";

interface TitleBarProps {
	activeView: string;
}

export function TitleBar({ activeView }: TitleBarProps) {
	const [isMaximized, setIsMaximized] = useState(false);

	// Safely resolve tauri window reference if running inside Tauri
	const isTauri =
		typeof window !== "undefined" &&
		(window as any).__TAURI_INTERNALS__ !== undefined;
	const appWindow = isTauri ? getCurrentWindow() : null;

	useEffect(() => {
		if (!appWindow) return;
		let mounted = true;
		appWindow
			.isMaximized()
			.then((value) => {
				if (mounted) setIsMaximized(value);
			})
			.catch(console.error);
		return () => {
			mounted = false;
		};
	}, [appWindow]);

	const handleMinimize = async (e: React.MouseEvent) => {
		e.stopPropagation();
		if (!appWindow) return;
		try {
			await appWindow.minimize();
		} catch (e) {
			console.error(e);
		}
	};

	const handleMaximize = async (e: React.MouseEvent) => {
		e.stopPropagation();
		if (!appWindow) return;
		try {
			await appWindow.toggleMaximize();
			const max = await appWindow.isMaximized();
			setIsMaximized(max);
		} catch (e) {
			console.error(e);
		}
	};

	const handleClose = async (e: React.MouseEvent) => {
		e.stopPropagation();
		if (!appWindow) return;
		try {
			await appWindow.close();
		} catch (e) {
			console.error(e);
		}
	};

	const getPageTitle = () => {
		switch (activeView) {
			case "chat":
				return "Consultation & Chat";
			case "sessions":
				return "Session Log History";
			case "tools":
				return "Tool Security & Permissions";
			case "workspace":
				return "Workspace Knowledge Map";
			case "settings":
				return "OpenNivara Hub Settings";
			case "marketplace":
				return "OpenNivara Store";
			default:
				return "Assistant";
		}
	};

	const stopProp = (e: React.SyntheticEvent) => {
		e.stopPropagation();
	};

	return (
		<div className="h-10 w-full bg-sidebar border-b border-sidebar-border select-none flex items-center justify-between shrink-0 text-muted-foreground font-sans text-xs px-3">
			{/* Brand logo & page title */}
			<div
				data-tauri-drag-region
				className="flex items-center gap-2.5 select-none h-full"
			>
				<Sparkles className="h-3.5 w-3.5 text-primary animate-pulse shrink-0 pointer-events-none" />
				<span className="font-extrabold text-[10px] tracking-widest text-foreground uppercase font-heading pointer-events-none">
					OPENNIVARA
				</span>
				<span className="h-3 w-px bg-sidebar-border pointer-events-none"></span>
				<span className="text-[11px] font-semibold text-muted-foreground capitalize pointer-events-none">
					{getPageTitle()}
				</span>
			</div>

			{/* Draggable center area filler */}
			<div data-tauri-drag-region className="flex-1 h-full select-none" />

			{/* Native window operations controls */}
			{isTauri && (
				<div
					className="flex items-center h-full shrink-0"
					onMouseDown={stopProp}
					onPointerDown={stopProp}
				>
					<button
						type="button"
						onClick={handleMinimize}
						onMouseDown={stopProp}
						onPointerDown={stopProp}
						className="h-full w-10 flex items-center justify-center hover:bg-sidebar-accent hover:text-foreground transition-colors cursor-pointer"
						title="Minimize"
					>
						<Minus className="h-3.5 w-3.5 pointer-events-none" />
					</button>
					<button
						type="button"
						onClick={handleMaximize}
						onMouseDown={stopProp}
						onPointerDown={stopProp}
						className="h-full w-10 flex items-center justify-center hover:bg-sidebar-accent hover:text-foreground transition-colors cursor-pointer"
						title={isMaximized ? "Restore Down" : "Maximize"}
					>
						<Square className="h-3 w-3 pointer-events-none" />
					</button>
					<button
						type="button"
						onClick={handleClose}
						onMouseDown={stopProp}
						onPointerDown={stopProp}
						className="h-full w-10 flex items-center justify-center hover:bg-rose-600 hover:text-white transition-colors cursor-pointer"
						title="Close"
					>
						<X className="h-3.5 w-3.5 pointer-events-none" />
					</button>
				</div>
			)}
		</div>
	);
}
