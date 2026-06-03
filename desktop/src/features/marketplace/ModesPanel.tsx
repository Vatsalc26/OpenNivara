import {
	AlertTriangle,
	CheckCircle2,
	FolderMinus,
	FolderPlus,
	Layout,
	Palette,
	Plus,
	Sliders,
	Sparkles,
} from "lucide-react";
import type React from "react";
import { useEffect, useState } from "react";
import {
	getPackActivationCapabilities,
	type InstalledPack,
	type ModesFile,
	type OpenNivaraTheme,
	type PackActivationCapabilities,
} from "@/api/marketplaceClient";
import { Card } from "@/components/ui/card";

interface ModesPanelProps {
	modesFile: ModesFile;
	installedPacks: InstalledPack[];
	onSetActive: (id: string) => void;
	onCreateMode: (mode: {
		id: string;
		name: string;
		description: string;
		enabled_pack_ids: string[];
		theme_id?: string | null;
		style_preset_id?: string | null;
	}) => void;
	onAddPack: (modeId: string, packId: string) => void;
	onRemovePack: (modeId: string, packId: string) => void;
	activeTheme: OpenNivaraTheme | null;
	onInstallBuiltin?: (packId: string, packName: string) => void;
	onUpdateTheme: (modeId: string, themeId: string | null) => void;
	onUpdateStylePack: (modeId: string, stylePackId: string | null) => void;
}

export function ModesPanel({
	modesFile,
	installedPacks,
	onSetActive,
	onCreateMode,
	onAddPack,
	onRemovePack,
	activeTheme: _activeTheme,
	onInstallBuiltin,
	onUpdateTheme,
	onUpdateStylePack,
}: ModesPanelProps) {
	const [newModeName, setNewModeName] = useState("");
	const [newModeId, setNewModeId] = useState("");
	const [showAddForm, setShowAddForm] = useState(false);
	const [selectedPackId, setSelectedPackId] = useState<Record<string, string>>(
		{},
	); // modeId -> packId
	const [packCapabilities, setPackCapabilities] = useState<
		Record<string, PackActivationCapabilities>
	>({});

	useEffect(() => {
		async function fetchCaps() {
			const caps: Record<string, PackActivationCapabilities> = {};
			for (const pack of installedPacks) {
				if (pack.enabled) {
					try {
						const cap = await getPackActivationCapabilities(pack.id);
						caps[pack.id] = cap;
					} catch (err) {
						console.error(
							`Failed to get capabilities for pack ${pack.id}`,
							err,
						);
					}
				}
			}
			setPackCapabilities(caps);
		}
		fetchCaps();
	}, [installedPacks]);

	const handleCreate = (e: React.FormEvent) => {
		e.preventDefault();
		if (!newModeId || !newModeName) return;

		// Enforce snake case for ID
		const sanitizedId = newModeId
			.trim()
			.toLowerCase()
			.replace(/[^a-z0-9_-]/g, "_");

		onCreateMode({
			id: sanitizedId,
			name: newModeName.trim(),
			description: "User created custom mode.",
			enabled_pack_ids: [],
		});

		setNewModeId("");
		setNewModeName("");
		setShowAddForm(false);
	};

	return (
		<div className="space-y-6 select-none">
			{/* Top action grid: Active status & custom mode creator */}
			<div className="grid grid-cols-1 md:grid-cols-3 gap-5">
				{/* Active Mode Summary Panel */}
				<Card className="md:col-span-2 bg-card border-border/40 p-5 rounded-2xl flex flex-col justify-between relative overflow-hidden">
					<div className="absolute -top-10 -right-10 h-24 w-24 rounded-full bg-primary/5 blur-xl animate-pulse" />
					<div className="space-y-3 relative">
						<h3 className="text-xs font-extrabold uppercase tracking-widest text-primary flex items-center gap-1.5 leading-none">
							<Sparkles className="h-4 w-4 animate-pulse text-primary" />
							<span>Active Mode</span>
						</h3>
						{(() => {
							const active = modesFile.modes.find(
								(m) => m.id === modesFile.active_mode,
							);
							if (!active) return null;
							return (
								<div className="space-y-2">
									<div className="text-lg font-black text-foreground uppercase tracking-wide">
										{active.name}
									</div>
									<p className="text-xs text-muted-foreground font-semibold leading-normal">
										{active.description}
									</p>
									<div className="flex flex-wrap items-center gap-4.5 text-[10px] text-muted-foreground font-bold uppercase tracking-wider pt-2">
										<span className="flex items-center gap-1.5">
											<Layout className="h-3.5 w-3.5 text-muted-foreground" />
											<span>
												{active.enabled_pack_ids.length} Packs in Mode
											</span>
										</span>
										<span className="flex items-center gap-1.5">
											<Palette className="h-3.5 w-3.5 text-muted-foreground" />
											<span>Theme: {active.theme_id || "default"}</span>
										</span>
									</div>
								</div>
							);
						})()}
					</div>
				</Card>

				{/* Add custom mode button / form */}
				<Card className="bg-card border-border p-5 rounded-2xl flex flex-col justify-center">
					{!showAddForm ? (
						<button
							onClick={() => setShowAddForm(true)}
							className="w-full h-full flex flex-col items-center justify-center gap-2 py-6 text-muted-foreground hover:text-primary group border border-border border-dashed rounded-xl hover:border-primary/20 transition-all duration-300 cursor-pointer"
						>
							<Plus className="h-6 w-6 text-muted-foreground group-hover:text-primary transition-colors" />
							<span className="text-[10px] font-extrabold uppercase tracking-widest leading-none">
								Create Mode
							</span>
						</button>
					) : (
						<form onSubmit={handleCreate} className="space-y-3">
							<h4 className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">
								New Mode Details
							</h4>
							<div className="space-y-2">
								<input
									type="text"
									placeholder="Unique ID (e.g. game_dev)"
									value={newModeId}
									onChange={(e) => setNewModeId(e.target.value)}
									className="w-full bg-secondary border border-border/40 px-3 py-2 rounded-xl text-xs outline-none focus:border-primary/30 text-foreground placeholder-muted-foreground"
									required
								/>
								<input
									type="text"
									placeholder="Friendly Name (e.g. Game Dev Mode)"
									value={newModeName}
									onChange={(e) => setNewModeName(e.target.value)}
									className="w-full bg-secondary border border-border/40 px-3 py-2 rounded-xl text-xs outline-none focus:border-primary/30 text-foreground placeholder-muted-foreground"
									required
								/>
							</div>
							<div className="flex gap-2 shrink-0 pt-1">
								<button
									type="button"
									onClick={() => setShowAddForm(false)}
									className="flex-1 py-1.5 rounded-lg bg-secondary text-muted-foreground hover:text-foreground text-[10px] font-extrabold uppercase transition-colors cursor-pointer"
								>
									Cancel
								</button>
								<button
									type="submit"
									className="flex-1 py-1.5 rounded-lg bg-primary hover:bg-primary/90 text-primary-foreground text-[10px] font-extrabold uppercase transition-colors cursor-pointer"
								>
									Create
								</button>
							</div>
						</form>
					)}
				</Card>
			</div>

			{/* Modes list tracking grid */}
			<div className="space-y-4">
				<h4 className="text-[10px] font-extrabold text-muted-foreground uppercase tracking-widest">
					Available Modes
				</h4>
				<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
					{modesFile.modes.map((mode) => {
						const isActive = mode.id === modesFile.active_mode;

						// Separate installed packs in mode from missing referenced packs
						const packsInMode: InstalledPack[] = [];
						const missingPackIds: string[] = [];

						mode.enabled_pack_ids.forEach((packId) => {
							const p = installedPacks.find((ip) => ip.id === packId);
							if (p) {
								packsInMode.push(p);
							} else {
								missingPackIds.push(packId);
							}
						});

						const hasDisabledPacks = packsInMode.some((p) => !p.enabled);

						// Filter installed packs that can be added to this mode (must be enabled)
						const availablePacksToAdd = installedPacks.filter(
							(ip) => ip.enabled && !mode.enabled_pack_ids.includes(ip.id),
						);

						return (
							<Card
								key={mode.id}
								className={`p-5 rounded-2xl flex flex-col justify-between gap-5 relative overflow-hidden transition-all duration-300 ${
									isActive
										? "bg-card border-2 border-primary/40 shadow shadow-primary/5"
										: "bg-card border border-border/40 hover:bg-card/85"
								}`}
							>
								<div className="space-y-3">
									{/* Mode title & activation controls */}
									<div className="flex justify-between items-center gap-2">
										<div className="flex items-center gap-1.5 flex-wrap">
											<h3 className="font-extrabold text-sm text-foreground flex items-center gap-1.5 leading-none">
												<Sliders
													className={`h-4 w-4 shrink-0 ${isActive ? "text-primary" : "text-muted-foreground/80"}`}
												/>
												<span>{mode.name}</span>
											</h3>
											{hasDisabledPacks && (
												<span className="flex items-center gap-1 text-[8px] text-amber-500 font-bold uppercase leading-none bg-amber-500/10 border border-amber-500/20 px-1.5 py-0.5 rounded">
													<AlertTriangle className="h-3 w-3 text-amber-500 shrink-0" />
													<span>Pack disabled</span>
												</span>
											)}
										</div>

										{!isActive ? (
											<button
												onClick={() => onSetActive(mode.id)}
												className="text-[9px] bg-secondary hover:bg-secondary/70 text-muted-foreground hover:text-foreground border border-border/40 px-2.5 py-1.5 rounded-xl font-extrabold uppercase tracking-wide transition-all cursor-pointer"
											>
												Activate
											</button>
										) : (
											<span className="flex items-center gap-1 text-[9px] text-primary font-extrabold uppercase leading-none">
												<CheckCircle2 className="h-3 w-3 text-primary" />
												<span>Active</span>
											</span>
										)}
									</div>

									<p className="text-xs text-muted-foreground leading-normal font-medium">
										{mode.description}
									</p>

									{/* Theme and Style configuration controls */}
									{mode.id !== "default" && (
										<div className="grid grid-cols-2 gap-3 pt-2.5 border-t border-border/40">
											<div className="space-y-1">
												<label className="text-[9px] font-extrabold text-muted-foreground uppercase tracking-widest leading-none block">
													Theme Source
												</label>
												<select
													value={mode.theme_id || ""}
													onChange={(e) =>
														onUpdateTheme(mode.id, e.target.value || null)
													}
													className="w-full bg-secondary text-foreground border border-border/40 rounded-lg px-2 py-1 text-[11px] outline-none focus:border-primary/40 cursor-pointer font-medium"
												>
													<option value="">Default</option>
													{mode.enabled_pack_ids.map((packId) => {
														const p = installedPacks.find(
															(ip) => ip.id === packId,
														);
														const cap = packCapabilities[packId];
														if (p && cap?.has_theme && cap?.theme_id) {
															return (
																<option key={cap.theme_id} value={cap.theme_id}>
																	{cap.theme_name || cap.theme_id} ({p.name})
																</option>
															);
														}
														return null;
													})}
												</select>
											</div>

											<div className="space-y-1">
												<label className="text-[9px] font-extrabold text-muted-foreground uppercase tracking-widest leading-none block">
													Style Source
												</label>
												<select
													value={mode.style_pack_id || ""}
													onChange={(e) =>
														onUpdateStylePack(mode.id, e.target.value || null)
													}
													className="w-full bg-secondary text-foreground border border-border/40 rounded-lg px-2 py-1 text-[11px] outline-none focus:border-primary/40 cursor-pointer font-medium"
												>
													<option value="">User Style</option>
													{mode.enabled_pack_ids.map((packId) => {
														const p = installedPacks.find(
															(ip) => ip.id === packId,
														);
														const cap = packCapabilities[packId];
														if (p && cap?.has_style) {
															return (
																<option key={p.id} value={p.id}>
																	Style from {p.name}
																</option>
															);
														}
														return null;
													})}
												</select>
											</div>
										</div>
									)}

									{/* Packs in this Mode list */}
									<div className="space-y-2 pt-2.5 border-t border-border/40">
										<span className="text-[9px] font-extrabold text-muted-foreground uppercase tracking-widest leading-none block">
											Packs in this Mode ({mode.enabled_pack_ids.length})
										</span>

										{mode.enabled_pack_ids.length === 0 ? (
											<div className="text-[10px] text-muted-foreground/60 italic font-medium">
												No packs added yet. Everything behaves as protected
												defaults.
											</div>
										) : (
											<div className="flex flex-wrap gap-1">
												{packsInMode.map((p) => {
													const isPackDisabled = !p.enabled;
													return (
														<span
															key={p.id}
															className={`text-[9px] border font-bold px-2 py-0.5 rounded-lg flex items-center gap-1.5 leading-none shadow-sm ${
																isPackDisabled
																	? "bg-amber-500/10 border-amber-500/20 text-amber-500"
																	: "bg-secondary border border-border/40 text-foreground"
															}`}
														>
															<span>{p.name}</span>
															{isPackDisabled && (
																<span className="text-[7px] bg-amber-500 text-black px-1 rounded uppercase tracking-wider font-extrabold shrink-0">
																	Disabled
																</span>
															)}
															<button
																onClick={() => onRemovePack(mode.id, p.id)}
																className="hover:text-rose-400 text-muted-foreground/60 shrink-0 font-bold transition-colors cursor-pointer"
																title="Remove Pack"
															>
																<FolderMinus className="h-3 w-3" />
															</button>
														</span>
													);
												})}

												{/* Display missing packs warnings */}
												{missingPackIds.map((packId) => {
													const isBuiltin =
														packId === "coding_basics" ||
														packId === "study_coach";
													const builtinName =
														packId === "coding_basics"
															? "Coding Basics Pack"
															: "Study Coach Pack";
													return (
														<div
															key={packId}
															className="w-full flex items-center justify-between gap-2 text-[9px] bg-rose-500/10 border border-rose-500/20 p-2 rounded-xl text-rose-400 font-extrabold uppercase leading-none shadow-sm mt-1"
														>
															<span className="flex items-center gap-1.5">
																<AlertTriangle className="h-3.5 w-3.5 text-rose-400 shrink-0" />
																<span>Missing pack: {packId}</span>
															</span>
															<div className="flex items-center gap-1.5 shrink-0">
																{isBuiltin && onInstallBuiltin && (
																	<button
																		onClick={() =>
																			onInstallBuiltin(packId, builtinName)
																		}
																		className="px-2 py-1 rounded bg-rose-500 hover:bg-rose-400 text-white font-extrabold text-[8px] uppercase tracking-wide cursor-pointer transition-colors leading-none"
																	>
																		Install Built-in
																	</button>
																)}
																<button
																	onClick={() => onRemovePack(mode.id, packId)}
																	className="px-2 py-1 rounded bg-secondary hover:bg-secondary/80 text-foreground border border-border/40 font-extrabold text-[8px] uppercase tracking-wide cursor-pointer transition-colors leading-none"
																>
																	Remove from Mode
																</button>
															</div>
														</div>
													);
												})}
											</div>
										)}
									</div>
								</div>

								{/* Add Packs dropdown selection flow */}
								{mode.id === "default" ? (
									<div className="pt-3.5 border-t border-border/40 text-center py-2 select-none">
										<span className="text-[9px] bg-primary/10 border border-primary/20 text-primary font-extrabold uppercase tracking-wide px-3 py-1.5 rounded-xl">
											Default Mode is Protected
										</span>
										<p className="text-[10px] text-muted-foreground font-semibold mt-2 max-w-[280px] leading-normal mx-auto">
											Default Mode is the safe base mode. Create or select
											another mode to add packs.
										</p>
									</div>
								) : (
									<div className="pt-3.5 border-t border-border/40 flex flex-col gap-2">
										<span className="text-[9px] font-extrabold text-muted-foreground uppercase tracking-widest leading-none block">
											Add Installed Pack to Mode
										</span>

										{installedPacks.length === 0 ? (
											<div className="text-[10px] text-muted-foreground/50 italic font-medium leading-normal">
												No installed packs yet. Install packs from Featured or
												Import Local.
											</div>
										) : availablePacksToAdd.length === 0 ? (
											<div className="text-[10px] text-muted-foreground/50 italic font-medium leading-normal">
												All active installed packs are already in this mode.
											</div>
										) : (
											<div className="space-y-2.5">
												{/* Obvious dropdown selection flow */}
												<div className="flex gap-2">
													<select
														value={selectedPackId[mode.id] || ""}
														onChange={(e) =>
															setSelectedPackId((prev) => ({
																...prev,
																[mode.id]: e.target.value,
															}))
														}
														className="flex-1 bg-secondary text-foreground border border-border/40 rounded-xl px-2.5 py-1.5 text-xs outline-none focus:border-primary/40 cursor-pointer font-medium"
													>
														<option value="" disabled>
															-- Select Installed Pack --
														</option>
														{availablePacksToAdd.map((ip) => (
															<option key={ip.id} value={ip.id}>
																{ip.name}
															</option>
														))}
													</select>
													<button
														onClick={() => {
															const pId = selectedPackId[mode.id];
															if (pId) {
																onAddPack(mode.id, pId);
																setSelectedPackId((prev) => ({
																	...prev,
																	[mode.id]: "",
																}));
															}
														}}
														disabled={!selectedPackId[mode.id]}
														className="px-3.5 py-1.5 rounded-xl bg-primary hover:bg-primary/90 text-primary-foreground font-extrabold text-[10px] uppercase transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed leading-none shrink-0"
													>
														Add
													</button>
												</div>

												{/* Alternate quick buttons for immediate click additions */}
												<div className="flex flex-wrap gap-1">
													{availablePacksToAdd.map((ip) => (
														<button
															key={ip.id}
															onClick={() => onAddPack(mode.id, ip.id)}
															className="text-[9px] bg-primary/10 hover:bg-primary/20 border border-primary/20 hover:border-primary/40 text-primary font-bold px-2 py-1 rounded-lg flex items-center gap-1.5 transition-all cursor-pointer leading-none"
														>
															<FolderPlus className="h-3 w-3 shrink-0" />
															<span>{ip.name}</span>
														</button>
													))}
												</div>
											</div>
										)}
									</div>
								)}
							</Card>
						);
					})}
				</div>
			</div>
		</div>
	);
}
