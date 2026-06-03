export function LoadingState() {
	return (
		<div className="flex flex-col gap-2 p-4 rounded-2xl max-w-[85%] mr-auto chat-bubble-model animate-pulse">
			<div className="flex items-center gap-2 mb-1.5">
				<div className="w-5 h-5 rounded-full bg-primary/35 flex items-center justify-center">
					<div className="w-2.5 h-2.5 rounded-full bg-primary animate-ping" />
				</div>
				<span className="text-xs font-semibold text-primary tracking-wide uppercase">
					OpenNivara is thinking
				</span>
			</div>
			<div className="space-y-2">
				<div className="h-3.5 bg-muted rounded-md w-3/4" />
				<div className="h-3.5 bg-muted rounded-md w-5/6" />
				<div className="h-3.5 bg-muted rounded-md w-1/2" />
			</div>
		</div>
	);
}
