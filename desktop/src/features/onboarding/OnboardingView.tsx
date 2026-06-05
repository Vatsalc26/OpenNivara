import { useQueryClient } from "@tanstack/react-query";
import {
	CheckCircle2,
	KeyRound,
	Loader2,
	Shield,
	Sparkles,
} from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";
import {
	type FirstRunStatus,
	initializeCleanFirstRun,
} from "@/api/opennivaraClient";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";

type OnboardingStep = "welcome" | "notice" | "gemini" | "setup";

interface OnboardingViewProps {
	status?: FirstRunStatus;
}

export function OnboardingView({ status }: OnboardingViewProps) {
	const queryClient = useQueryClient();
	const [step, setStep] = useState<OnboardingStep>("welcome");
	const [acceptedNotice, setAcceptedNotice] = useState(false);
	const [geminiKey, setGeminiKey] = useState("");
	const [isInitializing, setIsInitializing] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const statusRows = useMemo(
		() => [
			["Profile", status?.profile_exists],
			["Preferences", status?.preferences_exists],
			["Project Contexts", status?.contexts_exists],
			["Memory Database", status?.memory_ready],
			["Skill Library", status?.skills_ready],
		],
		[status],
	);

	const initialize = async (keyOverride?: string) => {
		if (!acceptedNotice || isInitializing) return;
		setIsInitializing(true);
		setError(null);
		setStep("setup");
		try {
			await initializeCleanFirstRun({
				accepted_alpha_notice: true,
				gemini_api_key: keyOverride?.trim() || null,
			});
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: ["firstRunStatus"] }),
				queryClient.invalidateQueries({ queryKey: ["apiKeyReady"] }),
				queryClient.invalidateQueries({ queryKey: ["tools"] }),
				queryClient.invalidateQueries({ queryKey: ["memory"] }),
				queryClient.invalidateQueries({ queryKey: ["skills"] }),
			]);
			toast.success("OpenNivara is ready.");
		} catch (err: any) {
			setError(err?.message || String(err));
			setStep("gemini");
		} finally {
			setIsInitializing(false);
		}
	};

	return (
		<div className="flex min-h-screen items-center justify-center bg-background px-5 py-8 text-foreground">
			<div className="w-full max-w-3xl space-y-5">
				<header className="space-y-3">
					<div className="flex items-center gap-3">
						<div className="flex h-10 w-10 items-center justify-center rounded-lg border border-primary/25 bg-primary/10">
							<Sparkles className="h-5 w-5 text-primary" />
						</div>
						<div>
							<h1 className="text-2xl font-bold tracking-tight">
								OpenNivara Alpha
							</h1>
							<p className="text-sm text-muted-foreground">
								Start with a clean local setup. Nothing demo-like is added for
								you.
							</p>
						</div>
					</div>
				</header>

				{step === "welcome" && (
					<Card className="p-5">
						<div className="space-y-4">
							<div>
								<h2 className="text-lg font-semibold">Welcome</h2>
								<p className="mt-2 max-w-2xl text-sm leading-relaxed text-muted-foreground">
									This setup creates neutral local files, an empty context list,
									empty preferences, memory storage, and the built-in library
									index. It does not install skill packs or invent a profile.
								</p>
							</div>
							<div className="grid gap-2 sm:grid-cols-2">
								{statusRows.map(([label, ready]) => (
									<div
										key={String(label)}
										className="flex items-center justify-between rounded-lg border border-border/40 bg-secondary/10 px-3 py-2 text-sm"
									>
										<span>{label}</span>
										<span className="text-xs text-muted-foreground">
											{ready ? "Ready" : "Clean start"}
										</span>
									</div>
								))}
							</div>
							<div className="flex justify-end">
								<Button onClick={() => setStep("notice")}>Continue</Button>
							</div>
						</div>
					</Card>
				)}

				{step === "notice" && (
					<Card className="p-5">
						<div className="space-y-4">
							<div className="flex items-start gap-3">
								<Shield className="mt-0.5 h-5 w-5 text-primary" />
								<div>
									<h2 className="text-lg font-semibold">
										Alpha Privacy Notice
									</h2>
									<p className="mt-2 max-w-2xl text-sm leading-relaxed text-muted-foreground">
										OpenNivara is alpha software. Local profile, preference,
										context, memory, skill, and key configuration files can be
										created on this computer. Model requests are sent to Gemini
										only when you ask the assistant something, and the context
										inspector shows what may be included.
									</p>
								</div>
							</div>
							<label className="flex items-start gap-3 rounded-lg border border-border/40 bg-secondary/10 p-3 text-sm">
								<input
									type="checkbox"
									checked={acceptedNotice}
									onChange={(event) => setAcceptedNotice(event.target.checked)}
									className="mt-1 h-4 w-4"
								/>
								<span>
									I understand this is an alpha build and want to create clean
									local state on this computer.
								</span>
							</label>
							<div className="flex justify-between gap-2">
								<Button variant="outline" onClick={() => setStep("welcome")}>
									Back
								</Button>
								<Button
									onClick={() => setStep("gemini")}
									disabled={!acceptedNotice}
								>
									Continue
								</Button>
							</div>
						</div>
					</Card>
				)}

				{step === "gemini" && (
					<Card className="p-5">
						<div className="space-y-4">
							<div className="flex items-start gap-3">
								<KeyRound className="mt-0.5 h-5 w-5 text-primary" />
								<div>
									<h2 className="text-lg font-semibold">Gemini Setup</h2>
									<p className="mt-2 max-w-2xl text-sm leading-relaxed text-muted-foreground">
										Add a Gemini API key now, or skip and add it later in the
										app. Desktop users do not need to edit an environment file.
										Keys saved here are stored in local alpha config storage; OS
										keychain storage is planned.
									</p>
								</div>
							</div>
							<div className="space-y-2">
								<label
									htmlFor="gemini-key"
									className="text-xs font-semibold uppercase text-muted-foreground"
								>
									Gemini API Key
								</label>
								<Input
									id="gemini-key"
									type="password"
									value={geminiKey}
									onChange={(event) => setGeminiKey(event.target.value)}
									placeholder="Optional"
									autoComplete="off"
								/>
							</div>
							{error && (
								<div className="rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
									{error}
								</div>
							)}
							<div className="flex flex-wrap justify-between gap-2">
								<Button variant="outline" onClick={() => setStep("notice")}>
									Back
								</Button>
								<div className="flex gap-2">
									<Button
										variant="outline"
										onClick={() => initialize("")}
										disabled={isInitializing}
									>
										Skip For Now
									</Button>
									<Button
										onClick={() => initialize(geminiKey)}
										disabled={isInitializing}
									>
										Create Clean Setup
									</Button>
								</div>
							</div>
						</div>
					</Card>
				)}

				{step === "setup" && (
					<Card className="p-5">
						<div className="flex items-start gap-3">
							{isInitializing ? (
								<Loader2 className="mt-0.5 h-5 w-5 animate-spin text-primary" />
							) : (
								<CheckCircle2 className="mt-0.5 h-5 w-5 text-emerald-500" />
							)}
							<div>
								<h2 className="text-lg font-semibold">
									{isInitializing ? "Preparing OpenNivara" : "Setup Complete"}
								</h2>
								<p className="mt-2 max-w-2xl text-sm leading-relaxed text-muted-foreground">
									{isInitializing
										? "Creating neutral local files and indexes."
										: "Opening the main app now."}
								</p>
							</div>
						</div>
					</Card>
				)}
			</div>
		</div>
	);
}
