import { AlertCircle, X } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";

interface ErrorBannerProps {
	message: string;
	onClose?: () => void;
}

export function ErrorBanner({ message, onClose }: ErrorBannerProps) {
	return (
		<div className="fixed bottom-6 right-6 max-w-md shadow-2xl rounded-xl overflow-hidden border border-destructive/20 animate-in fade-in slide-in-from-bottom-5 duration-300 z-50">
			<Alert
				variant="destructive"
				className="bg-destructive/10 border-0 flex items-start gap-3 p-4"
			>
				<AlertCircle className="h-5 w-5 mt-0.5 text-destructive shrink-0" />
				<div className="flex-1 space-y-1">
					<AlertTitle className="font-semibold text-sm text-destructive-foreground tracking-wide">
						OpenNivara System Alert
					</AlertTitle>
					<AlertDescription className="text-xs text-muted-foreground leading-relaxed">
						{message}
					</AlertDescription>
				</div>
				{onClose && (
					<Button
						variant="ghost"
						size="icon"
						onClick={onClose}
						className="h-6 w-6 text-muted-foreground hover:text-foreground shrink-0 rounded-full"
					>
						<X className="h-4 w-4" />
					</Button>
				)}
			</Alert>
		</div>
	);
}
