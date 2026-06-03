import { AlertTriangle, Check, Copy, RefreshCw } from "lucide-react";
import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
	children: ReactNode;
	resetKey?: any;
}

interface State {
	hasError: boolean;
	error: Error | null;
	copied: boolean;
}

export class ErrorBoundary extends Component<Props, State> {
	public state: State = {
		hasError: false,
		error: null,
		copied: false,
	};

	public static getDerivedStateFromError(error: Error): State {
		return { hasError: true, error, copied: false };
	}

	public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
		console.error(
			"ErrorBoundary caught an uncaught exception:",
			error,
			errorInfo,
		);
	}

	public componentDidUpdate(prevProps: Props) {
		if (this.state.hasError && prevProps.resetKey !== this.props.resetKey) {
			this.reset();
		}
	}

	private reset = () => {
		this.setState({
			hasError: false,
			error: null,
			copied: false,
		});
	};

	private handleCopy = async () => {
		if (!this.state.error) return;
		try {
			await navigator.clipboard.writeText(
				this.state.error.stack || this.state.error.message,
			);
			this.setState({ copied: true });
			setTimeout(() => this.setState({ copied: false }), 2000);
		} catch (err) {
			console.error("Failed to copy error details:", err);
		}
	};

	public render() {
		if (this.state.hasError) {
			return (
				<div className="flex flex-col items-center justify-center p-8 m-4 rounded-2xl bg-red-950/20 border border-red-500/30 backdrop-blur-md text-center max-w-lg mx-auto space-y-4">
					<div className="h-12 w-12 rounded-full bg-red-500/10 flex items-center justify-center border border-red-500/20 text-red-500 shadow-[0_0_15px_rgba(239,68,68,0.1)]">
						<AlertTriangle className="h-6 w-6 animate-pulse" />
					</div>

					<div className="space-y-1">
						<h3 className="font-heading font-extrabold text-sm tracking-wider text-red-400 uppercase">
							Something crashed in this panel
						</h3>
						<p className="text-xs text-muted-foreground max-w-sm leading-relaxed">
							An unexpected error occurred while rendering this interface.
						</p>
					</div>

					{this.state.error && (
						<div className="w-full text-left bg-muted/40 border border-border/40 rounded-xl p-3 text-[10px] font-mono text-muted-foreground/80 leading-relaxed overflow-x-auto max-h-36">
							{this.state.error.message}
						</div>
					)}

					<div className="flex items-center gap-3 w-full justify-center">
						<button
							onClick={this.reset}
							className="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-primary text-primary-foreground font-semibold text-xs uppercase tracking-wider hover:bg-primary/95 transition-all shadow cursor-pointer font-sans"
						>
							<RefreshCw className="h-3.5 w-3.5" />
							<span>Reload Panel</span>
						</button>

						<button
							onClick={this.handleCopy}
							className="flex items-center gap-1.5 px-4 py-2 rounded-xl border border-border/40 hover:bg-secondary/40 text-muted-foreground hover:text-foreground font-semibold text-xs uppercase tracking-wider transition-all cursor-pointer font-sans"
						>
							{this.state.copied ? (
								<>
									<Check className="h-3.5 w-3.5 text-emerald-400" />
									<span>Copied</span>
								</>
							) : (
								<>
									<Copy className="h-3.5 w-3.5" />
									<span>Copy Details</span>
								</>
							)}
						</button>
					</div>
				</div>
			);
		}

		return this.props.children;
	}
}
