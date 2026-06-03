import { open as tauriOpenDialog } from "@tauri-apps/plugin-dialog";
import {
	Check,
	Download,
	Eye,
	FolderPlus,
	Palette,
	RefreshCw,
	ShieldCheck,
	Sparkles,
	Trash2,
	X,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import {
	applyTheme,
	type BuiltinPackSummary,
	initMarketplace,
	installBuiltinPack,
	installBuiltinTheme,
	installTheme,
	listBuiltinPacks,
	listThemeStoreItems,
	type PackPreview,
	previewBuiltinPack,
	type ThemeStoreItem,
	uninstallTheme,
} from "@/api/marketplaceClient";
import { Card } from "@/components/ui/card";
import { useOpenNivaraTheme } from "@/theme/ThemeProvider";

type StoreTab = "themes" | "skills" | "installed";

interface StoreViewProps {
	defaultTab?: StoreTab;
}

export function StoreView({ defaultTab = "themes" }: StoreViewProps) {
	const [activeTab, setActiveTab] = useState<StoreTab>(defaultTab);
	const [themes, setThemes] = useState<ThemeStoreItem[]>([]);
	const [skillPacks, setSkillPacks] = useState<
		Array<{ pack: BuiltinPackSummary; preview: PackPreview }>
	>([]);
	const [loading, setLoading] = useState(true);
	const [selectedTheme, setSelectedTheme] = useState<ThemeStoreItem | null>(
		null,
	);
	const [selectedSkillPack, setSelectedSkillPack] = useState<{
		pack: BuiltinPackSummary;
		preview: PackPreview;
	} | null>(null);
	const [workingThemeId, setWorkingThemeId] = useState<string | null>(null);
	const { refreshTheme } = useOpenNivaraTheme();

	useEffect(() => {
		setActiveTab(defaultTab);
	}, [defaultTab]);

	const loadThemes = useCallback(async () => {
		setLoading(true);
		try {
			await initMarketplace();
			const [themeItems, builtinPacks] = await Promise.all([
				listThemeStoreItems(),
				listBuiltinPacks().catch(() => []),
			]);
			const previews = await Promise.all(
				builtinPacks.map(async (pack) => ({
					pack,
					preview: await previewBuiltinPack(pack.id),
				})),
			);
			setThemes(themeItems);
			setSkillPacks(
				previews.filter((item) => item.preview.additions.skills_count > 0),
			);
		} catch (err: any) {
			toast.error(`Failed to load themes: ${err?.message || err}`);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		loadThemes();
	}, [loadThemes]);

	const visibleThemes = useMemo(() => {
		if (activeTab === "installed") {
			return themes.filter((theme) => theme.installed);
		}
		return themes;
	}, [activeTab, themes]);

	const handleInstall = async (theme: ThemeStoreItem) => {
		setWorkingThemeId(theme.id);
		try {
			if (theme.source_kind === "builtin") {
				await installBuiltinTheme(theme.id);
			} else {
				await installTheme(theme.id);
			}
			toast.success(`${theme.name} installed.`);
			await loadThemes();
		} catch (err: any) {
			toast.error(`Failed to install theme: ${err?.message || err}`);
		} finally {
			setWorkingThemeId(null);
		}
	};

	const handleImportLocalTheme = async () => {
		const selected = await tauriOpenDialog({
			directory: true,
			multiple: false,
			title: "Choose a local theme folder",
		});
		if (!selected || Array.isArray(selected)) return;

		setWorkingThemeId("local");
		try {
			await installTheme(selected);
			toast.success("Local theme installed.");
			await loadThemes();
		} catch (err: any) {
			toast.error(`Failed to import theme: ${err?.message || err}`);
		} finally {
			setWorkingThemeId(null);
		}
	};

	const handleInstallSkillPack = async (packId: string, packName: string) => {
		setWorkingThemeId(packId);
		try {
			await installBuiltinPack(packId);
			toast.success(
				`${packName} installed. Enable skills in Settings -> Skills.`,
			);
			await loadThemes();
		} catch (err: any) {
			toast.error(`Failed to install skill pack: ${err?.message || err}`);
		} finally {
			setWorkingThemeId(null);
		}
	};

	const handleApply = async (theme: ThemeStoreItem) => {
		setWorkingThemeId(theme.id);
		try {
			if (!theme.installed) {
				await handleInstall(theme);
			}
			await applyTheme(theme.id);
			await refreshTheme();
			await loadThemes();
			toast.success(`${theme.name} applied. This changes UI only.`);
		} catch (err: any) {
			toast.error(`Failed to apply theme: ${err?.message || err}`);
		} finally {
			setWorkingThemeId(null);
		}
	};

	const handleUninstall = async (theme: ThemeStoreItem) => {
		setWorkingThemeId(theme.id);
		try {
			await uninstallTheme(theme.id);
			await refreshTheme();
			await loadThemes();
			toast.success(`${theme.name} uninstalled.`);
			setSelectedTheme(null);
		} catch (err: any) {
			toast.error(`Failed to uninstall theme: ${err?.message || err}`);
		} finally {
			setWorkingThemeId(null);
		}
	};

	return (
		<div className="flex h-full flex-col bg-background text-foreground">
			<header className="border-b border-border px-6 py-4">
				<div className="flex flex-wrap items-center justify-between gap-3">
					<div>
						<h1 className="text-xl font-semibold">Store</h1>
						<p className="text-sm text-muted-foreground">
							Discover visual themes for OpenNivara. Themes change the UI only.
						</p>
					</div>
					<div className="flex gap-2">
						<button
							type="button"
							onClick={handleImportLocalTheme}
							className="inline-flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:bg-muted"
						>
							<FolderPlus size={16} />
							Import Local Theme
						</button>
						<button
							type="button"
							onClick={loadThemes}
							className="inline-flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:bg-muted"
						>
							<RefreshCw size={16} />
							Refresh
						</button>
					</div>
				</div>
			</header>

			<nav className="flex gap-2 border-b border-border px-6 py-3">
				<button
					type="button"
					onClick={() => setActiveTab("themes")}
					className={`rounded-md px-3 py-2 text-sm ${activeTab === "themes" ? "bg-primary text-primary-foreground" : "hover:bg-muted"}`}
				>
					Themes
				</button>
				<button
					type="button"
					onClick={() => setActiveTab("skills")}
					className={`rounded-md px-3 py-2 text-sm ${activeTab === "skills" ? "bg-primary text-primary-foreground" : "hover:bg-muted"}`}
				>
					Skill Packs
				</button>
				<button
					type="button"
					onClick={() => setActiveTab("installed")}
					className={`rounded-md px-3 py-2 text-sm ${activeTab === "installed" ? "bg-primary text-primary-foreground" : "hover:bg-muted"}`}
				>
					Installed Themes
				</button>
			</nav>

			<main className="flex-1 overflow-auto p-6">
				{loading ? (
					<div className="text-sm text-muted-foreground">Loading themes...</div>
				) : activeTab === "skills" ? (
					skillPacks.length === 0 ? (
						<Card className="p-6 text-sm text-muted-foreground">
							No skill packs are available.
						</Card>
					) : (
						<div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
							{skillPacks.map(({ pack, preview }) => (
								<Card key={pack.id} className="flex flex-col gap-4 p-4">
									<div className="space-y-2">
										<div className="flex items-center justify-between gap-2">
											<h3 className="font-semibold">{pack.name}</h3>
											<Badge
												label={`${preview.additions.skills_count} skills`}
												tone="muted"
											/>
										</div>
										<p className="text-sm text-muted-foreground">
											{pack.description}
										</p>
										<div className="flex flex-wrap gap-2">
											<Badge label={pack.category} tone="muted" />
											<Badge
												label={`${examsCovered(preview).length || 1} exam areas`}
												tone="muted"
											/>
										</div>
										<SmallList
											label="Covers"
											items={examsCovered(preview)}
											empty="General India study workflows"
										/>
										<div className="flex flex-wrap gap-2">
											<Badge label="Data-only" tone="muted" />
											<Badge label="No executable code" tone="muted" />
											<Badge label="No tool permission changes" tone="muted" />
											<Badge
												label={
													preview.skill_previews.some(
														(skill) => skill.metadata.freshness_sensitive,
													)
														? "Fresh-info labels"
														: "No network tools"
												}
												tone="muted"
											/>
										</div>
									</div>
									<div className="mt-auto flex flex-wrap justify-end gap-2">
										<button
											type="button"
											onClick={() => setSelectedSkillPack({ pack, preview })}
											className="inline-flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:bg-muted"
										>
											<Eye size={15} />
											Open Details
										</button>
										<button
											type="button"
											onClick={() => handleInstallSkillPack(pack.id, pack.name)}
											disabled={workingThemeId === pack.id}
											className="inline-flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground disabled:opacity-60"
										>
											<Download size={15} />
											Install Pack
										</button>
									</div>
								</Card>
							))}
						</div>
					)
				) : visibleThemes.length === 0 ? (
					<Card className="p-6 text-sm text-muted-foreground">
						No themes are installed yet.
					</Card>
				) : (
					<div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
						{visibleThemes.map((theme) => (
							<ThemeCard
								key={theme.id}
								theme={theme}
								working={workingThemeId === theme.id}
								onDetails={() => setSelectedTheme(theme)}
								onInstall={() => handleInstall(theme)}
								onApply={() => handleApply(theme)}
							/>
						))}
					</div>
				)}
			</main>

			{selectedTheme && (
				<ThemeDetailsDialog
					theme={selectedTheme}
					working={workingThemeId === selectedTheme.id}
					onClose={() => setSelectedTheme(null)}
					onInstall={() => handleInstall(selectedTheme)}
					onApply={() => handleApply(selectedTheme)}
					onUninstall={() => handleUninstall(selectedTheme)}
				/>
			)}
			{selectedSkillPack && (
				<SkillPackDetailsDialog
					pack={selectedSkillPack.pack}
					preview={selectedSkillPack.preview}
					working={workingThemeId === selectedSkillPack.pack.id}
					onClose={() => setSelectedSkillPack(null)}
					onInstall={() =>
						handleInstallSkillPack(
							selectedSkillPack.pack.id,
							selectedSkillPack.pack.name,
						)
					}
				/>
			)}
		</div>
	);
}

function SkillPackDetailsDialog({
	pack,
	preview,
	working,
	onClose,
	onInstall,
}: {
	pack: BuiltinPackSummary;
	preview: PackPreview;
	working: boolean;
	onClose: () => void;
	onInstall: () => void;
}) {
	const skillCount = preview.skill_previews.length;
	return (
		<div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
			<Card className="max-h-[90vh] w-full max-w-4xl overflow-auto p-5">
				<div className="mb-4 flex items-start justify-between gap-4">
					<div>
						<div className="mb-1 flex items-center gap-2 text-sm text-muted-foreground">
							<Sparkles size={16} />
							Skill Pack Details
						</div>
						<h2 className="text-xl font-semibold">{pack.name}</h2>
						<p className="text-sm text-muted-foreground">{pack.description}</p>
					</div>
					<button
						type="button"
						onClick={onClose}
						className="rounded-md p-2 hover:bg-muted"
						aria-label="Close skill pack details"
					>
						<X size={18} />
					</button>
				</div>

				<div className="grid gap-4 md:grid-cols-[0.85fr_1.15fr]">
					<div className="space-y-4 text-sm">
						<div>
							<h3 className="mb-2 font-medium">Pack Info</h3>
							<div className="space-y-1 text-muted-foreground">
								<div>Author: {pack.author}</div>
								<div>Version: {pack.version}</div>
								<div>Category: {pack.category}</div>
								<div>Skills: {skillCount}</div>
							</div>
						</div>
						<div>
							<h3 className="mb-2 flex items-center gap-2 font-medium">
								<ShieldCheck size={16} />
								Safety
							</h3>
							<div className="grid gap-2">
								<Badge label="Data-only skill pack" tone="muted" />
								<Badge label="No executable code" tone="muted" />
								<Badge label="No tool permission changes" tone="muted" />
								<Badge label="Install does not enable skills" tone="muted" />
							</div>
						</div>
						<SmallList
							label="Exam areas"
							items={examsCovered(preview)}
							empty="General India study workflows"
						/>
					</div>

					<div className="space-y-3">
						<h3 className="font-medium">Included Skills</h3>
						<div className="space-y-3">
							{preview.skill_previews.map((skill) => (
								<Card key={skill.id} className="space-y-2 p-3">
									<div className="flex flex-wrap items-center justify-between gap-2">
										<h4 className="text-sm font-semibold">{skill.name}</h4>
										<Badge
											label={skill.metadata.exam || skill.category}
											tone="muted"
										/>
									</div>
									<p className="text-xs text-muted-foreground">
										{skill.description}
									</p>
									<SmallList
										label="Best for"
										items={skill.store_preview.best_for}
										empty="Focused study help"
									/>
									<SmallList
										label="Try"
										items={skill.store_preview.sample_prompts}
										empty="Ask OpenNivara to use this skill"
									/>
									{skill.metadata.official_source_labels.length > 0 && (
										<SmallList
											label="Fresh-info labels"
											items={skill.metadata.official_source_labels}
											empty=""
										/>
									)}
								</Card>
							))}
						</div>
					</div>
				</div>

				<div className="mt-5 flex justify-end">
					<button
						type="button"
						onClick={onInstall}
						disabled={working}
						className="inline-flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground disabled:opacity-60"
					>
						<Download size={15} />
						Install Pack
					</button>
				</div>
			</Card>
		</div>
	);
}

function ThemeCard({
	theme,
	working,
	onDetails,
	onInstall,
	onApply,
}: {
	theme: ThemeStoreItem;
	working: boolean;
	onDetails: () => void;
	onInstall: () => void;
	onApply: () => void;
}) {
	return (
		<article>
			<Card className="flex h-full flex-col gap-4 p-4">
				<ThemePreview theme={theme} />
				<div className="flex flex-1 flex-col gap-2">
					<div className="flex items-start justify-between gap-3">
						<div>
							<h3 className="font-semibold">{theme.name}</h3>
							<p className="text-sm text-muted-foreground">
								{theme.description}
							</p>
						</div>
						<div className="flex flex-col items-end gap-1 text-xs">
							{theme.applied && <Badge label="Applied" tone="primary" />}
							{theme.installed && <Badge label="Installed" tone="muted" />}
						</div>
					</div>
					<div className="mt-auto flex flex-wrap gap-2 pt-2">
						<button
							type="button"
							onClick={onDetails}
							className="inline-flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:bg-muted"
						>
							<Eye size={15} />
							Open Details
						</button>
						{theme.installed ? (
							<button
								type="button"
								onClick={onApply}
								disabled={working || theme.applied}
								className="inline-flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground disabled:opacity-60"
							>
								<Check size={15} />
								{theme.applied ? "Applied" : "Apply Theme"}
							</button>
						) : (
							<button
								type="button"
								onClick={onInstall}
								disabled={working}
								className="inline-flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground disabled:opacity-60"
							>
								<Download size={15} />
								Install Theme
							</button>
						)}
					</div>
				</div>
			</Card>
		</article>
	);
}

function ThemeDetailsDialog({
	theme,
	working,
	onClose,
	onInstall,
	onApply,
	onUninstall,
}: {
	theme: ThemeStoreItem;
	working: boolean;
	onClose: () => void;
	onInstall: () => void;
	onApply: () => void;
	onUninstall: () => void;
}) {
	return (
		<div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
			<Card className="max-h-[90vh] w-full max-w-3xl overflow-auto p-5">
				<div className="mb-4 flex items-start justify-between gap-4">
					<div>
						<div className="mb-1 flex items-center gap-2 text-sm text-muted-foreground">
							<Palette size={16} />
							Theme Details
						</div>
						<h2 className="text-xl font-semibold">{theme.name}</h2>
						<p className="text-sm text-muted-foreground">{theme.description}</p>
					</div>
					<button
						type="button"
						onClick={onClose}
						className="rounded-md p-2 hover:bg-muted"
						aria-label="Close theme details"
					>
						<X size={18} />
					</button>
				</div>

				<div className="grid gap-4 md:grid-cols-[1.1fr_0.9fr]">
					<ThemePreview theme={theme} large />
					<div className="space-y-4 text-sm">
						<div>
							<h3 className="mb-2 font-medium">Theme Info</h3>
							<div className="space-y-1 text-muted-foreground">
								<div>Author: {theme.author}</div>
								<div>Version: {theme.version}</div>
								<div>Source: {theme.source_kind}</div>
							</div>
						</div>
						<div>
							<h3 className="mb-2 font-medium">UI areas affected</h3>
							<div className="grid grid-cols-2 gap-2 text-muted-foreground">
								<span>background</span>
								<span>primary/accent color</span>
								<span>card border/glow</span>
								<span>sidebar selected item</span>
								<span>buttons</span>
								<span>inputs</span>
							</div>
						</div>
						<div>
							<h3 className="mb-2 flex items-center gap-2 font-medium">
								<ShieldCheck size={16} />
								Safety
							</h3>
							<div className="grid gap-2">
								<Badge label="Data-only theme" tone="muted" />
								<Badge label="No executable code" tone="muted" />
								<Badge label="No tool permission changes" tone="muted" />
								<Badge label="No network requirement" tone="muted" />
							</div>
						</div>
					</div>
				</div>

				<div className="mt-5 flex flex-wrap justify-end gap-2">
					{theme.installed && (
						<button
							type="button"
							onClick={onUninstall}
							disabled={working}
							className="inline-flex items-center gap-2 rounded-md border border-destructive/50 px-3 py-2 text-sm text-destructive hover:bg-destructive/10 disabled:opacity-60"
						>
							<Trash2 size={15} />
							Uninstall
						</button>
					)}
					{theme.installed ? (
						<button
							type="button"
							onClick={onApply}
							disabled={working || theme.applied}
							className="inline-flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground disabled:opacity-60"
						>
							<Check size={15} />
							{theme.applied ? "Applied" : "Apply Theme"}
						</button>
					) : (
						<button
							type="button"
							onClick={onInstall}
							disabled={working}
							className="inline-flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground disabled:opacity-60"
						>
							<Download size={15} />
							Install Theme
						</button>
					)}
				</div>
			</Card>
		</div>
	);
}

function ThemePreview({
	theme,
	large = false,
}: {
	theme: ThemeStoreItem;
	large?: boolean;
}) {
	const colors = theme.preview_colors;
	return (
		<div
			className={`overflow-hidden rounded-md border border-border ${large ? "min-h-72" : "min-h-40"}`}
			style={{ background: colors.background, color: colors.foreground }}
		>
			<div className="flex h-full">
				<div
					className="w-1/4 p-3"
					style={{ background: colors.panel, color: colors.foreground }}
				>
					<div
						className="rounded px-2 py-1 text-xs"
						style={{ background: colors.primary, color: colors.background }}
					>
						Selected
					</div>
				</div>
				<div className="flex-1 space-y-3 p-3">
					<div
						className="rounded border p-3"
						style={{ background: colors.card, borderColor: colors.accent }}
					>
						<div
							className="mb-2 h-2 w-2/3 rounded"
							style={{ background: colors.foreground }}
						/>
						<div
							className="h-2 w-1/2 rounded"
							style={{ background: colors.muted }}
						/>
					</div>
					<div className="flex gap-2">
						<div
							className="h-8 flex-1 rounded"
							style={{ background: colors.primary }}
						/>
						<div
							className="h-8 flex-1 rounded"
							style={{ background: colors.accent }}
						/>
					</div>
				</div>
			</div>
		</div>
	);
}

function examsCovered(preview: PackPreview) {
	return Array.from(
		new Set(
			preview.skill_previews
				.map((skill) => skill.metadata.exam)
				.filter((exam) => exam.trim().length > 0),
		),
	);
}

function SmallList({
	label,
	items,
	empty,
}: {
	label: string;
	items: string[];
	empty: string;
}) {
	const visible = items.slice(0, 4);
	return (
		<div className="space-y-1 text-xs">
			<div className="font-medium text-foreground">{label}</div>
			<div className="flex flex-wrap gap-1">
				{visible.length > 0 ? (
					visible.map((item) => (
						<span
							key={item}
							className="rounded-md border border-border px-2 py-1 text-muted-foreground"
						>
							{item}
						</span>
					))
				) : empty ? (
					<span className="text-muted-foreground">{empty}</span>
				) : null}
				{items.length > visible.length && (
					<span className="rounded-md border border-border px-2 py-1 text-muted-foreground">
						+{items.length - visible.length} more
					</span>
				)}
			</div>
		</div>
	);
}

function Badge({ label, tone }: { label: string; tone: "primary" | "muted" }) {
	return (
		<span
			className={`inline-flex w-fit items-center rounded-full px-2 py-1 text-xs ${
				tone === "primary"
					? "bg-primary text-primary-foreground"
					: "bg-muted text-muted-foreground"
			}`}
		>
			{label}
		</span>
	);
}
