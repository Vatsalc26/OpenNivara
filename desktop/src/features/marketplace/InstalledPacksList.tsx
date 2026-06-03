import { FolderClosed } from "lucide-react";
import type { InstalledPack } from "@/api/marketplaceClient";
import { PackCard } from "./PackCard";

interface InstalledPacksListProps {
	packs: InstalledPack[];
	onUninstall: (id: string) => void;
	onAddToMode: (id: string) => void;
	onPreview: (pack: InstalledPack) => void;
	onToggleEnable: (id: string, enabled: boolean) => void;
	activeModeIsDefault?: boolean;
}

export function InstalledPacksList({
	packs,
	onUninstall,
	onAddToMode,
	onPreview,
	onToggleEnable,
	activeModeIsDefault = false,
}: InstalledPacksListProps) {
	if (packs.length === 0) {
		return (
			<div className="flex flex-col items-center justify-center py-16 space-y-4 bg-secondary/20 border border-border border-dashed rounded-2xl p-6">
				<div className="h-10 w-10 rounded-full bg-secondary/60 border border-border flex items-center justify-center">
					<FolderClosed className="h-5 w-5 text-muted-foreground" />
				</div>
				<div className="text-center space-y-1">
					<h4 className="text-xs font-bold text-foreground uppercase tracking-wide">
						No Installed Packs
					</h4>
					<p className="text-[11px] text-muted-foreground font-semibold max-w-[280px] leading-normal">
						Import local folders or try out built-in packs under the Featured
						tab.
					</p>
				</div>
			</div>
		);
	}

	return (
		<div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
			{packs.map((pack) => (
				<PackCard
					key={pack.id}
					pack={pack}
					isInstalled={true}
					onPreview={() => onPreview(pack)}
					onUninstall={() => onUninstall(pack.id)}
					onAddToMode={() => onAddToMode(pack.id)}
					onToggleEnable={(enabled) => onToggleEnable(pack.id, enabled)}
					activeModeIsDefault={activeModeIsDefault}
				/>
			))}
		</div>
	);
}
