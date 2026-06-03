import { revealItemInDir } from "@tauri-apps/plugin-opener";
import {
	AlertTriangle,
	BookOpen,
	ChevronDown,
	ChevronUp,
	Compass,
	Copy,
	FileText,
	FolderOpen,
	Info,
	Languages,
	MapPin,
	Palette,
	Plus,
	Save,
	Shield,
	Sliders,
	Sparkles,
	Terminal,
	Trash2,
	User,
} from "lucide-react";
import type React from "react";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import {
	applyTheme,
	listThemeStoreItems,
	resetTheme,
	type ThemeStoreItem,
} from "@/api/marketplaceClient";
import {
	type ContextEntry,
	type Contexts,
	getContexts,
	getPreferences,
	getProfile,
	getStyle,
	type PreferenceSection,
	type Preferences,
	type Profile,
	type Style,
	saveContexts,
	savePreferences,
	saveProfile,
	saveStyle,
} from "@/api/opennivaraClient";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { useOpenNivaraTheme } from "@/theme/ThemeProvider";
import { SkillsSettingsPanel } from "./SkillsSettingsPanel";

type SettingsCategory =
	| "profile"
	| "style"
	| "preferences"
	| "contexts"
	| "skills"
	| "appearance"
	| "paths";

interface SettingsViewProps {
	paths: {
		profile: string | null;
		preferences: string | null;
		style: string | null;
		tools: string | null;
		contexts: string | null;
		telegram?: string | null;
	};
	defaultTab?: string;
}

export function SettingsView({
	paths,
	defaultTab = "profile",
}: SettingsViewProps) {
	const resolveInitialCategory = (tab: string): SettingsCategory => {
		if (tab === "paths") return "paths";
		if (
			tab === "profile" ||
			tab === "style" ||
			tab === "preferences" ||
			tab === "contexts" ||
			tab === "skills" ||
			tab === "appearance"
		) {
			return tab;
		}
		return "profile";
	};

	const [activeCategory, setActiveCategory] = useState<SettingsCategory>(
		resolveInitialCategory(defaultTab),
	);

	useEffect(() => {
		if (defaultTab) {
			setActiveCategory(resolveInitialCategory(defaultTab));
		}
	}, [defaultTab]);

	const [loading, setLoading] = useState<boolean>(true);
	const [saving, setSaving] = useState<boolean>(false);

	// Profile accordion states
	const [openSections, setOpenSections] = useState<Record<string, boolean>>({
		identity: true,
		location: true,
		languages: true,
		technical: true,
		personal: true,
		privacy: true,
	});

	const toggleSection = (key: string) => {
		setOpenSections((prev) => ({ ...prev, [key]: !prev[key] }));
	};

	// Base state storage
	const [profile, setProfile] = useState<Profile | null>(null);
	const [style, setStyle] = useState<Style | null>(null);
	const [preferences, setPreferences] = useState<Preferences | null>(null);
	const [contexts, setContexts] = useState<Contexts | null>(null);

	const [prefAccordion, setPrefAccordion] = useState<Record<string, string>>(
		{},
	);
	const [ctxAccordion, setCtxAccordion] = useState<Record<string, string>>({});

	const [installedThemes, setInstalledThemes] = useState<ThemeStoreItem[]>([]);

	const { refreshTheme } = useOpenNivaraTheme();

	// Load all configurations
	const loadAllConfigs = useCallback(async () => {
		setLoading(true);
		try {
			const [profData, styleData, prefsData, ctxsData, themesData] =
				await Promise.all([
					getProfile(),
					getStyle(),
					getPreferences(),
					getContexts(),
					listThemeStoreItems().catch(() => []),
				]);

			setProfile(profData);
			setStyle(styleData);
			setPreferences(prefsData);
			setContexts(ctxsData);
			setInstalledThemes(themesData);
		} catch (err: any) {
			toast.error(
				`Failed to load settings configuration: ${err?.message || err}`,
			);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		loadAllConfigs();
	}, [loadAllConfigs]);

	const handleSaveProfile = async (e: React.FormEvent) => {
		e.preventDefault();
		if (!profile) return;
		setSaving(true);
		try {
			await saveProfile(profile);
			toast.success("Identity profile successfully saved!");
			await loadAllConfigs();
		} catch (err: any) {
			toast.error(`Failed to save profile: ${err?.message || err}`);
		} finally {
			setSaving(false);
		}
	};

	const handleSaveStyle = async (e: React.FormEvent) => {
		e.preventDefault();
		if (!style) return;
		setSaving(true);
		try {
			await saveStyle(style);
			toast.success("Style guidelines successfully saved!");
			await loadAllConfigs();
		} catch (err: any) {
			toast.error(`Failed to save style guidelines: ${err?.message || err}`);
		} finally {
			setSaving(false);
		}
	};

	const handleSavePreferences = async () => {
		if (!preferences) return;
		setSaving(true);
		try {
			await savePreferences(preferences);
			toast.success("Base topic preferences saved!");
			await loadAllConfigs();
		} catch (err: any) {
			toast.error(`Failed to save preferences: ${err?.message || err}`);
		} finally {
			setSaving(false);
		}
	};

	const handleSaveContexts = async () => {
		if (!contexts) return;
		setSaving(true);
		try {
			await saveContexts(contexts);
			toast.success("Base project goals saved!");
			await loadAllConfigs();
		} catch (err: any) {
			toast.error(`Failed to save goal contexts: ${err?.message || err}`);
		} finally {
			setSaving(false);
		}
	};

	const handleApplyTheme = async (themeId: string | null) => {
		const loader = toast.loading("Applying theme color appearance...");
		try {
			if (themeId) {
				await applyTheme(themeId);
			} else {
				await resetTheme();
			}
			toast.success(
				themeId ? "Theme colors successfully applied!" : "Default style reset.",
				{ id: loader },
			);
			await loadAllConfigs();
			await refreshTheme();
		} catch (err: any) {
			toast.error(`Failed to apply theme: ${err?.message || err}`, {
				id: loader,
			});
		}
	};

	if (loading) {
		return (
			<div className="flex flex-col items-center justify-center h-full space-y-4 font-sans select-none">
				<div className="h-10 w-10 border-4 border-primary border-t-transparent rounded-full animate-spin"></div>
				<p className="text-xs text-muted-foreground font-extrabold uppercase tracking-wider">
					Loading OpenNivara Configuration Center...
				</p>
			</div>
		);
	}

	// Left categories sidebar navigation items list
	const categoriesList = [
		{
			id: "profile",
			label: "User Identity",
			desc: "Identity & privacy boundaries",
			icon: User,
		},
		{
			id: "style",
			label: "Response Style",
			desc: "Communication guidelines",
			icon: Palette,
		},
		{
			id: "preferences",
			label: "Topic Prefs",
			desc: "Topic keyword trigger lists",
			icon: Sliders,
		},
		{
			id: "contexts",
			label: "Project Goals",
			desc: "Dynamic landmark goal contexts",
			icon: BookOpen,
		},
		{
			id: "skills",
			label: "Skills",
			desc: "Behavior activation",
			icon: Sparkles,
		},
		{
			id: "appearance",
			label: "Appearance",
			desc: "Colors & theme selector",
			icon: Palette,
		},
		{
			id: "paths",
			label: "Config Files",
			desc: "Raw directories & file mappings",
			icon: Info,
		},
	];

	return (
		<div className="flex-1 flex overflow-hidden bg-background text-foreground h-full font-sans">
			{/* Category Sidebar Navigation (Left column) */}
			<div className="w-64 border-r border-border/40 bg-background/50 flex flex-col shrink-0 overflow-y-auto p-4 space-y-2 select-none">
				<div className="px-3.5 py-4 shrink-0">
					<span className="text-[10px] text-primary font-black uppercase tracking-widest block">
						System Config Hub
					</span>
					<h2 className="text-xs font-bold text-muted-foreground uppercase tracking-wider mt-0.5">
						Android-like Settings
					</h2>
				</div>

				<nav className="flex-1 space-y-1.5">
					{categoriesList.map((cat) => {
						const Icon = cat.icon;
						const isActive = activeCategory === cat.id;
						return (
							<button
								key={cat.id}
								onClick={() => setActiveCategory(cat.id as SettingsCategory)}
								className={`w-full flex items-start gap-3 p-3.5 rounded-2xl text-left transition-all duration-200 cursor-pointer ${
									isActive
										? "bg-secondary text-primary border border-border/40 font-black"
										: "text-muted-foreground hover:text-foreground hover:bg-secondary/40"
								}`}
							>
								<Icon
									className={`h-4.5 w-4.5 shrink-0 mt-0.5 ${isActive ? "text-primary" : "text-muted-foreground/80"}`}
								/>
								<div className="space-y-0.5 leading-none">
									<span className="text-xs font-bold block">{cat.label}</span>
									<span className="text-[9px] text-muted-foreground/75 font-semibold block uppercase tracking-wide">
										{cat.desc}
									</span>
								</div>
							</button>
						);
					})}
				</nav>
			</div>

			{/* Main Content Detail Panels (Right column - Wide layouts taking full desktop width) */}
			<div className="flex-1 flex flex-col overflow-hidden bg-background/25">
				{/* Settings view Title bar */}
				<div className="p-6 border-b border-border/40 flex items-center justify-between shrink-0 bg-background/40 select-none">
					<div>
						<h1 className="font-extrabold text-base tracking-wide uppercase font-heading text-foreground">
							{categoriesList.find((c) => c.id === activeCategory)?.label}
						</h1>
						<p className="text-[10px] text-muted-foreground font-extrabold uppercase tracking-wider">
							{categoriesList.find((c) => c.id === activeCategory)?.desc}
						</p>
					</div>
				</div>

				{/* Categories scrollable container panel */}
				<div className="flex-1 overflow-y-auto p-6 space-y-6">
					{/* USER IDENTITY TAB */}
					{activeCategory === "profile" && profile && (
						<form
							onSubmit={handleSaveProfile}
							className="space-y-5 w-full xl:max-w-6xl font-sans select-none"
						>
							{/* Profile Identity info */}
							<Card className="bg-card border border-border overflow-hidden rounded-2xl shadow">
								<button
									type="button"
									onClick={() => toggleSection("identity")}
									className="w-full flex items-center justify-between p-4 bg-secondary/20 hover:bg-secondary/30 border-b border-border select-none cursor-pointer"
								>
									<div className="flex items-center gap-3">
										<User className="h-4.5 w-4.5 text-primary shrink-0" />
										<div className="text-left">
											<span className="font-black text-xs uppercase tracking-wider text-foreground">
												Identity & Background Details
											</span>
											<span className="text-[9px] text-muted-foreground font-bold uppercase block tracking-wider mt-0.5">
												Will only be transmitted if basic identity toggle is
												turned on in privacy controls below.
											</span>
										</div>
									</div>
									{openSections.identity ? (
										<ChevronUp className="h-4 w-4 text-muted-foreground" />
									) : (
										<ChevronDown className="h-4 w-4 text-muted-foreground" />
									)}
								</button>

								{openSections.identity && (
									<div className="p-5 grid grid-cols-1 md:grid-cols-2 gap-4">
										<div className="space-y-1.5">
											<label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
												Display Name
											</label>
											<input
												type="text"
												value={profile.identity.display_name}
												onChange={(e) =>
													setProfile({
														...profile,
														identity: {
															...profile.identity,
															display_name: e.target.value,
														},
													})
												}
												className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground focus:border-primary/50 outline-none"
												placeholder="e.g. Alice"
												required
											/>
										</div>
										<div className="space-y-1.5">
											<label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
												Full Name
											</label>
											<input
												type="text"
												value={profile.identity.full_name || ""}
												onChange={(e) =>
													setProfile({
														...profile,
														identity: {
															...profile.identity,
															full_name: e.target.value,
														},
													})
												}
												className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground focus:border-primary/50 outline-none"
												placeholder="e.g. Alice Smith"
											/>
										</div>
										<div className="grid grid-cols-2 gap-3 md:col-span-2">
											<div className="space-y-1.5">
												<label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
													Pronouns
												</label>
												<input
													type="text"
													value={profile.identity.pronouns || ""}
													onChange={(e) =>
														setProfile({
															...profile,
															identity: {
																...profile.identity,
																pronouns: e.target.value,
															},
														})
													}
													className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground focus:border-primary/50 outline-none"
													placeholder="e.g. she/her"
												/>
											</div>
											<div className="space-y-1.5">
												<label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
													Gender
												</label>
												<input
													type="text"
													value={profile.identity.gender || ""}
													onChange={(e) =>
														setProfile({
															...profile,
															identity: {
																...profile.identity,
																gender: e.target.value,
															},
														})
													}
													className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground focus:border-primary/50 outline-none"
													placeholder="e.g. Female"
												/>
											</div>
										</div>
									</div>
								)}
							</Card>

							{/* Physical Location info */}
							<Card className="bg-card border border-border overflow-hidden rounded-2xl shadow">
								<button
									type="button"
									onClick={() => toggleSection("location")}
									className="w-full flex items-center justify-between p-4 bg-secondary/20 hover:bg-secondary/30 border-b border-border select-none cursor-pointer"
								>
									<div className="flex items-center gap-3">
										<MapPin className="h-4.5 w-4.5 text-primary shrink-0" />
										<div className="text-left">
											<span className="font-black text-xs uppercase tracking-wider text-foreground">
												Location Settings
											</span>
											<span className="text-[9px] text-muted-foreground font-bold uppercase block tracking-wider mt-0.5">
												Sent only when physical location toggle is enabled.
											</span>
										</div>
									</div>
									{openSections.location ? (
										<ChevronUp className="h-4 w-4 text-muted-foreground" />
									) : (
										<ChevronDown className="h-4 w-4 text-muted-foreground" />
									)}
								</button>

								{openSections.location && (
									<div className="p-5 grid grid-cols-1 md:grid-cols-2 gap-4">
										<div className="space-y-1.5">
											<label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
												City
											</label>
											<input
												type="text"
												value={profile.location.city || ""}
												onChange={(e) =>
													setProfile({
														...profile,
														location: {
															...profile.location,
															city: e.target.value,
														},
													})
												}
												className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground focus:border-primary/50 outline-none"
												placeholder="e.g. San Francisco"
											/>
										</div>
										<div className="space-y-1.5">
											<label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
												State / Region
											</label>
											<input
												type="text"
												value={profile.location.state_or_region || ""}
												onChange={(e) =>
													setProfile({
														...profile,
														location: {
															...profile.location,
															state_or_region: e.target.value,
														},
													})
												}
												className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground focus:border-primary/50 outline-none"
												placeholder="e.g. CA"
											/>
										</div>
									</div>
								)}
							</Card>

							{/* Languages */}
							<Card className="bg-card border border-border overflow-hidden rounded-2xl shadow">
								<button
									type="button"
									onClick={() => toggleSection("languages")}
									className="w-full flex items-center justify-between p-4 bg-secondary/20 hover:bg-secondary/30 border-b border-border select-none cursor-pointer"
								>
									<div className="flex items-center gap-3">
										<Languages className="h-4.5 w-4.5 text-primary shrink-0" />
										<div className="text-left">
											<span className="font-black text-xs uppercase tracking-wider text-foreground">
												Language Preferences
											</span>
										</div>
									</div>
									{openSections.languages ? (
										<ChevronUp className="h-4 w-4 text-muted-foreground" />
									) : (
										<ChevronDown className="h-4 w-4 text-muted-foreground" />
									)}
								</button>

								{openSections.languages && (
									<div className="p-5 grid grid-cols-1 md:grid-cols-2 gap-4">
										<div className="space-y-1.5">
											<label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
												Primary Language
											</label>
											<input
												type="text"
												value={profile.languages.preferred_human_language || ""}
												onChange={(e) =>
													setProfile({
														...profile,
														languages: {
															...profile.languages,
															preferred_human_language: e.target.value,
														},
													})
												}
												className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground focus:border-primary/50 outline-none"
												placeholder="English"
												required
											/>
										</div>
									</div>
								)}
							</Card>

							{/* Technical Profile Specs */}
							<Card className="bg-card border border-border overflow-hidden rounded-2xl shadow">
								<button
									type="button"
									onClick={() => toggleSection("technical")}
									className="w-full flex items-center justify-between p-4 bg-secondary/20 hover:bg-secondary/30 border-b border-border select-none cursor-pointer"
								>
									<div className="flex items-center gap-3">
										<Compass className="h-4.5 w-4.5 text-primary shrink-0" />
										<div className="text-left">
											<span className="font-black text-xs uppercase tracking-wider text-foreground">
												Technical Profile Specs
											</span>
										</div>
									</div>
									{openSections.technical ? (
										<ChevronUp className="h-4 w-4 text-muted-foreground" />
									) : (
										<ChevronDown className="h-4 w-4 text-muted-foreground" />
									)}
								</button>

								{openSections.technical && (
									<div className="p-5 grid grid-cols-1 md:grid-cols-2 gap-4">
										<div className="space-y-1.5">
											<label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
												Primary Editor
											</label>
											<input
												type="text"
												value={profile.technical.main_editor || ""}
												onChange={(e) =>
													setProfile({
														...profile,
														technical: {
															...profile.technical,
															main_editor: e.target.value,
														},
													})
												}
												className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground focus:border-primary/50 outline-none"
												placeholder="e.g. VS Code"
											/>
										</div>
										<div className="space-y-1.5">
											<label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
												Operating System
											</label>
											<input
												type="text"
												value={profile.technical.current_os || ""}
												onChange={(e) =>
													setProfile({
														...profile,
														technical: {
															...profile.technical,
															current_os: e.target.value,
														},
													})
												}
												className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground focus:border-primary/50 outline-none"
												placeholder="e.g. Windows"
											/>
										</div>
										<div className="space-y-1.5">
											<label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
												Coding Level
											</label>
											<select
												aria-label="Coding Level"
												value={profile.technical.coding_level}
												onChange={(e) =>
													setProfile({
														...profile,
														technical: {
															...profile.technical,
															coding_level: e.target.value,
														},
													})
												}
												className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground focus:border-primary/50 outline-none"
											>
												<option value="beginner">Beginner</option>
												<option value="intermediate">Intermediate</option>
												<option value="Advanced">Advanced (Expert)</option>
											</select>
										</div>
										<div className="space-y-1.5">
											<label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wide">
												Preferred Languages (Comma separated)
											</label>
											<input
												type="text"
												value={(
													profile.technical.preferred_coding_languages || []
												).join(", ")}
												onChange={(e) =>
													setProfile({
														...profile,
														technical: {
															...profile.technical,
															preferred_coding_languages: e.target.value
																.split(",")
																.map((s) => s.trim())
																.filter(Boolean),
														},
													})
												}
												className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground focus:border-primary/50 outline-none"
												placeholder="e.g. Rust, TypeScript"
											/>
										</div>
									</div>
								)}
							</Card>

							{/* Privacy and Data Boundaries */}
							<Card className="bg-card border border-border overflow-hidden rounded-2xl shadow">
								<button
									type="button"
									onClick={() => toggleSection("privacy")}
									className="w-full flex items-center justify-between p-4 bg-secondary/20 hover:bg-secondary/30 border-b border-border select-none cursor-pointer"
								>
									<div className="flex items-center gap-3">
										<Shield className="h-4.5 w-4.5 text-primary shrink-0" />
										<div className="text-left">
											<span className="font-black text-xs uppercase tracking-wider text-foreground">
												Privacy & Identity Controls
											</span>
											<span className="text-[9px] text-muted-foreground font-bold uppercase block tracking-wider mt-0.5">
												Identity details are protected and will never be shared
												without your permission.
											</span>
										</div>
									</div>
									{openSections.privacy ? (
										<ChevronUp className="h-4 w-4 text-muted-foreground" />
									) : (
										<ChevronDown className="h-4 w-4 text-muted-foreground" />
									)}
								</button>

								{openSections.privacy && (
									<div className="p-5 space-y-4">
										<div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
											{[
												{
													key: "send_identity",
													label: "Include Basic Identity Info",
												},
												{
													key: "send_location",
													label: "Include Physical Location",
												},
												{
													key: "send_gender",
													label: "Include Pronouns / Gender",
												},
												{
													key: "send_technical",
													label: "Include Technical Prefs",
												},
												{
													key: "send_personal",
													label: "Include Personal Interests",
												},
											].map((item) => {
												const val =
													profile.privacy[
														item.key as keyof typeof profile.privacy
													];
												return (
													<label
														key={item.key}
														className="flex items-start gap-3 p-3 rounded-xl border border-border bg-secondary/20 hover:bg-secondary/35 cursor-pointer select-none transition-all"
													>
														<input
															type="checkbox"
															checked={val}
															onChange={(e) =>
																setProfile({
																	...profile,
																	privacy: {
																		...profile.privacy,
																		[item.key]: e.target.checked,
																	},
																})
															}
															className="mt-0.5 text-primary rounded border-border focus:ring-primary h-4 w-4 shrink-0"
														/>
														<span className="text-xs font-bold text-foreground leading-none">
															{item.label}
														</span>
													</label>
												);
											})}
										</div>
									</div>
								)}
							</Card>

							{/* Submit */}
							<div className="flex items-center justify-end pt-3">
								<button
									type="submit"
									disabled={saving}
									className="flex items-center justify-center gap-2 px-5 py-2.5 rounded-xl bg-primary hover:bg-primary/95 text-primary-foreground font-extrabold text-[10px] uppercase tracking-wider transition-all shadow cursor-pointer"
								>
									<Save className="h-4 w-4" />
									<span>{saving ? "Saving..." : "Save Identity Profile"}</span>
								</button>
							</div>
						</form>
					)}

					{/* RESPONSE STYLE TAB */}
					{activeCategory === "style" && style && (
						<form
							onSubmit={handleSaveStyle}
							className="space-y-6 w-full xl:max-w-6xl font-sans"
						>
							{/* Communication Guidelines Card */}
							<Card className="p-5 bg-card border border-border space-y-4 rounded-2xl shadow">
								<div className="flex items-center gap-3 border-b border-border/40 pb-3">
									<Palette className="h-4.5 w-4.5 text-primary" />
									<h3 className="font-extrabold text-sm text-foreground uppercase tracking-wide">
										Communication Style Guidelines
									</h3>
								</div>
								<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
									<div className="space-y-1.5 md:col-span-2">
										<label className="text-[10px] font-bold text-muted-foreground uppercase">
											Response Tone / Persona
										</label>
										<input
											type="text"
											value={style.communication.tone}
											onChange={(e) =>
												setStyle({
													...style,
													communication: {
														...style.communication,
														tone: e.target.value,
													},
												})
											}
											className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground focus:border-primary/50 outline-none"
											placeholder="e.g. clear, direct, beginner-friendly"
											required
										/>
									</div>

									<div className="space-y-1.5">
										<label className="text-[10px] font-bold text-muted-foreground uppercase">
											Detail Level
										</label>
										<select
											aria-label="Detail Level"
											value={style.communication.detail_level}
											onChange={(e) =>
												setStyle({
													...style,
													communication: {
														...style.communication,
														detail_level: e.target.value,
													},
												})
											}
											className="w-full bg-background border border-border rounded-xl p-2 text-xs text-foreground focus:border-primary/50 outline-none bg-zinc-950"
										>
											<option value="low">Low (Brief answers)</option>
											<option value="medium">
												Medium (Standard explanations)
											</option>
											<option value="high">High (In-depth analysis)</option>
										</select>
									</div>
								</div>

								<div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2">
									{[
										{
											key: "use_examples",
											label: "Include Usage Examples",
											desc: "Always demonstrate code/instructions with practical examples",
										},
										{
											key: "use_step_by_step",
											label: "Use Step-by-Step Layout",
											desc: "Break complex operations down into numbered checklists",
										},
										{
											key: "avoid_unexplained_jargon",
											label: "Avoid Unexplained Jargon",
											desc: "Ensure technical terms are simplified or defined",
										},
										{
											key: "ask_fewer_questions",
											label: "Ask Fewer Clarifying Questions",
											desc: "Provide high-probability answers first instead of stalling",
										},
										{
											key: "prefer_actionable_answers",
											label: "Prefer Actionable Answers",
											desc: "Focus output on direct commands, code, and exact steps",
										},
									].map((item) => {
										const val = style.communication[
											item.key as keyof typeof style.communication
										] as boolean;
										return (
											<label
												key={item.key}
												className="flex items-start gap-3 p-3 rounded-xl border border-border/40 bg-secondary/15 hover:bg-secondary/35 cursor-pointer select-none transition-all"
											>
												<input
													type="checkbox"
													checked={val}
													onChange={(e) =>
														setStyle({
															...style,
															communication: {
																...style.communication,
																[item.key]: e.target.checked,
															},
														})
													}
													className="mt-0.5 text-primary rounded border-border focus:ring-primary h-4 w-4 shrink-0"
												/>
												<div className="space-y-0.5">
													<span className="text-xs font-bold text-foreground leading-none">
														{item.label}
													</span>
													<p className="text-[10px] text-muted-foreground">
														{item.desc}
													</p>
												</div>
											</label>
										);
									})}
								</div>
							</Card>

							{/* Coding Guidance */}
							<Card className="p-5 bg-card border border-border space-y-4 rounded-2xl shadow">
								<div className="flex items-center gap-3 border-b border-border/40 pb-3">
									<Sparkles className="h-4.5 w-4.5 text-primary" />
									<h3 className="font-extrabold text-sm text-foreground uppercase tracking-wide">
										Coding Output Guidance
									</h3>
								</div>
								<div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
									{[
										{
											key: "show_simple_solution_first",
											label: "Show Simple Solution First",
											desc: "Prioritize readable, minimal code before heavy optimizations",
										},
										{
											key: "explain_after_code",
											label: "Explain After Code",
											desc: "Present implementation blocks first and architectural comments below",
										},
										{
											key: "prefer_mvp_architecture",
											label: "Prefer MVP Architecture",
											desc: "Stick to clean Minimum Viable Product paradigms",
										},
										{
											key: "avoid_overengineering",
											label: "Avoid Overengineering",
											desc: "Do not add redundant factories, wrappers, or excessive layers",
										},
										{
											key: "use_beginner_comments",
											label: "Use Beginner Comments",
											desc: "Document code blocks clearly to explain non-obvious details",
										},
									].map((item) => {
										const val = style.coding[
											item.key as keyof typeof style.coding
										] as boolean;
										return (
											<label
												key={item.key}
												className="flex items-start gap-3 p-3 rounded-xl border border-border/40 bg-secondary/15 hover:bg-secondary/35 cursor-pointer select-none transition-all"
											>
												<input
													type="checkbox"
													checked={val}
													onChange={(e) =>
														setStyle({
															...style,
															coding: {
																...style.coding,
																[item.key]: e.target.checked,
															},
														})
													}
													className="mt-0.5 text-primary rounded border-border focus:ring-primary h-4 w-4 shrink-0"
												/>
												<div className="space-y-0.5">
													<span className="text-xs font-bold text-foreground leading-none">
														{item.label}
													</span>
													<p className="text-[10px] text-muted-foreground">
														{item.desc}
													</p>
												</div>
											</label>
										);
									})}
								</div>
							</Card>

							{/* Formatting & Layout */}
							<Card className="p-5 bg-card border border-border space-y-4 rounded-2xl shadow">
								<div className="flex items-center gap-3 border-b border-border/40 pb-3">
									<FileText className="h-4.5 w-4.5 text-primary" />
									<h3 className="font-extrabold text-sm text-foreground uppercase tracking-wide">
										Formatting & Layout
									</h3>
								</div>
								<div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-3">
									{[
										{
											key: "use_markdown",
											label: "Use Markdown",
											desc: "Format response using clear markdown styling",
										},
										{
											key: "use_short_sections",
											label: "Use Short Sections",
											desc: "Keep paragraphs concise and highly readable",
										},
										{
											key: "include_next_step",
											label: "Include Next Step",
											desc: "Add suggested action item at the very end",
										},
										{
											key: "avoid_long_walls_of_text",
											label: "Avoid Long Walls of Text",
											desc: "Break up massive prose blocks using formatting",
										},
									].map((item) => {
										const val = style.formatting[
											item.key as keyof typeof style.formatting
										] as boolean;
										return (
											<label
												key={item.key}
												className="flex items-start gap-3 p-3 rounded-xl border border-border/40 bg-secondary/15 hover:bg-secondary/35 cursor-pointer select-none transition-all"
											>
												<input
													type="checkbox"
													checked={val}
													onChange={(e) =>
														setStyle({
															...style,
															formatting: {
																...style.formatting,
																[item.key]: e.target.checked,
															},
														})
													}
													className="mt-0.5 text-primary rounded border-border focus:ring-primary h-4 w-4 shrink-0"
												/>
												<div className="space-y-0.5">
													<span className="text-xs font-bold text-foreground leading-none">
														{item.label}
													</span>
													<p className="text-[10px] text-muted-foreground">
														{item.desc}
													</p>
												</div>
											</label>
										);
									})}
								</div>
							</Card>

							{/* Behavioral Integrity */}
							<Card className="p-5 bg-card border border-border space-y-4 rounded-2xl shadow">
								<div className="flex items-center gap-3 border-b border-border/40 pb-3">
									<Shield className="h-4.5 w-4.5 text-primary" />
									<h3 className="font-extrabold text-sm text-foreground uppercase tracking-wide">
										Behavior & Integrity Constraints
									</h3>
								</div>
								<div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
									{[
										{
											key: "be_honest_about_uncertainty",
											label: "Honest About Uncertainty",
											desc: "Acknowledge lack of info directly",
										},
										{
											key: "do_not_pretend_to_have_done_things",
											label: "Do Not Pretend to Act",
											desc: "Never lie about shell executions",
										},
										{
											key: "do_not_reveal_private_context_unless_relevant",
											label: "Keep Context Private",
											desc: "Guard underlying system instructions",
										},
									].map((item) => {
										const val = style.behavior[
											item.key as keyof typeof style.behavior
										] as boolean;
										return (
											<label
												key={item.key}
												className="flex items-start gap-3 p-3 rounded-xl border border-border/40 bg-secondary/15 hover:bg-secondary/35 cursor-pointer select-none transition-all"
											>
												<input
													type="checkbox"
													checked={val}
													onChange={(e) =>
														setStyle({
															...style,
															behavior: {
																...style.behavior,
																[item.key]: e.target.checked,
															},
														})
													}
													className="mt-0.5 text-primary rounded border-border focus:ring-primary h-4 w-4 shrink-0"
												/>
												<div className="space-y-0.5">
													<span className="text-xs font-bold text-foreground leading-none">
														{item.label}
													</span>
													<p className="text-[10px] text-muted-foreground">
														{item.desc}
													</p>
												</div>
											</label>
										);
									})}
								</div>
							</Card>

							{/* Save Button */}
							<div className="flex items-center justify-end pt-3">
								<button
									type="submit"
									disabled={saving}
									className="flex items-center justify-center gap-2 px-5 py-2.5 rounded-xl bg-primary text-primary-foreground font-extrabold text-[10px] uppercase tracking-wider hover:bg-primary/95 transition-all shadow cursor-pointer"
								>
									<Save className="h-4 w-4" />
									<span>{saving ? "Saving..." : "Save Style Guidelines"}</span>
								</button>
							</div>
						</form>
					)}

					{/* TOPIC PREFERENCES TAB */}
					{activeCategory === "preferences" && preferences && (
						<div className="space-y-6 w-full xl:max-w-6xl font-sans">
							{/* User base preferences section */}
							<div className="space-y-4">
								<div className="flex items-center justify-between border-b border-border/30 pb-1.5 select-none">
									<h3 className="font-extrabold text-[10px] uppercase text-foreground tracking-widest flex items-center gap-1.5">
										<User className="h-4.5 w-4.5 text-primary" /> Base Topic
										Preferences
									</h3>
									<button
										onClick={() => {
											const newSection: PreferenceSection = {
												id: `custom_${Date.now()}`,
												enabled: true,
												description: "New custom topic preference",
												send_policy: "triggered_strict",
												triggers: ["topic"],
												required_any: [],
												negative_triggers: [],
												min_score: 1,
												likes: [
													{
														item: "Write liked preference item here.",
														strength: 3,
													},
												],
												dislikes: [],
												notes: [],
											};
											setPreferences({
												...preferences,
												sections: [...preferences.sections, newSection],
											});
											toast.success(
												"Added new card. Don't forget to click Save!",
											);
										}}
										className="flex items-center justify-center gap-1 px-2.5 py-1 rounded-lg border border-primary/20 bg-primary/5 hover:bg-primary/10 text-primary font-bold text-[10px] tracking-wide uppercase transition-all cursor-pointer"
									>
										<Plus className="h-3.5 w-3.5" />
										<span>Add Preference Section</span>
									</button>
								</div>

								{preferences.sections.length === 0 ? (
									<div className="text-center py-10 bg-secondary/15 rounded-2xl border border-dashed border-border select-none">
										<AlertTriangle className="h-8 w-8 text-amber-500 mx-auto mb-2" />
										<p className="text-xs font-bold text-foreground">
											No base topic preferences defined.
										</p>
									</div>
								) : (
									preferences.sections.map((section, sIdx) => {
										const activeSub = prefAccordion[section.id] || "basic";
										const setActiveSub = (tab: string) => {
											setPrefAccordion((prev) => ({
												...prev,
												[section.id]: tab,
											}));
										};
										return (
											<Card
												key={section.id}
												className="p-5 bg-card border border-border space-y-4 rounded-2xl shadow relative group"
											>
												<div className="flex items-start justify-between gap-4 border-b border-border/40 pb-3">
													<div className="space-y-1.5 flex-1">
														<div className="flex items-center gap-2">
															<span className="text-[9px] bg-primary/10 border border-primary/20 text-primary font-black px-2 py-0.5 rounded font-mono uppercase">
																ID: {section.id}
															</span>
														</div>
														<input
															type="text"
															value={section.description || ""}
															onChange={(e) => {
																const updated = [...preferences.sections];
																updated[sIdx].description = e.target.value;
																setPreferences({
																	...preferences,
																	sections: updated,
																});
															}}
															className="bg-transparent border-b border-transparent focus:border-primary/50 text-xs font-bold text-foreground outline-none w-full py-0.5"
															placeholder="Description of this topic preference"
														/>
													</div>
													<div className="flex items-center gap-3 shrink-0">
														<label className="flex items-center gap-1.5 text-[10px] font-bold text-muted-foreground uppercase cursor-pointer select-none">
															<input
																type="checkbox"
																checked={section.enabled}
																onChange={(e) => {
																	const updated = [...preferences.sections];
																	updated[sIdx].enabled = e.target.checked;
																	setPreferences({
																		...preferences,
																		sections: updated,
																	});
																}}
																className="text-primary rounded border-border h-3.5 w-3.5"
															/>
															<span>Active</span>
														</label>
														<button
															type="button"
															onClick={() => {
																const updated = preferences.sections.filter(
																	(_, idx) => idx !== sIdx,
																);
																setPreferences({
																	...preferences,
																	sections: updated,
																});
																toast.error(
																	"Omitted section card. Remember to click Save!",
																);
															}}
															className="h-7 w-7 rounded-lg hover:bg-rose-500/10 text-rose-400 flex items-center justify-center shrink-0 transition-colors cursor-pointer"
															title="Delete Section"
														>
															<Trash2 className="h-3.5 w-3.5" />
														</button>
													</div>
												</div>

												{/* Sub-Accordions Navigation */}
												<div className="flex border-b border-border/15 pb-1 gap-1 shrink-0">
													{[
														{ id: "basic", label: "Basic" },
														{ id: "triggers", label: "Trigger Rules" },
														{ id: "likes", label: "Likes / Strengths" },
														{ id: "dislikes", label: "Dislikes / Strengths" },
														{ id: "notes", label: "Notes" },
													].map((tab) => {
														const isActive = activeSub === tab.id;
														return (
															<button
																key={tab.id}
																type="button"
																onClick={() => setActiveSub(tab.id)}
																className={`px-2.5 py-1 rounded text-[10px] font-bold tracking-wide transition-all cursor-pointer ${
																	isActive
																		? "bg-primary/10 text-primary"
																		: "text-muted-foreground hover:text-foreground"
																}`}
															>
																{tab.label}
															</button>
														);
													})}
												</div>

												{/* Tab Content Panels */}
												{activeSub === "basic" && (
													<div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
														<div className="space-y-1">
															<label className="text-[10px] font-bold text-muted-foreground uppercase">
																Send Policy
															</label>
															<select
																aria-label="Preference Send Policy"
																value={section.send_policy}
																onChange={(e) => {
																	const updated = [...preferences.sections];
																	updated[sIdx].send_policy = e.target.value;
																	setPreferences({
																		...preferences,
																		sections: updated,
																	});
																}}
																className="w-full bg-background border border-border rounded-xl p-2 text-xs outline-none bg-zinc-950 text-foreground"
															>
																<option value="always">Always Sent</option>
																<option value="manual">Manual</option>
																<option value="session_pinned">
																	Session Pinned
																</option>
																<option value="triggered_strict">
																	Triggered Strict
																</option>
																<option value="never">Never Sent</option>
																<option value="disabled">Disabled</option>
															</select>
														</div>
													</div>
												)}

												{activeSub === "triggers" && (
													<div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
														<div className="space-y-1">
															<label className="text-[10px] font-bold text-muted-foreground uppercase">
																Keyword Triggers (comma-separated)
															</label>
															<input
																type="text"
																value={section.triggers.join(", ")}
																onChange={(e) => {
																	const updated = [...preferences.sections];
																	updated[sIdx].triggers = e.target.value
																		.split(",")
																		.map((s) => s.trim())
																		.filter(Boolean);
																	setPreferences({
																		...preferences,
																		sections: updated,
																	});
																}}
																className="w-full bg-background border border-border rounded-xl p-2 text-xs font-mono outline-none text-foreground"
																placeholder="e.g. rust, typescript"
															/>
														</div>
														<div className="space-y-1">
															<label className="text-[10px] font-bold text-muted-foreground uppercase">
																Required Any (comma-separated)
															</label>
															<input
																type="text"
																value={(section.required_any || []).join(", ")}
																onChange={(e) => {
																	const updated = [...preferences.sections];
																	updated[sIdx].required_any = e.target.value
																		.split(",")
																		.map((s) => s.trim())
																		.filter(Boolean);
																	setPreferences({
																		...preferences,
																		sections: updated,
																	});
																}}
																className="w-full bg-background border border-border rounded-xl p-2 text-xs font-mono outline-none text-foreground"
																placeholder="e.g. build, error"
															/>
														</div>
														<div className="space-y-1">
															<label className="text-[10px] font-bold text-muted-foreground uppercase">
																Negative Triggers (comma-separated)
															</label>
															<input
																type="text"
																value={(section.negative_triggers || []).join(
																	", ",
																)}
																onChange={(e) => {
																	const updated = [...preferences.sections];
																	updated[sIdx].negative_triggers =
																		e.target.value
																			.split(",")
																			.map((s) => s.trim())
																			.filter(Boolean);
																	setPreferences({
																		...preferences,
																		sections: updated,
																	});
																}}
																className="w-full bg-background border border-border rounded-xl p-2 text-xs font-mono outline-none text-foreground"
																placeholder="e.g. web, UI"
															/>
														</div>
														<div className="space-y-1">
															<label className="text-[10px] font-bold text-muted-foreground uppercase">
																Minimum Trigger Score
															</label>
															<input
																type="number"
																value={section.min_score}
																min={0}
																onChange={(e) => {
																	const updated = [...preferences.sections];
																	updated[sIdx].min_score =
																		parseInt(e.target.value, 10) || 0;
																	setPreferences({
																		...preferences,
																		sections: updated,
																	});
																}}
																className="w-full bg-background border border-border rounded-xl p-2 text-xs outline-none text-foreground"
															/>
														</div>
													</div>
												)}

												{activeSub === "likes" && (
													<div className="space-y-2">
														<div className="flex items-center justify-between">
															<label className="text-[9px] font-extrabold text-emerald-400 uppercase">
																Preferences Bullet Guidelines (Likes)
															</label>
															<button
																type="button"
																onClick={() => {
																	const updated = [...preferences.sections];
																	updated[sIdx].likes.push({
																		item: "New liked guideline item",
																		strength: 3,
																	});
																	setPreferences({
																		...preferences,
																		sections: updated,
																	});
																}}
																className="text-[9px] text-primary hover:underline flex items-center gap-1 font-bold uppercase cursor-pointer"
															>
																<Plus className="h-3 w-3" />
																<span>Add Bullet</span>
															</button>
														</div>
														<div className="space-y-2">
															{(section.likes || []).map((like, lIdx) => (
																<div
																	key={lIdx}
																	className="flex gap-2 items-center"
																>
																	<span className="text-emerald-400 font-mono text-xs">
																		•
																	</span>
																	<input
																		type="text"
																		value={like.item}
																		onChange={(e) => {
																			const updated = [...preferences.sections];
																			updated[sIdx].likes[lIdx].item =
																				e.target.value;
																			setPreferences({
																				...preferences,
																				sections: updated,
																			});
																		}}
																		className="bg-background border border-border rounded-xl p-2 text-xs flex-1 outline-none font-medium text-foreground animate-none"
																	/>
																	<div className="flex items-center gap-1.5 shrink-0">
																		<span className="text-[9px] text-muted-foreground uppercase font-bold">
																			Strength
																		</span>
																		<select
																			aria-label="Like Strength"
																			value={like.strength}
																			onChange={(e) => {
																				const updated = [
																					...preferences.sections,
																				];
																				updated[sIdx].likes[lIdx].strength =
																					parseInt(e.target.value, 10) || 3;
																				setPreferences({
																					...preferences,
																					sections: updated,
																				});
																			}}
																			className="bg-background border border-border rounded-xl p-1.5 text-xs outline-none text-foreground bg-zinc-950"
																		>
																			{[1, 2, 3, 4, 5].map((v) => (
																				<option key={v} value={v}>
																					{v}
																				</option>
																			))}
																		</select>
																	</div>
																	<button
																		type="button"
																		onClick={() => {
																			const updated = [...preferences.sections];
																			updated[sIdx].likes.splice(lIdx, 1);
																			setPreferences({
																				...preferences,
																				sections: updated,
																			});
																		}}
																		className="h-6.5 w-6.5 rounded hover:bg-rose-500/10 text-rose-400 flex items-center justify-center shrink-0 transition-colors cursor-pointer"
																	>
																		<Trash2 className="h-3.5 w-3.5" />
																	</button>
																</div>
															))}
														</div>
													</div>
												)}

												{activeSub === "dislikes" && (
													<div className="space-y-2">
														<div className="flex items-center justify-between">
															<label className="text-[9px] font-extrabold text-rose-400 uppercase">
																Preferences Bullet Guidelines (Dislikes)
															</label>
															<button
																type="button"
																onClick={() => {
																	const updated = [...preferences.sections];
																	updated[sIdx].dislikes =
																		updated[sIdx].dislikes || [];
																	updated[sIdx].dislikes.push({
																		item: "New disliked guideline item",
																		strength: 3,
																	});
																	setPreferences({
																		...preferences,
																		sections: updated,
																	});
																}}
																className="text-[9px] text-primary hover:underline flex items-center gap-1 font-bold uppercase cursor-pointer"
															>
																<Plus className="h-3 w-3" />
																<span>Add Bullet</span>
															</button>
														</div>
														<div className="space-y-2">
															{(section.dislikes || []).map((dislike, dIdx) => (
																<div
																	key={dIdx}
																	className="flex gap-2 items-center"
																>
																	<span className="text-rose-400 font-mono text-xs">
																		•
																	</span>
																	<input
																		type="text"
																		value={dislike.item}
																		onChange={(e) => {
																			const updated = [...preferences.sections];
																			updated[sIdx].dislikes[dIdx].item =
																				e.target.value;
																			setPreferences({
																				...preferences,
																				sections: updated,
																			});
																		}}
																		className="bg-background border border-border rounded-xl p-2 text-xs flex-1 outline-none font-medium text-foreground animate-none"
																	/>
																	<div className="flex items-center gap-1.5 shrink-0">
																		<span className="text-[9px] text-muted-foreground uppercase font-bold">
																			Strength
																		</span>
																		<select
																			aria-label="Dislike Strength"
																			value={dislike.strength}
																			onChange={(e) => {
																				const updated = [
																					...preferences.sections,
																				];
																				updated[sIdx].dislikes[dIdx].strength =
																					parseInt(e.target.value, 10) || 3;
																				setPreferences({
																					...preferences,
																					sections: updated,
																				});
																			}}
																			className="bg-background border border-border rounded-xl p-1.5 text-xs outline-none text-foreground bg-zinc-950"
																		>
																			{[1, 2, 3, 4, 5].map((v) => (
																				<option key={v} value={v}>
																					{v}
																				</option>
																			))}
																		</select>
																	</div>
																	<button
																		type="button"
																		onClick={() => {
																			const updated = [...preferences.sections];
																			updated[sIdx].dislikes.splice(dIdx, 1);
																			setPreferences({
																				...preferences,
																				sections: updated,
																			});
																		}}
																		className="h-6.5 w-6.5 rounded hover:bg-rose-500/10 text-rose-400 flex items-center justify-center shrink-0 transition-colors cursor-pointer"
																	>
																		<Trash2 className="h-3.5 w-3.5" />
																	</button>
																</div>
															))}
														</div>
													</div>
												)}

												{activeSub === "notes" && (
													<div className="space-y-2">
														<div className="flex items-center justify-between">
															<label className="text-[9px] font-extrabold text-blue-400 uppercase">
																Guidance Notes
															</label>
															<button
																type="button"
																onClick={() => {
																	const updated = [...preferences.sections];
																	updated[sIdx].notes =
																		updated[sIdx].notes || [];
																	updated[sIdx].notes.push(
																		"New guidance note text",
																	);
																	setPreferences({
																		...preferences,
																		sections: updated,
																	});
																}}
																className="text-[9px] text-primary hover:underline flex items-center gap-1 font-bold uppercase cursor-pointer"
															>
																<Plus className="h-3 w-3" />
																<span>Add Note</span>
															</button>
														</div>
														<div className="space-y-2">
															{(section.notes || []).map((note, nIdx) => (
																<div
																	key={nIdx}
																	className="flex gap-2 items-center"
																>
																	<span className="text-blue-400 font-mono text-xs">
																		•
																	</span>
																	<input
																		type="text"
																		value={note}
																		onChange={(e) => {
																			const updated = [...preferences.sections];
																			updated[sIdx].notes[nIdx] =
																				e.target.value;
																			setPreferences({
																				...preferences,
																				sections: updated,
																			});
																		}}
																		className="bg-background border border-border rounded-xl p-2 text-xs flex-1 outline-none font-medium text-foreground animate-none"
																	/>
																	<button
																		type="button"
																		onClick={() => {
																			const updated = [...preferences.sections];
																			updated[sIdx].notes.splice(nIdx, 1);
																			setPreferences({
																				...preferences,
																				sections: updated,
																			});
																		}}
																		className="h-6.5 w-6.5 rounded hover:bg-rose-500/10 text-rose-400 flex items-center justify-center shrink-0 transition-colors cursor-pointer"
																	>
																		<Trash2 className="h-3.5 w-3.5" />
																	</button>
																</div>
															))}
														</div>
													</div>
												)}
											</Card>
										);
									})
								)}

								{/* Save preferences */}
								<div className="flex items-center justify-end pt-3">
									<button
										onClick={handleSavePreferences}
										disabled={saving}
										className="flex items-center justify-center gap-2 px-5 py-2.5 rounded-xl bg-primary text-primary-foreground font-extrabold text-[10px] uppercase tracking-wider hover:bg-primary/95 transition-all shadow cursor-pointer"
									>
										<Save className="h-4 w-4" />
										<span>{saving ? "Saving..." : "Save Preferences"}</span>
									</button>
								</div>
							</div>
						</div>
					)}

					{/* PROJECT GOALS TAB */}
					{activeCategory === "contexts" && contexts && (
						<div className="space-y-6 w-full xl:max-w-6xl font-sans">
							{/* Base contexts list */}
							<div className="space-y-4">
								<div className="flex items-center justify-between border-b border-border/30 pb-1.5 select-none">
									<h3 className="font-extrabold text-[10px] uppercase text-foreground tracking-widest flex items-center gap-1.5">
										<User className="h-4.5 w-4.5 text-primary" /> Base Project
										Goals
									</h3>
									<button
										onClick={() => {
											const newEntry: ContextEntry = {
												id: `custom_context_${Date.now()}`,
												enabled: true,
												kind: "project",
												send_policy: "triggered_strict",
												title: "New Goal Context Details",
												summary: "High-level summary of this specific goal.",
												triggers: ["work"],
												required_any: [],
												negative_triggers: [],
												min_score: 1,
												facts: ["Detail some workspace project facts."],
												rules: [],
											};
											setContexts({
												...contexts,
												contexts: [...contexts.contexts, newEntry],
											});
											toast.success(
												"Added new context. Don't forget to click Save!",
											);
										}}
										className="flex items-center justify-center gap-1 px-2.5 py-1 rounded-lg border border-primary/20 bg-primary/5 hover:bg-primary/10 text-primary font-bold text-[10px] tracking-wide uppercase transition-all cursor-pointer"
									>
										<Plus className="h-3.5 w-3.5" />
										<span>Add Goal Context</span>
									</button>
								</div>

								{contexts.contexts.length === 0 ? (
									<div className="text-center py-10 bg-secondary/15 rounded-2xl border border-dashed border-border select-none">
										<AlertTriangle className="h-8 w-8 text-amber-500 mx-auto mb-2" />
										<p className="text-xs font-bold text-foreground">
											No base contexts defined.
										</p>
									</div>
								) : (
									contexts.contexts.map((entry, cIdx) => {
										const activeSub = ctxAccordion[entry.id] || "basic";
										const setActiveSub = (tab: string) => {
											setCtxAccordion((prev) => ({ ...prev, [entry.id]: tab }));
										};
										return (
											<Card
												key={entry.id}
												className="p-5 bg-card border border-border space-y-4 rounded-2xl shadow relative group"
											>
												<div className="flex items-start justify-between gap-4 border-b border-border/40 pb-3">
													<div className="space-y-1.5 flex-1">
														<div className="flex items-center gap-2">
															<span className="text-[9px] bg-primary/10 border border-primary/20 text-primary font-black px-2 py-0.5 rounded font-mono uppercase">
																ID: {entry.id}
															</span>
														</div>
														<input
															type="text"
															value={entry.title || ""}
															onChange={(e) => {
																const updated = [...contexts.contexts];
																updated[cIdx].title = e.target.value;
																setContexts({ ...contexts, contexts: updated });
															}}
															className="bg-transparent border-b border-transparent focus:border-primary/50 text-xs font-bold text-foreground outline-none w-full py-0.5"
															placeholder="Context Title"
														/>
													</div>
													<div className="flex items-center gap-3 shrink-0">
														<label className="flex items-center gap-1.5 text-[10px] font-bold text-muted-foreground uppercase cursor-pointer select-none">
															<input
																type="checkbox"
																checked={entry.enabled}
																onChange={(e) => {
																	const updated = [...contexts.contexts];
																	updated[cIdx].enabled = e.target.checked;
																	setContexts({
																		...contexts,
																		contexts: updated,
																	});
																}}
																className="text-primary rounded border-border h-3.5 w-3.5"
															/>
															<span>Active</span>
														</label>
														<button
															type="button"
															onClick={() => {
																const updated = contexts.contexts.filter(
																	(_, idx) => idx !== cIdx,
																);
																setContexts({ ...contexts, contexts: updated });
																toast.error(
																	"Omitted context card. Remember to click Save!",
																);
															}}
															className="h-7 w-7 rounded-lg hover:bg-rose-500/10 text-rose-400 flex items-center justify-center shrink-0 transition-colors cursor-pointer"
															title="Delete Context"
														>
															<Trash2 className="h-3.5 w-3.5" />
														</button>
													</div>
												</div>

												{/* Sub-Accordions Navigation */}
												<div className="flex border-b border-border/15 pb-1 gap-1 shrink-0">
													{[
														{ id: "basic", label: "Basic" },
														{ id: "triggers", label: "Trigger Rules" },
														{ id: "facts", label: "Facts List" },
														{ id: "rules", label: "Rules List" },
													].map((tab) => {
														const isActive = activeSub === tab.id;
														return (
															<button
																key={tab.id}
																type="button"
																onClick={() => setActiveSub(tab.id)}
																className={`px-2.5 py-1 rounded text-[10px] font-bold tracking-wide transition-all cursor-pointer ${
																	isActive
																		? "bg-primary/10 text-primary"
																		: "text-muted-foreground hover:text-foreground"
																}`}
															>
																{tab.label}
															</button>
														);
													})}
												</div>

												{/* Tab Content Panels */}
												{activeSub === "basic" && (
													<div className="space-y-3 text-xs">
														<div className="space-y-1">
															<label className="text-[10px] font-bold text-muted-foreground uppercase">
																Summary descriptor
															</label>
															<input
																type="text"
																value={entry.summary}
																onChange={(e) => {
																	const updated = [...contexts.contexts];
																	updated[cIdx].summary = e.target.value;
																	setContexts({
																		...contexts,
																		contexts: updated,
																	});
																}}
																className="w-full bg-background border border-border rounded-xl p-2.5 text-xs text-foreground outline-none focus:border-primary/50"
															/>
														</div>
														<div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
															<div className="space-y-1">
																<label className="text-[10px] font-bold text-muted-foreground uppercase">
																	Context Kind
																</label>
																<select
																	aria-label="Context Kind"
																	value={entry.kind}
																	onChange={(e) => {
																		const updated = [...contexts.contexts];
																		updated[cIdx].kind = e.target.value;
																		setContexts({
																			...contexts,
																			contexts: updated,
																		});
																	}}
																	className="w-full bg-background border border-border rounded-xl p-2 text-xs outline-none bg-zinc-950 text-foreground"
																>
																	<option value="project">Project</option>
																	<option value="goal">Goal</option>
																	<option value="learning">Learning</option>
																	<option value="personal">Personal</option>
																	<option value="work">Work</option>
																	<option value="custom">Custom</option>
																</select>
															</div>
															<div className="space-y-1">
																<label className="text-[10px] font-bold text-muted-foreground uppercase">
																	Send Policy
																</label>
																<select
																	aria-label="Context Send Policy"
																	value={entry.send_policy}
																	onChange={(e) => {
																		const updated = [...contexts.contexts];
																		updated[cIdx].send_policy = e.target.value;
																		setContexts({
																			...contexts,
																			contexts: updated,
																		});
																	}}
																	className="w-full bg-background border border-border rounded-xl p-2 text-xs outline-none bg-zinc-950 text-foreground"
																>
																	<option value="always">Always Sent</option>
																	<option value="manual">Manual</option>
																	<option value="session_pinned">
																		Session Pinned
																	</option>
																	<option value="triggered_strict">
																		Triggered Strict
																	</option>
																	<option value="never">Never Sent</option>
																	<option value="disabled">Disabled</option>
																</select>
															</div>
														</div>
													</div>
												)}

												{activeSub === "triggers" && (
													<div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
														<div className="space-y-1">
															<label className="text-[10px] font-bold text-muted-foreground uppercase">
																Keyword Triggers (comma-separated)
															</label>
															<input
																type="text"
																value={entry.triggers.join(", ")}
																onChange={(e) => {
																	const updated = [...contexts.contexts];
																	updated[cIdx].triggers = e.target.value
																		.split(",")
																		.map((s) => s.trim())
																		.filter(Boolean);
																	setContexts({
																		...contexts,
																		contexts: updated,
																	});
																}}
																className="w-full bg-background border border-border rounded-xl p-2 text-xs font-mono outline-none text-foreground"
																placeholder="e.g. cli, compiler"
															/>
														</div>
														<div className="space-y-1">
															<label className="text-[10px] font-bold text-muted-foreground uppercase">
																Required Any (comma-separated)
															</label>
															<input
																type="text"
																value={(entry.required_any || []).join(", ")}
																onChange={(e) => {
																	const updated = [...contexts.contexts];
																	updated[cIdx].required_any = e.target.value
																		.split(",")
																		.map((s) => s.trim())
																		.filter(Boolean);
																	setContexts({
																		...contexts,
																		contexts: updated,
																	});
																}}
																className="w-full bg-background border border-border rounded-xl p-2 text-xs font-mono outline-none text-foreground"
																placeholder="e.g. rust, test"
															/>
														</div>
														<div className="space-y-1">
															<label className="text-[10px] font-bold text-muted-foreground uppercase">
																Negative Triggers (comma-separated)
															</label>
															<input
																type="text"
																value={(entry.negative_triggers || []).join(
																	", ",
																)}
																onChange={(e) => {
																	const updated = [...contexts.contexts];
																	updated[cIdx].negative_triggers =
																		e.target.value
																			.split(",")
																			.map((s) => s.trim())
																			.filter(Boolean);
																	setContexts({
																		...contexts,
																		contexts: updated,
																	});
																}}
																className="w-full bg-background border border-border rounded-xl p-2 text-xs font-mono outline-none text-foreground"
																placeholder="e.g. movie, food"
															/>
														</div>
														<div className="space-y-1">
															<label className="text-[10px] font-bold text-muted-foreground uppercase">
																Minimum Trigger Score
															</label>
															<input
																type="number"
																value={entry.min_score}
																min={0}
																onChange={(e) => {
																	const updated = [...contexts.contexts];
																	updated[cIdx].min_score =
																		parseInt(e.target.value, 10) || 0;
																	setContexts({
																		...contexts,
																		contexts: updated,
																	});
																}}
																className="w-full bg-background border border-border rounded-xl p-2 text-xs outline-none text-foreground"
															/>
														</div>
													</div>
												)}

												{activeSub === "facts" && (
													<div className="space-y-2">
														<div className="flex items-center justify-between">
															<label className="text-[9px] font-extrabold text-emerald-400 uppercase">
																Workspace Context Facts
															</label>
															<button
																type="button"
																onClick={() => {
																	const updated = [...contexts.contexts];
																	updated[cIdx].facts =
																		updated[cIdx].facts || [];
																	updated[cIdx].facts.push(
																		"New workspace project fact statement",
																	);
																	setContexts({
																		...contexts,
																		contexts: updated,
																	});
																}}
																className="text-[9px] text-primary hover:underline flex items-center gap-1 font-bold uppercase cursor-pointer"
															>
																<Plus className="h-3 w-3" />
																<span>Add Fact</span>
															</button>
														</div>
														<div className="space-y-2">
															{(entry.facts || []).map((fact, fIdx) => (
																<div
																	key={fIdx}
																	className="flex gap-2 items-center"
																>
																	<span className="text-emerald-400 font-mono text-xs">
																		•
																	</span>
																	<input
																		type="text"
																		value={fact}
																		onChange={(e) => {
																			const updated = [...contexts.contexts];
																			updated[cIdx].facts[fIdx] =
																				e.target.value;
																			setContexts({
																				...contexts,
																				contexts: updated,
																			});
																		}}
																		className="bg-background border border-border rounded-xl p-2 text-xs flex-1 outline-none font-medium text-foreground animate-none"
																	/>
																	<button
																		type="button"
																		onClick={() => {
																			const updated = [...contexts.contexts];
																			updated[cIdx].facts.splice(fIdx, 1);
																			setContexts({
																				...contexts,
																				contexts: updated,
																			});
																		}}
																		className="h-6.5 w-6.5 rounded hover:bg-rose-500/10 text-rose-400 flex items-center justify-center shrink-0 transition-colors cursor-pointer"
																	>
																		<Trash2 className="h-3.5 w-3.5" />
																	</button>
																</div>
															))}
														</div>
													</div>
												)}

												{activeSub === "rules" && (
													<div className="space-y-2">
														<div className="flex items-center justify-between">
															<label className="text-[9px] font-extrabold text-blue-400 uppercase">
																Context Rule Constraints
															</label>
															<button
																type="button"
																onClick={() => {
																	const updated = [...contexts.contexts];
																	updated[cIdx].rules =
																		updated[cIdx].rules || [];
																	updated[cIdx].rules.push(
																		"New prompt guidance constraint rule",
																	);
																	setContexts({
																		...contexts,
																		contexts: updated,
																	});
																}}
																className="text-[9px] text-primary hover:underline flex items-center gap-1 font-bold uppercase cursor-pointer"
															>
																<Plus className="h-3 w-3" />
																<span>Add Rule</span>
															</button>
														</div>
														<div className="space-y-2">
															{(entry.rules || []).map((rule, rIdx) => (
																<div
																	key={rIdx}
																	className="flex gap-2 items-center"
																>
																	<span className="text-blue-400 font-mono text-xs">
																		•
																	</span>
																	<input
																		type="text"
																		value={rule}
																		onChange={(e) => {
																			const updated = [...contexts.contexts];
																			updated[cIdx].rules[rIdx] =
																				e.target.value;
																			setContexts({
																				...contexts,
																				contexts: updated,
																			});
																		}}
																		className="bg-background border border-border rounded-xl p-2 text-xs flex-1 outline-none font-medium text-foreground animate-none"
																	/>
																	<button
																		type="button"
																		onClick={() => {
																			const updated = [...contexts.contexts];
																			updated[cIdx].rules.splice(rIdx, 1);
																			setContexts({
																				...contexts,
																				contexts: updated,
																			});
																		}}
																		className="h-6.5 w-6.5 rounded hover:bg-rose-500/10 text-rose-400 flex items-center justify-center shrink-0 transition-colors cursor-pointer"
																	>
																		<Trash2 className="h-3.5 w-3.5" />
																	</button>
																</div>
															))}
														</div>
													</div>
												)}
											</Card>
										);
									})
								)}

								{/* Save contexts */}
								<div className="flex items-center justify-end pt-3">
									<button
										onClick={handleSaveContexts}
										disabled={saving}
										className="flex items-center justify-center gap-2 px-5 py-2.5 rounded-xl bg-primary text-primary-foreground font-extrabold text-[10px] uppercase tracking-wider hover:bg-primary/95 transition-all shadow cursor-pointer"
									>
										<Save className="h-4 w-4" />
										<span>{saving ? "Saving..." : "Save Goal Contexts"}</span>
									</button>
								</div>
							</div>
						</div>
					)}

					{/* SKILLS TAB */}
					{activeCategory === "skills" && <SkillsSettingsPanel />}

					{/* THEMES & APPEARANCE TAB */}
					{activeCategory === "appearance" && (
						<div className="space-y-6 w-full xl:max-w-6xl font-sans">
							<Card className="p-5 bg-card border border-border rounded-2xl shadow flex items-center justify-between">
								<div className="flex items-center gap-3">
									<div className="h-9 w-9 rounded-xl bg-primary/10 border border-primary/20 flex items-center justify-center">
										<Palette className="h-5 w-5 text-primary" />
									</div>
									<div>
										<h3 className="font-extrabold text-sm text-foreground uppercase tracking-wide">
											Current Theme
										</h3>
										<p className="text-xs text-muted-foreground font-semibold">
											Currently using theme:{" "}
											<strong className="text-foreground">
												{installedThemes.find((theme) => theme.applied)?.name ||
													"Default Obsidian"}
											</strong>
										</p>
									</div>
								</div>

								<button
									onClick={() => handleApplyTheme(null)}
									className="px-3.5 py-1.5 rounded-xl bg-secondary hover:bg-secondary/80 text-foreground font-bold text-xs cursor-pointer text-center"
								>
									Reset to Default
								</button>
							</Card>

							<div className="space-y-4">
								<h3 className="text-[10px] font-extrabold text-muted-foreground uppercase tracking-widest border-b border-border/30 pb-1.5 select-none">
									Installed Themes
								</h3>

								<div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
									{installedThemes
										.filter((theme) => theme.installed)
										.map((theme) => {
											const isThemeActive = theme.applied;
											return (
												<Card
													key={theme.id}
													className={`p-4.5 border rounded-2xl flex flex-col justify-between transition-all duration-300 shadow select-none ${
														isThemeActive
															? "bg-card border-primary/40 ring-1 ring-primary/45"
															: "bg-card hover:bg-card/90 border-border"
													}`}
												>
													<div className="space-y-2">
														<div className="flex justify-between items-center">
															<span className="text-[9px] font-extrabold uppercase px-2 py-0.5 rounded bg-primary/10 text-primary border border-primary/20 font-mono">
																{theme.source_kind}
															</span>
															{isThemeActive && (
																<span className="text-[9px] font-black text-emerald-400 uppercase">
																	Applied
																</span>
															)}
														</div>

														<h4 className="font-extrabold text-sm text-foreground pt-1 flex items-center gap-1.5">
															<Palette className="h-4.5 w-4.5 text-primary" />
															{theme.name}
														</h4>
														<p className="text-xs text-muted-foreground font-semibold leading-relaxed">
															{theme.description}
														</p>
													</div>

													<div className="pt-4 border-t border-border/20 mt-4 flex justify-end">
														{isThemeActive ? (
															<div className="text-[10px] font-bold text-emerald-400 bg-emerald-500/5 border border-emerald-500/10 px-2 py-1 rounded">
																Currently Applied
															</div>
														) : (
															<button
																onClick={() => handleApplyTheme(theme.id)}
																className="px-3.5 py-1.5 rounded-xl bg-primary hover:bg-primary/95 text-primary-foreground font-extrabold text-xs cursor-pointer text-center"
															>
																Apply Theme
															</button>
														)}
													</div>
												</Card>
											);
										})}
								</div>
							</div>

							<Card className="p-5 bg-card border border-border rounded-2xl shadow">
								<h3 className="text-[10px] font-extrabold text-muted-foreground uppercase tracking-widest mb-4">
									Theme Preview
								</h3>
								<div className="grid grid-cols-[120px_1fr] gap-4 rounded-xl border border-border p-4">
									<div className="space-y-2">
										<div className="rounded-md bg-primary px-3 py-2 text-primary-foreground text-xs font-bold">
											Selected
										</div>
										<div className="rounded-md bg-secondary px-3 py-2 text-xs">
											Sidebar Item
										</div>
									</div>
									<div className="space-y-3">
										<div className="rounded-xl border border-border bg-card p-4 text-sm">
											Sample card with theme colors and borders.
										</div>
										<button className="rounded-xl bg-primary px-4 py-2 text-primary-foreground text-xs font-bold">
											Sample Button
										</button>
										<input
											readOnly
											value="Sample input"
											className="w-full rounded-xl border border-border bg-background px-3 py-2 text-sm"
										/>
									</div>
								</div>
							</Card>
						</div>
					)}

					{/* CONFIG FILES TAB */}
					{activeCategory === "paths" && (
						<div className="space-y-6 w-full xl:max-w-6xl font-sans">
							<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
								{[
									{
										label: "User Profile Configuration",
										path: paths.profile,
										filename: "profile.toml",
										desc: "Identity facts, physical coordinates, occupation role, and dynamic privacy gates.",
									},
									{
										label: "Format Styling Guidelines",
										path: paths.style,
										filename: "style.toml",
										desc: "Determines response verbosity, tone selectors, and structured block guidelines.",
									},
									{
										label: "Topic Preference Mappings",
										path: paths.preferences,
										filename: "preferences.toml",
										desc: "Custom triggering keywords lists and private guidelines notes.",
									},
									{
										label: "Project & Active Goals Contexts",
										path: paths.contexts,
										filename: "contexts.toml",
										desc: "Delineates workspace project facts, landmarks, and trigger boundaries.",
									},
									{
										label: "Terminal Safe Tool Permissions",
										path: paths.tools,
										filename: "tools.toml",
										desc: "Allowed sandbox execution roots, blocked regex matches, and approval steps.",
									},
									{
										label: "Appearance Theme Configuration",
										path: paths.profile
											? paths.profile.replace(
													"profile.toml",
													"marketplace\\appearance.toml",
												)
											: null,
										filename: "appearance.toml",
										desc: "Tracks the active visual theme only. It does not change assistant behavior or tool permissions.",
									},
								].map((item, index) => {
									const handleCopyPath = () => {
										if (item.path) {
											navigator.clipboard.writeText(item.path);
											toast.success(
												`${item.filename} path copied to clipboard!`,
											);
										}
									};

									const handleRevealFile = async () => {
										if (item.path) {
											try {
												await revealItemInDir(item.path);
												toast.success(`Revealed ${item.filename} in folder!`);
											} catch (err: any) {
												toast.error(
													`Failed to reveal path: ${err?.message || err}`,
												);
											}
										}
									};

									return (
										<Card
											key={index}
											className="p-4 bg-secondary/25 border border-border/50 hover:border-border transition-all duration-300 flex flex-col justify-between gap-3 shadow rounded-2xl"
										>
											<div className="space-y-2">
												<div className="flex items-center justify-between gap-2 border-b border-border/20 pb-2">
													<h3 className="font-bold text-xs uppercase tracking-wider text-zinc-200">
														{item.label}
													</h3>
													<Badge
														variant="outline"
														className="text-[9px] font-mono font-bold bg-secondary text-primary border-primary/20 px-1.5 py-0.5 rounded select-none"
													>
														{item.filename}
													</Badge>
												</div>

												<p className="text-xs text-muted-foreground font-semibold leading-relaxed">
													{item.desc}
												</p>

												{item.path ? (
													<div className="text-[10px] font-mono bg-zinc-950 p-2.5 rounded-xl border border-border/20 overflow-x-auto select-all text-zinc-400 select-all font-semibold">
														{item.path}
													</div>
												) : (
													<div className="text-[10px] text-muted-foreground italic bg-secondary/15 p-2.5 rounded-xl border border-border/20 select-none">
														Not yet initialized.
													</div>
												)}
											</div>

											{item.path && (
												<div className="flex gap-2 pt-2 border-t border-border/10 shrink-0">
													<button
														onClick={handleCopyPath}
														className="flex-1 flex items-center justify-center gap-1.5 py-2.5 rounded-xl border border-border/40 hover:border-border bg-secondary/15 hover:bg-secondary/35 text-[10px] font-bold uppercase transition-colors cursor-pointer select-none text-zinc-200"
													>
														<Copy className="h-3.5 w-3.5 text-primary shrink-0" />
														<span>Copy Path</span>
													</button>
													<button
														onClick={handleRevealFile}
														className="flex-1 flex items-center justify-center gap-1.5 py-2.5 rounded-xl border border-border/40 hover:border-border bg-secondary/15 hover:bg-secondary/35 text-[10px] font-bold uppercase transition-colors cursor-pointer select-none text-zinc-200"
													>
														<FolderOpen className="h-3.5 w-3.5 text-primary shrink-0" />
														<span>Reveal</span>
													</button>
												</div>
											)}
										</Card>
									);
								})}
							</div>

							{/* Coexistence info banner */}
							<Card className="p-5 bg-secondary/20 border border-border space-y-3.5 rounded-2xl shadow">
								<div className="flex items-center gap-3 border-b border-border/40 pb-3 select-none">
									<Info className="h-4.5 w-4.5 text-primary shrink-0" />
									<h3 className="font-extrabold text-sm tracking-wide text-foreground uppercase">
										Dual Assistant Coexistence
									</h3>
								</div>

								<p className="text-xs text-muted-foreground leading-relaxed font-semibold">
									`opennivara` utilizes a unified, single Rust library core
									underneath. This means both the terminal CLI and this desktop
									assistant frontend consume the exact same user profile, style
									TOML rules, preferences triggers, and sqlite message history
									directories.
								</p>

								<div className="bg-primary/5 p-4 rounded-xl border border-primary/10 flex items-start gap-3 select-none">
									<Terminal className="h-4.5 w-4.5 text-primary shrink-0 mt-0.5 animate-pulse" />
									<div className="space-y-1.5 flex-1">
										<p className="font-extrabold text-xs text-foreground tracking-wide uppercase">
											The CLI remains fully active and available!
										</p>
										<p className="text-[11px] text-muted-foreground leading-relaxed font-medium">
											You can close this window at any time and run{" "}
											<code className="bg-muted px-1.5 py-0.5 rounded border font-mono text-[10px] text-primary select-all">
												opennivara chat --resume latest
											</code>{" "}
											directly in your terminal shell. All conversations
											compiled here will instantly resume in the terminal!
										</p>
									</div>
								</div>
							</Card>
						</div>
					)}
				</div>
			</div>
		</div>
	);
}
