import { Check, ShieldAlert, ShieldCheck, Terminal, X } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";

export type PendingToolCall = {
	id: string;
	name: string;
	args: any;
	risk: "low" | "medium" | "high";
	reason?: string;
};

export type ToolApprovalDecision =
	| { kind: "allow_once"; id: string }
	| { kind: "deny"; id: string }
	| { kind: "always_allow_exact"; id: string };

interface ToolApprovalDialogProps {
	pendingCall: PendingToolCall | null;
	onDecision: (decision: ToolApprovalDecision) => void;
	onClose: () => void;
}

export function ToolApprovalDialog({
	pendingCall,
	onDecision,
	onClose,
}: ToolApprovalDialogProps) {
	if (!pendingCall) return null;

	const riskColors = {
		low: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
		medium: "bg-amber-500/10 text-amber-400 border-amber-500/20",
		high: "bg-rose-500/10 text-rose-400 border-rose-500/20",
	};

	return (
		<Dialog open={!!pendingCall} onOpenChange={() => onClose()}>
			<DialogContent className="glass-panel text-foreground border-border/60 max-w-lg shadow-2xl">
				<DialogHeader className="space-y-3">
					<div className="flex items-center gap-3">
						<div className="h-10 w-10 rounded-xl bg-primary/10 flex items-center justify-center border border-primary/20">
							<ShieldAlert className="h-5 w-5 text-primary" />
						</div>
						<div>
							<DialogTitle className="text-lg font-bold tracking-wide">
								Tool Safety Authorization
							</DialogTitle>
							<DialogDescription className="text-xs text-muted-foreground">
								OpenNivara is requesting permission to interact with your local
								environment.
							</DialogDescription>
						</div>
					</div>
				</DialogHeader>

				<div className="space-y-4 py-4">
					<div className="flex items-center justify-between border-b border-border/40 pb-3">
						<div className="flex items-center gap-2">
							<Terminal className="h-4.5 w-4.5 text-muted-foreground" />
							<span className="font-semibold text-sm tracking-wide font-mono">
								{pendingCall.name}
							</span>
						</div>
						<Badge
							variant="outline"
							className={`${riskColors[pendingCall.risk]} capitalize text-[10px] px-2.5 py-0.5 rounded-full font-semibold`}
						>
							{pendingCall.risk} Risk
						</Badge>
					</div>

					{pendingCall.reason && (
						<div className="text-xs text-muted-foreground bg-secondary/35 p-3 rounded-lg border border-border/30">
							<p className="font-medium text-foreground mb-1">
								Reason for request:
							</p>
							{pendingCall.reason}
						</div>
					)}

					<div className="space-y-2">
						<p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
							Arguments
						</p>
						<pre className="text-xs font-mono bg-secondary/50 p-3 rounded-lg overflow-x-auto max-h-40 border border-border/35 leading-relaxed text-emerald-400">
							{JSON.stringify(pendingCall.args, null, 2)}
						</pre>
					</div>
				</div>

				<DialogFooter className="flex-col sm:flex-row gap-2 sm:gap-0 sm:space-x-2 pt-2">
					<Button
						variant="outline"
						onClick={() => onDecision({ kind: "deny", id: pendingCall.id })}
						className="w-full sm:w-auto text-xs bg-destructive/10 hover:bg-destructive/20 border-destructive/25 text-destructive hover:text-destructive flex items-center justify-center gap-1.5"
					>
						<X className="h-3.5 w-3.5" /> Deny Request
					</Button>
					<Button
						variant="outline"
						onClick={() =>
							onDecision({ kind: "always_allow_exact", id: pendingCall.id })
						}
						className="w-full sm:w-auto text-xs bg-secondary/50 hover:bg-accent/40 border-border flex items-center justify-center gap-1.5"
					>
						<ShieldCheck className="h-3.5 w-3.5 text-primary" /> Always Trust
						App
					</Button>
					<Button
						onClick={() =>
							onDecision({ kind: "allow_once", id: pendingCall.id })
						}
						className="w-full sm:w-auto text-xs bg-primary hover:bg-primary/90 text-primary-foreground font-semibold flex items-center justify-center gap-1.5"
					>
						<Check className="h-3.5 w-3.5" /> Allow Once
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}
