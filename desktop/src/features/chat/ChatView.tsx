import {
	Copy,
	CopyCheck,
	Eye,
	Pin,
	Send,
	Sparkles,
	Terminal,
	User,
	X,
} from "lucide-react";
import React, { useEffect, useRef, useState } from "react";
import Markdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import { askOpenNivara, listPinnedSkills } from "@/api/opennivaraClient";
import { listSkills, type SkillSummary } from "@/api/skillClient";
import { Button } from "@/components/ui/button";
import { LoadingState } from "@/components/ui/LoadingState";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Textarea } from "@/components/ui/textarea";
import { ContextInspector } from "./ContextInspector";

export interface Message {
	role: "user" | "model";
	content: string;
	timestamp: Date;
}

function CodeBlock({
	language,
	codeText,
}: {
	language: string;
	codeText: string;
}) {
	const [copied, setCopied] = React.useState(false);

	const copyCode = async () => {
		try {
			await navigator.clipboard.writeText(codeText);
			setCopied(true);
			setTimeout(() => setCopied(false), 2000);
		} catch (e) {
			console.error(e);
		}
	};

	return (
		<div className="my-3 rounded-xl overflow-hidden border border-zinc-800 bg-zinc-950/40 relative group font-sans">
			<div className="bg-zinc-950 px-4 py-2 text-[10px] font-mono border-b border-zinc-800/80 flex items-center justify-between text-zinc-500 select-none">
				<span>{language.toUpperCase()}</span>
				<button
					onClick={copyCode}
					className="px-2 py-0.5 rounded border border-zinc-850 bg-zinc-900 text-zinc-400 hover:text-zinc-200 transition-colors flex items-center gap-1 cursor-pointer font-sans"
				>
					<span>{copied ? "Copied" : "Copy"}</span>
				</button>
			</div>
			<pre className="p-4 bg-zinc-950/30 font-mono text-xs overflow-x-auto leading-relaxed text-cyan-400 select-all">
				<code>{codeText}</code>
			</pre>
		</div>
	);
}

const markdownComponents: Components = {
	code({
		className,
		children,
		...props
	}: React.ComponentProps<"code"> & { node?: unknown }) {
		const match = /language-(\w+)/.exec(className || "");
		const codeText = String(children).replace(/\n$/, "");
		return match ? (
			<CodeBlock language={match[1]} codeText={codeText} />
		) : (
			<code
				className="bg-zinc-850 border border-zinc-800 px-1.5 py-0.5 rounded font-mono text-xs text-cyan-400 font-semibold"
				{...props}
			>
				{children}
			</code>
		);
	},
	table({ children }: React.ComponentProps<"table">) {
		return (
			<div className="my-3 overflow-x-auto rounded-lg border border-border/30">
				<table className="min-w-full divide-y divide-border/35 text-xs">
					{children}
				</table>
			</div>
		);
	},
	thead({ children }: React.ComponentProps<"thead">) {
		return <thead className="bg-secondary/45">{children}</thead>;
	},
	tbody({ children }: React.ComponentProps<"tbody">) {
		return <tbody className="divide-y divide-border/20">{children}</tbody>;
	},
	tr({ children }: React.ComponentProps<"tr">) {
		return <tr className="hover:bg-muted/10">{children}</tr>;
	},
	th({ children }: React.ComponentProps<"th">) {
		return (
			<th className="px-4 py-2 text-left font-semibold text-foreground uppercase tracking-wider">
				{children}
			</th>
		);
	},
	td({ children }: React.ComponentProps<"td">) {
		return <td className="px-4 py-2 text-muted-foreground">{children}</td>;
	},
	ul({ children }: React.ComponentProps<"ul">) {
		return <ul className="list-disc pl-5 space-y-1.5 my-2.5">{children}</ul>;
	},
	ol({ children }: React.ComponentProps<"ol">) {
		return <ol className="list-decimal pl-5 space-y-1.5 my-2.5">{children}</ol>;
	},
};

interface ChatViewProps {
	currentSessionId: string | null;
	onSessionCreated: (sessionId: string) => void;
	initialMessages?: Message[];
	showInspector?: boolean;
	onToggleInspector?: () => void;
}

const EMPTY_MESSAGES: Message[] = [];

export function ChatView({
	currentSessionId,
	onSessionCreated,
	initialMessages = EMPTY_MESSAGES,
	showInspector: externalShowInspector,
	onToggleInspector,
}: ChatViewProps) {
	const [localShowInspector, setLocalShowInspector] = useState(false);
	const showInspector =
		externalShowInspector !== undefined
			? externalShowInspector
			: localShowInspector;
	const setShowInspector =
		onToggleInspector !== undefined ? onToggleInspector : setLocalShowInspector;

	const [messages, setMessages] = useState<Message[]>(initialMessages);
	const [input, setInput] = useState("");
	const [isLoading, setIsLoading] = useState(false);
	const [copiedId, setCopiedId] = useState<number | null>(null);
	const [enabledSkills, setEnabledSkills] = useState<SkillSummary[]>([]);
	const [selectedSkillId, setSelectedSkillId] = useState("");
	const [keepSelectedSkill, setKeepSelectedSkill] = useState(false);
	const [pinnedSkillIds, setPinnedSkillIds] = useState<string[]>([]);
	const messagesEndRef = useRef<HTMLDivElement>(null);
	const selectedSkill = enabledSkills.find(
		(skill) => skill.id === selectedSkillId,
	);
	const pinnedSkills = enabledSkills.filter((skill) =>
		pinnedSkillIds.includes(skill.id),
	);

	const handleQuickAction = (text: string) => {
		setInput(text);
	};

	useEffect(() => {
		setMessages(initialMessages);
	}, [initialMessages]);

	useEffect(() => {
		messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
	}, []);

	useEffect(() => {
		let active = true;
		listSkills()
			.then((skills) => {
				if (active) {
					setEnabledSkills(skills.filter((skill) => skill.enabled));
				}
			})
			.catch((err) => {
				console.error("Failed to load enabled skills for chat composer", err);
			});
		return () => {
			active = false;
		};
	}, []);

	useEffect(() => {
		let active = true;
		if (!currentSessionId) {
			setPinnedSkillIds([]);
			return;
		}
		listPinnedSkills(currentSessionId)
			.then((ids) => {
				if (active) {
					setPinnedSkillIds(ids);
				}
			})
			.catch((err) => {
				console.error("Failed to load pinned skills for chat composer", err);
			});
		return () => {
			active = false;
		};
	}, [currentSessionId]);

	const handleSend = async () => {
		if (!input.trim() || isLoading) return;

		const userMessage: Message = {
			role: "user",
			content: input.trim(),
			timestamp: new Date(),
		};

		setMessages((prev) => [...prev, userMessage]);
		setInput("");
		setIsLoading(true);

		try {
			// Execute command over tauri bridge into Unified OpenNivara Engine
			const res = await askOpenNivara(
				userMessage.content,
				currentSessionId || undefined,
				selectedSkillId || undefined,
				keepSelectedSkill,
			);

			if (!currentSessionId && res.session_id) {
				onSessionCreated(res.session_id);
			}

			const modelMessage: Message = {
				role: "model",
				content: res.answer,
				timestamp: new Date(),
			};

			setMessages((prev) => [...prev, modelMessage]);
			if (!keepSelectedSkill) {
				setSelectedSkillId("");
			}
			if (keepSelectedSkill && selectedSkillId) {
				setPinnedSkillIds((prev) =>
					prev.includes(selectedSkillId) ? prev : [...prev, selectedSkillId],
				);
			}
		} catch (err: any) {
			const errorMessage: Message = {
				role: "model",
				content: `**System Error**: ${err?.message || err || "Unable to consult with OpenNivara engine."}`,
				timestamp: new Date(),
			};
			setMessages((prev) => [...prev, errorMessage]);
		} finally {
			setIsLoading(false);
		}
	};

	const handleCopy = (text: string, index: number) => {
		navigator.clipboard.writeText(text);
		setCopiedId(index);
		setTimeout(() => setCopiedId(null), 2000);
	};

	const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
		if (e.key === "Enter" && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	};

	return (
		<div className="flex h-full w-full overflow-hidden bg-background/30 backdrop-blur-md">
			{/* Left side: Chat view column */}
			<div className="flex-1 flex flex-col h-full min-w-0">
				{/* Header bar */}
				<div className="px-5 py-3 border-b border-zinc-900 flex items-center justify-between shrink-0 bg-zinc-950/40 select-none">
					<div className="flex items-center gap-2">
						<span className="text-[10px] uppercase font-bold tracking-widest text-zinc-500">
							Consultation Session
						</span>
						{currentSessionId ? (
							<span className="text-[9px] bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 px-2 py-0.5 rounded font-mono font-bold">
								{currentSessionId.substring(0, 8)}
							</span>
						) : (
							<span className="text-[9px] bg-zinc-900 text-zinc-500 border border-zinc-800 px-2 py-0.5 rounded font-semibold font-sans">
								Unsaved Temporary Session
							</span>
						)}
					</div>
					<Button
						variant="ghost"
						size="sm"
						onClick={() => setShowInspector((prev) => !prev)}
						className={`flex items-center gap-1.5 px-3 py-1 rounded-lg text-[10px] font-extrabold uppercase tracking-wider transition-all cursor-pointer ${
							showInspector
								? "bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 shadow-sm"
								: "text-zinc-500 hover:text-zinc-200"
						}`}
					>
						<Eye className="h-3.5 w-3.5" />
						<span>Inspect Context</span>
					</Button>
				</div>

				{/* Upper Timeline */}
				<ScrollArea className="flex-1 px-6 py-6 overflow-y-auto">
					<div className="max-w-4xl mx-auto space-y-6">
						{messages.length === 0 ? (
							<div className="flex flex-col items-center justify-center py-12 text-center space-y-6 max-w-lg mx-auto">
								<div className="w-14 h-14 rounded-2xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center shadow-[0_0_15px_rgba(6,182,212,0.1)]">
									<Sparkles className="h-7 w-7 text-cyan-400 animate-pulse" />
								</div>
								<div className="space-y-2">
									<h3 className="font-extrabold text-base tracking-wider text-zinc-100 font-heading uppercase">
										Consult with OpenNivara
									</h3>
									<p className="text-xs text-zinc-400 leading-relaxed max-w-sm">
										Ask a question, review what context would be shared, or
										start with memory and privacy controls before sending
										anything sensitive.
									</p>
								</div>

								{/* Quick actions cards */}
								<div className="grid grid-cols-2 gap-3 w-full pt-4">
									{[
										{
											label: "Start Private Chat",
											desc: "Keep memory out of this conversation",
											prompt:
												"Start a private chat and explain what memory is excluded.",
										},
										{
											label: "Inspect Shared Context",
											desc: "Preview what may be included",
											prompt: "What context would be shared for this message?",
										},
										{
											label: "Set Up Project",
											desc: "Add context only when you choose",
											prompt: "How do I add a project workspace in OpenNivara?",
										},
										{
											label: "Use Memory Carefully",
											desc: "Understand alpha memory behavior",
											prompt: "Explain how OpenNivara memory works in alpha.",
										},
									].map((act, i) => (
										<button
											key={i}
											onClick={() => handleQuickAction(act.prompt)}
											className="p-3.5 text-left rounded-xl border border-zinc-800 bg-zinc-950/40 hover:bg-zinc-900/60 hover:border-zinc-700 transition-all select-none group cursor-pointer"
										>
											<div className="font-bold text-xs text-zinc-200 group-hover:text-cyan-400 transition-colors leading-tight">
												{act.label}
											</div>
											<div className="text-[10px] text-zinc-500 font-medium leading-normal mt-1">
												{act.desc}
											</div>
										</button>
									))}
								</div>
							</div>
						) : (
							messages.map((msg, index) => (
								<div
									key={index}
									className={`flex gap-4 p-5 rounded-2xl max-w-[85%] ${
										msg.role === "user"
											? "ml-auto bg-primary text-primary-foreground select-none flex-row-reverse"
											: "mr-auto chat-bubble-model shadow-sm"
									}`}
								>
									{/* Avatar icons */}
									<div
										className={`h-8 w-8 rounded-lg flex items-center justify-center shrink-0 ${
											msg.role === "user"
												? "bg-primary-foreground/10"
												: "bg-primary/10 border border-primary/20"
										}`}
									>
										{msg.role === "user" ? (
											<User className="h-4.5 w-4.5 text-primary-foreground" />
										) : (
											<Terminal className="h-4.5 w-4.5 text-primary" />
										)}
									</div>

									<div className="space-y-3 flex-1 overflow-hidden">
										<div className="flex items-center justify-between gap-4">
											<span
												className={`text-[10px] uppercase font-bold tracking-wider ${
													msg.role === "user"
														? "text-primary-foreground/60"
														: "text-primary"
												}`}
											>
												{msg.role === "user" ? "You" : "OpenNivara Assistant"}
											</span>
											{msg.role === "model" && (
												<Button
													variant="ghost"
													size="icon"
													onClick={() => handleCopy(msg.content, index)}
													className="h-6 w-6 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted"
												>
													{copiedId === index ? (
														<CopyCheck className="h-3.5 w-3.5 text-emerald-400" />
													) : (
														<Copy className="h-3.5 w-3.5" />
													)}
												</Button>
											)}
										</div>

										<div
											className={`prose max-w-none text-sm leading-relaxed overflow-wrap break-word ${
												msg.role === "user"
													? "prose-invert text-primary-foreground font-medium"
													: "text-foreground"
											}`}
										>
											<Markdown
												remarkPlugins={[remarkGfm]}
												components={markdownComponents}
											>
												{msg.content}
											</Markdown>
										</div>
									</div>
								</div>
							))
						)}

						{isLoading && <LoadingState />}
						<div ref={messagesEndRef} />
					</div>
				</ScrollArea>

				{/* Input composer box */}
				<div className="px-6 py-4 bg-background/25 border-t border-border/30">
					<div className="max-w-4xl mx-auto flex items-end gap-3 glass-panel p-2 rounded-xl focus-within:ring-1 focus-within:ring-primary/50 focus-within:border-primary/50 transition-all duration-300">
						<div className="flex min-w-[180px] max-w-[240px] flex-col gap-1.5 px-1 pb-0.5">
							<label htmlFor="chat-skill-select" className="sr-only">
								Select skill for message
							</label>
							<select
								id="chat-skill-select"
								aria-label="Select skill for message"
								value={selectedSkillId}
								onChange={(event) => setSelectedSkillId(event.target.value)}
								disabled={isLoading || enabledSkills.length === 0}
								className="h-8 rounded-lg border border-border/50 bg-background/70 px-2 text-[11px] font-semibold text-foreground outline-none disabled:opacity-50"
							>
								<option value="">Auto skills</option>
								{enabledSkills.map((skill) => (
									<option key={skill.id} value={skill.id}>
										{skill.name}
									</option>
								))}
							</select>
							{selectedSkill && (
								<div className="flex items-center gap-1 rounded-lg border border-primary/20 bg-primary/10 px-2 py-1 text-[10px] text-primary">
									<Sparkles className="h-3 w-3 shrink-0" />
									<span className="min-w-0 flex-1 truncate font-bold">
										{selectedSkill.name}
									</span>
									<button
										type="button"
										aria-label="Clear selected skill"
										onClick={() => {
											setSelectedSkillId("");
											setKeepSelectedSkill(false);
										}}
										className="rounded p-0.5 hover:bg-primary/10"
									>
										<X className="h-3 w-3" />
									</button>
								</div>
							)}
							{selectedSkill && (
								<label className="flex items-center gap-1.5 text-[10px] font-semibold text-muted-foreground">
									<input
										type="checkbox"
										aria-label="Keep using selected skill in this chat"
										checked={keepSelectedSkill}
										onChange={(event) =>
											setKeepSelectedSkill(event.target.checked)
										}
										className="h-3 w-3"
									/>
									<span>Keep using</span>
								</label>
							)}
							{pinnedSkills.length > 0 && !selectedSkill && (
								<div className="flex items-center gap-1 rounded-lg border border-border/40 px-2 py-1 text-[10px] text-muted-foreground">
									<Pin className="h-3 w-3 shrink-0 rotate-45" />
									<span className="truncate">
										{pinnedSkills.map((skill) => skill.name).join(", ")}
									</span>
								</div>
							)}
						</div>
						<Textarea
							value={input}
							onChange={(e) => setInput(e.target.value)}
							onKeyDown={handleKeyDown}
							placeholder="Ask OpenNivara a question..."
							className="flex-1 min-h-[44px] max-h-32 bg-transparent border-0 ring-0 focus-visible:ring-0 focus-visible:ring-offset-0 text-sm py-2.5 resize-none leading-relaxed"
						/>
						<Button
							aria-label="Send message"
							size="icon"
							onClick={handleSend}
							disabled={!input.trim() || isLoading}
							className="h-9 w-9 rounded-lg bg-primary hover:bg-primary/95 text-primary-foreground font-semibold flex items-center justify-center shrink-0 shadow"
						>
							<Send className="h-4 w-4" />
						</Button>
					</div>
					<p className="text-[10px] text-center text-muted-foreground/60 mt-2 tracking-wide font-medium">
						Alpha reminder: review the context inspector before sending
						sensitive information to Gemini.
					</p>
				</div>
			</div>

			{/* Right side: Context Inspector side drawer */}
			{showInspector && (
				<ContextInspector
					sessionId={currentSessionId}
					currentInputText={input}
					onClose={() => setShowInspector(false)}
				/>
			)}
		</div>
	);
}
