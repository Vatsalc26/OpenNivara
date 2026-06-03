import { Check, ChevronDown, Sparkles } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import type { ModesFile } from "@/api/marketplaceClient";

interface ModeSwitcherProps {
	modesFile: ModesFile | null;
	onSetActiveMode: (id: string) => void;
}

export function ModeSwitcher({
	modesFile,
	onSetActiveMode,
}: ModeSwitcherProps) {
	const [isOpen, setIsOpen] = useState(false);
	const dropdownRef = useRef<HTMLDivElement>(null);

	useEffect(() => {
		function handleClickOutside(event: MouseEvent) {
			if (
				dropdownRef.current &&
				!dropdownRef.current.contains(event.target as Node)
			) {
				setIsOpen(false);
			}
		}
		document.addEventListener("mousedown", handleClickOutside);
		return () => {
			document.removeEventListener("mousedown", handleClickOutside);
		};
	}, []);

	if (!modesFile) return null;

	const activeMode = modesFile.modes.find(
		(m) => m.id === modesFile.active_mode,
	) || {
		id: "default",
		name: "Default",
	};

	const handleSelect = (modeId: string) => {
		onSetActiveMode(modeId);
		setIsOpen(false);
	};

	return (
		<div
			className="relative shrink-0 select-none text-muted-foreground font-sans"
			ref={dropdownRef}
		>
			{/* Trigger Button */}
			<button
				onClick={() => setIsOpen((prev) => !prev)}
				className="w-full flex items-center justify-between gap-3 px-3 py-2 border border-sidebar-border bg-sidebar/80 hover:bg-sidebar-accent/60 text-foreground hover:text-foreground font-bold text-xs uppercase tracking-wider rounded-xl transition-all duration-300 cursor-pointer shadow hover:shadow-primary/10 leading-none focus:outline-none"
			>
				<div className="flex items-center gap-2">
					<Sparkles className="h-3.5 w-3.5 text-primary animate-pulse shrink-0" />
					<span className="truncate max-w-[100px]">{activeMode.name}</span>
				</div>
				<ChevronDown className="h-3 w-3 text-muted-foreground/80 transition-transform duration-300" />
			</button>

			{/* Floating Dropdown List Popover */}
			{isOpen && (
				<div className="absolute right-0 bottom-full mb-2.5 z-50 w-52 bg-popover/95 border border-border/90 backdrop-blur-xl rounded-2xl shadow-2xl p-1.5 space-y-0.5 max-h-60 overflow-y-auto animate-in fade-in slide-in-from-bottom-2 duration-200">
					<div className="px-3.5 py-2 border-b border-border/40 text-[9px] font-extrabold uppercase tracking-widest text-muted-foreground leading-none">
						Switch Mode
					</div>
					{modesFile.modes.map((mode) => {
						const isSelected = mode.id === modesFile.active_mode;
						return (
							<button
								key={mode.id}
								onClick={() => handleSelect(mode.id)}
								className={`w-full flex items-center justify-between px-3 py-2.5 rounded-xl text-left text-xs font-semibold tracking-wide transition-all cursor-pointer ${
									isSelected
										? "bg-primary/10 text-primary font-bold"
										: "text-muted-foreground hover:text-foreground hover:bg-secondary/40"
								}`}
							>
								<div className="flex flex-col gap-0.5 min-w-0 pr-2">
									<span className="truncate leading-tight text-foreground">
										{mode.name}
									</span>
									<span className="text-[9px] text-muted-foreground font-medium truncate leading-none">
										{mode.id}
									</span>
								</div>
								{isSelected && (
									<Check className="h-3.5 w-3.5 text-primary shrink-0" />
								)}
							</button>
						);
					})}
				</div>
			)}
		</div>
	);
}
