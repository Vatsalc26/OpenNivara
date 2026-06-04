import { Eye, Search, ShieldCheck, Sparkles, X } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import {
	getSkill,
	listSkills,
	type RouteDecision,
	type SkillManifest,
	type SkillSummary,
	setSkillEnabled,
	testSkillRoute,
} from "@/api/skillClient";
import { Card } from "@/components/ui/card";

export function SkillsSettingsPanel() {
	const [skills, setSkills] = useState<SkillSummary[]>([]);
	const [loading, setLoading] = useState(true);
	const [query, setQuery] = useState("");
	const [packFilter, setPackFilter] = useState("all");
	const [categoryFilter, setCategoryFilter] = useState("all");
	const [examFilter, setExamFilter] = useState("all");
	const [routeMessage, setRouteMessage] = useState("");
	const [routeDecision, setRouteDecision] = useState<RouteDecision | null>(
		null,
	);
	const [workingSkillId, setWorkingSkillId] = useState<string | null>(null);
	const [selectedSkill, setSelectedSkill] = useState<SkillManifest | null>(
		null,
	);

	const load = useCallback(async () => {
		setLoading(true);
		try {
			setSkills(await listSkills());
		} catch (err: any) {
			toast.error(`Failed to load skills: ${err?.message || err}`);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		load();
	}, [load]);

	const packs = useMemo(
		() => Array.from(new Set(skills.map((skill) => skill.pack_id || "user"))),
		[skills],
	);
	const categories = useMemo(
		() => Array.from(new Set(skills.map((skill) => skill.category))),
		[skills],
	);
	const exams = useMemo(
		() =>
			Array.from(
				new Set(
					skills
						.map((skill) => skill.exam)
						.filter((exam): exam is string => (exam ?? "").trim().length > 0),
				),
			),
		[skills],
	);

	const filteredSkills = useMemo(() => {
		const normalized = query.trim().toLowerCase();
		return skills.filter((skill) => {
			const pack = skill.pack_id || "user";
			const matchesQuery =
				!normalized ||
				skill.name.toLowerCase().includes(normalized) ||
				skill.id.toLowerCase().includes(normalized) ||
				skill.description.toLowerCase().includes(normalized);
			return (
				matchesQuery &&
				(packFilter === "all" || pack === packFilter) &&
				(categoryFilter === "all" || skill.category === categoryFilter) &&
				(examFilter === "all" || skill.exam === examFilter)
			);
		});
	}, [categoryFilter, examFilter, packFilter, query, skills]);

	const toggleSkill = async (skill: SkillSummary) => {
		setWorkingSkillId(skill.id);
		try {
			await setSkillEnabled(skill.id, !skill.enabled);
			toast.success(`${skill.name} ${skill.enabled ? "disabled" : "enabled"}.`);
			await load();
		} catch (err: any) {
			toast.error(`Failed to update skill: ${err?.message || err}`);
		} finally {
			setWorkingSkillId(null);
		}
	};

	const runRouteTest = async () => {
		if (!routeMessage.trim()) return;
		try {
			setRouteDecision(await testSkillRoute(routeMessage));
		} catch (err: any) {
			toast.error(`Route test failed: ${err?.message || err}`);
		}
	};

	const openSkillDetails = async (skill: SkillSummary) => {
		setWorkingSkillId(skill.id);
		try {
			setSelectedSkill(await getSkill(skill.id));
		} catch (err: any) {
			toast.error(`Failed to load skill details: ${err?.message || err}`);
		} finally {
			setWorkingSkillId(null);
		}
	};

	if (loading) {
		return (
			<div className="text-xs text-muted-foreground">Loading skills...</div>
		);
	}

	return (
		<div className="space-y-5 w-full xl:max-w-6xl font-sans">
			<Card className="p-4 bg-card border border-border rounded-2xl shadow space-y-4">
				<div className="grid grid-cols-1 lg:grid-cols-[1fr_170px_170px_170px] gap-3">
					<div className="relative">
						<Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
						<input
							value={query}
							onChange={(event) => setQuery(event.target.value)}
							placeholder="Search installed skills"
							className="w-full bg-background border border-border rounded-xl py-2 pl-9 pr-3 text-xs outline-none"
						/>
					</div>
					<select
						value={packFilter}
						onChange={(event) => setPackFilter(event.target.value)}
						aria-label="Filter by pack"
						className="bg-background border border-border rounded-xl px-3 py-2 text-xs"
					>
						<option value="all">All packs</option>
						{packs.map((pack) => (
							<option key={pack} value={pack}>
								{pack}
							</option>
						))}
					</select>
					<select
						value={categoryFilter}
						onChange={(event) => setCategoryFilter(event.target.value)}
						aria-label="Filter by category"
						className="bg-background border border-border rounded-xl px-3 py-2 text-xs"
					>
						<option value="all">All categories</option>
						{categories.map((category) => (
							<option key={category} value={category}>
								{category}
							</option>
						))}
					</select>
					<select
						value={examFilter}
						onChange={(event) => setExamFilter(event.target.value)}
						aria-label="Filter by exam"
						className="bg-background border border-border rounded-xl px-3 py-2 text-xs"
					>
						<option value="all">All exams</option>
						{exams.map((exam) => (
							<option key={exam} value={exam}>
								{exam}
							</option>
						))}
					</select>
				</div>
			</Card>

			<div className="grid grid-cols-1 lg:grid-cols-[1fr_360px] gap-4">
				<div className="space-y-3">
					{filteredSkills.length === 0 ? (
						<Card className="p-5 bg-card border border-border rounded-2xl text-xs text-muted-foreground">
							No skills installed yet. Install a Skill Pack from Store, then
							enable skills here.
						</Card>
					) : (
						filteredSkills.map((skill) => (
							<Card
								key={skill.id}
								className="p-4 bg-card border border-border rounded-2xl shadow space-y-3"
							>
								<div className="flex items-start justify-between gap-3">
									<div className="space-y-1">
										<div className="flex items-center gap-2">
											<Sparkles className="h-4 w-4 text-primary" />
											<h3 className="text-sm font-extrabold text-foreground">
												{skill.name}
											</h3>
										</div>
										<p className="text-xs text-muted-foreground leading-relaxed">
											{skill.description}
										</p>
										<div className="flex flex-wrap gap-2 text-[10px] font-bold uppercase text-muted-foreground">
											<span>{skill.pack_id || "user"}</span>
											<span>{skill.category}</span>
											{skill.exam && <span>{skill.exam}</span>}
											{skill.exam_stage && <span>{skill.exam_stage}</span>}
											<span>{skill.route_policy}</span>
										</div>
									</div>
									<button
										type="button"
										onClick={() => toggleSkill(skill)}
										disabled={workingSkillId === skill.id}
										className={`shrink-0 rounded-xl px-3 py-2 text-[10px] font-extrabold uppercase ${
											skill.enabled
												? "bg-primary text-primary-foreground"
												: "bg-secondary text-foreground"
										}`}
									>
										{skill.enabled ? "Enabled" : "Disabled"}
									</button>
								</div>
								<div className="flex flex-wrap gap-2">
									<span className="inline-flex items-center gap-1 rounded-lg border border-border px-2 py-1 text-[10px] text-muted-foreground">
										<ShieldCheck className="h-3 w-3 text-emerald-400" />
										Risk: {skill.risk_level}
									</span>
									<span className="rounded-lg border border-border px-2 py-1 text-[10px] text-muted-foreground">
										Tools:{" "}
										{skill.allowed_tools.length
											? skill.allowed_tools.join(", ")
											: "none"}
									</span>
									{skill.denied_tools.length > 0 && (
										<span className="rounded-lg border border-border px-2 py-1 text-[10px] text-muted-foreground">
											Denied: {skill.denied_tools.join(", ")}
										</span>
									)}
									{skill.freshness_sensitive && (
										<span className="rounded-lg border border-border px-2 py-1 text-[10px] text-muted-foreground">
											Fresh info required
										</span>
									)}
									{skill.best_for.slice(0, 2).map((item) => (
										<span
											key={item}
											className="rounded-lg border border-border px-2 py-1 text-[10px] text-muted-foreground"
										>
											{item}
										</span>
									))}
								</div>
								<div className="flex justify-end">
									<button
										type="button"
										onClick={() => openSkillDetails(skill)}
										disabled={workingSkillId === skill.id}
										className="inline-flex items-center gap-2 rounded-xl border border-border px-3 py-2 text-[10px] font-extrabold uppercase hover:bg-secondary disabled:opacity-60"
									>
										<Eye className="h-3 w-3" />
										Open Details
									</button>
								</div>
							</Card>
						))
					)}
				</div>

				<Card className="p-4 bg-card border border-border rounded-2xl shadow space-y-3 h-fit">
					<h3 className="text-xs font-extrabold uppercase tracking-wide">
						Route Test
					</h3>
					<textarea
						value={routeMessage}
						onChange={(event) => setRouteMessage(event.target.value)}
						placeholder="Type a message to test skill routing"
						className="min-h-24 w-full bg-background border border-border rounded-xl p-3 text-xs outline-none"
					/>
					<button
						type="button"
						onClick={runRouteTest}
						className="w-full rounded-xl bg-primary px-3 py-2 text-xs font-extrabold uppercase text-primary-foreground"
					>
						Test Route
					</button>
					{routeDecision && (
						<div className="space-y-2 text-xs">
							<div className="rounded-xl bg-secondary/30 p-3">
								<div className="font-bold">Primary</div>
								<div className="text-muted-foreground">
									{routeDecision.primary_skill
										? `${routeDecision.primary_skill.name} (${routeDecision.primary_skill.score})`
										: "None"}
								</div>
							</div>
							<div className="space-y-1">
								{routeDecision.candidates.map((candidate) => (
									<div
										key={candidate.id}
										className="rounded-lg border border-border px-2 py-1 text-[10px] text-muted-foreground"
									>
										{candidate.id}: {candidate.score} -{" "}
										{candidate.reason || "no match"}
									</div>
								))}
							</div>
						</div>
					)}
				</Card>
			</div>
			{selectedSkill && (
				<SkillDetailDialog
					skill={selectedSkill}
					onClose={() => setSelectedSkill(null)}
				/>
			)}
		</div>
	);
}

function SkillDetailDialog({
	skill,
	onClose,
}: {
	skill: SkillManifest;
	onClose: () => void;
}) {
	return (
		<div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
			<Card className="max-h-[90vh] w-full max-w-3xl overflow-auto p-5">
				<div className="mb-4 flex items-start justify-between gap-4">
					<div>
						<div className="mb-1 flex items-center gap-2 text-xs font-bold uppercase text-muted-foreground">
							<Sparkles className="h-4 w-4 text-primary" />
							Skill Details
						</div>
						<h2 className="text-lg font-extrabold">{skill.name}</h2>
						<p className="text-xs leading-relaxed text-muted-foreground">
							{skill.description}
						</p>
					</div>
					<button
						type="button"
						onClick={onClose}
						className="rounded-md p-2 hover:bg-muted"
						aria-label="Close skill details"
					>
						<X className="h-4 w-4" />
					</button>
				</div>

				<div className="grid gap-4 md:grid-cols-2">
					<div className="space-y-4">
						<DetailGroup
							label="Best for"
							items={skill.store_preview.best_for}
							empty="Focused study help"
						/>
						<DetailGroup
							label="Sample prompts"
							items={skill.store_preview.sample_prompts}
							empty="Ask OpenNivara to use this skill"
						/>
						<DetailGroup
							label="What it does"
							items={skill.store_preview.what_it_does}
							empty="Guide the requested study workflow"
						/>
					</div>
					<div className="space-y-4">
						<div className="space-y-2 text-xs">
							<h3 className="font-bold">Metadata</h3>
							<div className="grid gap-1 text-muted-foreground">
								<div>Pack: {skill.pack_id || "user"}</div>
								<div>Country: {skill.metadata.country || "unspecified"}</div>
								<div>Exam: {skill.metadata.exam || "general"}</div>
								<div>Stage: {skill.metadata.exam_stage || "general"}</div>
								<div>Route: {skill.route_policy}</div>
								<div>Enabled: {skill.enabled ? "yes" : "no"}</div>
							</div>
						</div>
						<div className="space-y-2 text-xs">
							<h3 className="flex items-center gap-2 font-bold">
								<ShieldCheck className="h-4 w-4 text-emerald-400" />
								Safety
							</h3>
							<div className="flex flex-wrap gap-2">
								<Chip label={`Risk: ${skill.safety.risk_level}`} />
								<Chip label="No install-time enablement" />
								<Chip
									label={
										skill.tools.allow.length
											? `Allowed: ${skill.tools.allow.join(", ")}`
											: "Allowed: none"
									}
								/>
								<Chip
									label={
										skill.tools.deny.length
											? `Denied: ${skill.tools.deny.join(", ")}`
											: "Denied: none"
									}
								/>
							</div>
						</div>
						<DetailGroup
							label="Official source labels"
							items={skill.metadata.official_source_labels}
							empty="No fresh official-source lookup required"
						/>
						<DetailGroup
							label="What it will not do"
							items={skill.store_preview.what_it_will_not_do}
							empty="No unsupported actions declared"
						/>
					</div>
				</div>
			</Card>
		</div>
	);
}

function DetailGroup({
	label,
	items,
	empty,
}: {
	label: string;
	items: string[];
	empty: string;
}) {
	return (
		<div className="space-y-2 text-xs">
			<h3 className="font-bold">{label}</h3>
			{items.length > 0 ? (
				<ul className="space-y-1 text-muted-foreground">
					{items.map((item) => (
						<li key={item}>{item}</li>
					))}
				</ul>
			) : (
				<div className="text-muted-foreground">{empty}</div>
			)}
		</div>
	);
}

function Chip({ label }: { label: string }) {
	return (
		<span className="rounded-lg border border-border px-2 py-1 text-[10px] text-muted-foreground">
			{label}
		</span>
	);
}
