// TODO: CSP Cleanup - Before public production release, replace the null CSP ('csp': null) in tauri.conf.json with a highly restrictive Content Security Policy.

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { Toaster } from "sonner";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { ThemeProvider } from "@/theme/ThemeProvider";
import { router } from "./router";

const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			refetchOnWindowFocus: false,
			retry: false,
		},
	},
});

export default function App() {
	return (
		<QueryClientProvider client={queryClient}>
			<ThemeProvider>
				<ErrorBoundary>
					<RouterProvider router={router} />
				</ErrorBoundary>
				<Toaster richColors position="top-right" />
			</ThemeProvider>
		</QueryClientProvider>
	);
}
