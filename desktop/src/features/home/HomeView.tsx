import { useQuery } from "@tanstack/react-query";
import {
	AlertTriangle,
	BookOpen,
	Brain,
	KeyRound,
	type LucideIcon,
	MessageSquare,
	Shield,
	Sparkles,
} from "lucide-react";
import { getMemorySettings } from "@/api/memoryClient";
import { checkGeminiKey, getMapSummary } from "@/api/opennivaraClient";
import { listSkills } from "@/api/skillClient";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";

interface HomeViewProps {
	onNavigate: (view: string, tab?: string) => void;
}

export function HomeView({ onNavigate }: HomeViewProps) {
	const { data: geminiKey } = useQuery({
		queryKey: ["geminiKey"],
		queryFn: checkGeminiKey,
	});
	const { data: memorySettings } = useQuery({
		queryKey: ["memory", "settings"],
		queryFn: getMemorySettings,
	});
	const { data: skills = [] } = useQuery({
		queryKey: ["skills"],
		queryFn: listSkills,
	});
	const { data: workspaceSummary = null } = useQuery({
		queryKey: ["workspace", "summary"],
		queryFn: () => getMapSummary().catch(() => null),
	});

	const enabledSkillCount = skills.filter((skill) => skill.enabled).length;
	const projectStatus = workspaceSummary ? "Available" : "Not added";
	const memoryLabel =
		memorySettings?.private_chat || memorySettings?.pause_memory
			? "Private"
			: memorySettings?.mode === "off"
				? "Off"
				: "Ask before saving";

	const statusCards = [
		{
			icon: KeyRound,
			label: "Gemini",
			value: geminiKey?.available ? "Connected" : "Missing",
			detail: geminiKey?.source
				? `Using ${geminiKey.source}`
				: "Add a key when you are ready to chat.",
		},
		{
			icon: Brain,
			label: "Memory",
			value: memoryLabel,
			detail: "Local memory stays reviewable from Privacy.",
		},
		{
			icon: BookOpen,
			label: "Skills",
			value: `${enabledSkillCount} enabled`,
			detail: "Skill packs are optional and never auto-installed.",
		},
		{
			icon: Shield,
			label: "Projects",
			value: projectStatus,
			detail: "Add a workspace only when you want project context.",
		},
	];

	return (
		<div className="h-full overflow-y-auto bg-background text-foreground">
			<section className="border-b border-border/30 bg-secondary/10 px-6 py-6">
				<div className="flex flex-wrap items-start justify-between gap-5">
					<div className="max-w-2xl space-y-3">
						<div className="flex items-center gap-3 text-primary">
							<Sparkles className="h-5 w-5" />
							<span className="text-xs font-bold uppercase tracking-widest">
								Clean Alpha Workspace
							</span>
						</div>
						<div>
							<h1 className="text-2xl font-bold tracking-tight">
								OpenNivara Alpha
							</h1>
							<p className="mt-2 text-sm leading-relaxed text-muted-foreground">
								Your setup is intentionally empty: no demo profile, no fake
								project context, and no recommended skill pack interruption.
							</p>
						</div>
					</div>
					<div className="rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-200">
						<div className="flex items-center gap-2 font-semibold">
							<AlertTriangle className="h-4 w-4" />
							Alpha build
						</div>
						<p className="mt-1 max-w-sm text-xs leading-relaxed text-amber-100/80">
							Review privacy settings before sharing sensitive information with
							a model.
						</p>
					</div>
				</div>
			</section>

			<main className="space-y-6 px-6 py-6">
				<section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
					{statusCards.map((card) => {
						const Icon = card.icon;
						return (
							<Card key={card.label} className="p-4">
								<div className="flex items-start justify-between gap-3">
									<div>
										<div className="text-xs font-bold uppercase tracking-wide text-muted-foreground">
											{card.label}
										</div>
										<div className="mt-1 text-lg font-semibold text-foreground">
											{card.value}
										</div>
									</div>
									<div className="flex h-9 w-9 items-center justify-center rounded-lg border border-primary/20 bg-primary/10">
										<Icon className="h-4 w-4 text-primary" />
									</div>
								</div>
								<p className="text-xs leading-relaxed text-muted-foreground">
									{card.detail}
								</p>
							</Card>
						);
					})}
				</section>

				<section className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_360px]">
					<div className="space-y-3">
						<h2 className="text-sm font-bold uppercase tracking-wide text-muted-foreground">
							Start
						</h2>
						<div className="grid gap-3 sm:grid-cols-2">
							<ActionButton
								icon={MessageSquare}
								label="Start Chat"
								description="Ask a question with the current privacy settings."
								onClick={() => onNavigate("chat")}
							/>
							<ActionButton
								icon={Shield}
								label="View Privacy"
								description="Control memory, location, and context inclusion."
								onClick={() => onNavigate("memory", "privacy")}
							/>
							<ActionButton
								icon={BookOpen}
								label="Browse Skill Library"
								description="Install optional packs only when you choose them."
								onClick={() => onNavigate("store", "skills")}
							/>
							<ActionButton
								icon={Brain}
								label="Open Memory"
								description="Review local memories and pending proposals."
								onClick={() => onNavigate("memory")}
							/>
						</div>
					</div>

					<Card className="h-fit p-4">
						<div className="flex items-center gap-2 text-sm font-bold">
							<AlertTriangle className="h-4 w-4 text-amber-400" />
							Shared Context
						</div>
						<p className="text-xs leading-relaxed text-muted-foreground">
							OpenNivara can include local profile, preferences, memories,
							skills, runtime hints, and workspace context when they are enabled
							and relevant. Use the chat context inspector before sending
							anything sensitive.
						</p>
						<div className="flex flex-wrap gap-2">
							<Button
								variant="outline"
								size="sm"
								onClick={() => onNavigate("memory", "privacy")}
							>
								<Shield className="h-4 w-4" />
								Learn What Gets Shared
							</Button>
							<Button
								variant="outline"
								size="sm"
								onClick={() => onNavigate("chat")}
							>
								<MessageSquare className="h-4 w-4" />
								Open Inspector
							</Button>
						</div>
					</Card>
				</section>
			</main>
		</div>
	);
}

function ActionButton({
	icon: Icon,
	label,
	description,
	onClick,
}: {
	icon: LucideIcon;
	label: string;
	description: string;
	onClick: () => void;
}) {
	return (
		<button
			type="button"
			onClick={onClick}
			className="rounded-lg border border-border/40 bg-card p-4 text-left text-sm transition-colors hover:bg-muted/40"
		>
			<div className="flex items-center gap-2 font-semibold text-foreground">
				<Icon className="h-4 w-4 text-primary" />
				<span>{label}</span>
			</div>
			<p className="mt-2 text-xs leading-relaxed text-muted-foreground">
				{description}
			</p>
		</button>
	);
}
