import { useQuery } from "@tanstack/react-query";
import {
	createRootRoute,
	createRoute,
	createRouter,
	Outlet,
	useMatches,
	useNavigate,
} from "@tanstack/react-router";
import {
	createContext,
	type Dispatch,
	type SetStateAction,
	useContext,
	useEffect,
	useState,
} from "react";
import {
	checkApiKey,
	getContextsPath,
	getMapSummary,
	getPreferencesPath,
	getProfilePath,
	getSessionMessages,
	getStylePath,
	getTelegramPath,
	getToolsPath,
	listSessions,
	listTools,
} from "@/api/opennivaraClient";
import type { Session } from "@/api/tauriClient";
import { AppShell } from "@/components/app/AppShell";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { ErrorBanner } from "@/components/ui/ErrorBanner";
import { ChatView, type Message } from "@/features/chat/ChatView";
import { StoreView } from "@/features/marketplace/StoreView";
import { MemoryView } from "@/features/memory/MemoryView";
import { SessionsList } from "@/features/sessions/SessionsList";
import { SettingsView } from "@/features/settings/SettingsView";
import {
	type PendingToolCall,
	type ToolApprovalDecision,
	ToolApprovalDialog,
} from "@/features/tools/ToolApprovalDialog";
import { ToolStatusList } from "@/features/tools/ToolStatusList";
import { WorkspaceMapView } from "@/features/workspace/WorkspaceMapView";

interface LayoutState {
	currentSessionId: string | null;
	setCurrentSessionId: (sessionId: string | null) => void;
	sessionMessages: Message[];
	setSessionMessages: (messages: Message[]) => void;
	showInspector: boolean;
	setShowInspector: Dispatch<SetStateAction<boolean>>;
}

const LayoutStateContext = createContext<LayoutState | null>(null);

function useLayoutState() {
	const context = useContext(LayoutStateContext);
	if (!context) {
		throw new Error("useLayoutState must be used inside LayoutComponent");
	}
	return context;
}

// Root Route
export const rootRoute = createRootRoute({
	component: RootComponent,
});

function RootComponent() {
	return <Outlet />;
}

// Layout Route wrapping AppShell
export const layoutRoute = createRoute({
	getParentRoute: () => rootRoute,
	id: "layout",
	component: LayoutComponent,
});

function LayoutComponent() {
	const matches = useMatches();
	const navigate = useNavigate();

	const currentPath = matches[matches.length - 1]?.pathname || "/";
	const activeView = currentPath.startsWith("/settings")
		? "settings"
		: currentPath.startsWith("/store")
			? "marketplace"
			: currentPath.startsWith("/memory")
				? "memory"
				: currentPath.substring(1) || "chat";

	const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);
	const [sessionMessages, setSessionMessages] = useState<Message[]>([]);
	const [globalError, setGlobalError] = useState<string | null>(null);
	const [paletteOpen, setPaletteOpen] = useState(false);
	const [showInspector, setShowInspector] = useState(false);
	const [pendingToolCall, setPendingToolCall] =
		useState<PendingToolCall | null>(null);

	const { data: apiKeyReady = false } = useQuery({
		queryKey: ["apiKeyReady"],
		queryFn: checkApiKey,
	});

	const { data: toolsConfig = null } = useQuery({
		queryKey: ["tools"],
		queryFn: listTools,
	});

	const toolsEnabled = toolsConfig?.general?.enabled ?? false;

	// Invalidate messages on session change
	useEffect(() => {
		if (!currentSessionId) {
			setSessionMessages([]);
			return;
		}

		const fetchMessages = async () => {
			try {
				const dbMsgs = await getSessionMessages(currentSessionId);
				const mapped: Message[] = dbMsgs
					.filter((m) => m.role === "user" || m.role === "model")
					.map((m) => ({
						role: m.role as "user" | "model",
						content: m.content,
						timestamp: new Date(m.created_at),
					}));
				setSessionMessages(mapped);
			} catch (err: any) {
				setGlobalError(`Failed to load messages: ${err?.message || err}`);
			}
		};

		fetchMessages();
	}, [currentSessionId]);

	const handleNavigate = (view: string, tab?: any) => {
		if (view === "settings" || view === "/settings") {
			navigate({ to: `/settings/${tab || "profile"}` });
		} else if (
			view === "marketplace" ||
			view === "store" ||
			view === "/store"
		) {
			navigate({ to: `/store/${tab || "themes"}` });
		} else {
			const destination = view.startsWith("/") ? view : `/${view}`;
			navigate({ to: destination === "/chat" ? "/" : destination });
		}
	};

	const handleNewChat = () => {
		setCurrentSessionId(null);
		setSessionMessages([]);
		navigate({ to: "/" });
	};

	const handleToolDecision = (decision: ToolApprovalDecision) => {
		setPendingToolCall(null);
		console.log("Decision recorded:", decision);
	};

	const RouterOutlet = Outlet as any;
	const layoutState: LayoutState = {
		currentSessionId,
		setCurrentSessionId,
		sessionMessages,
		setSessionMessages,
		showInspector,
		setShowInspector,
	};

	return (
		<AppShell
			activeView={activeView}
			onNavigate={handleNavigate}
			onNewChat={handleNewChat}
			apiKeyReady={apiKeyReady}
			toolsEnabled={toolsEnabled}
			paletteOpen={paletteOpen}
			setPaletteOpen={setPaletteOpen}
			showInspector={showInspector}
			onToggleInspector={() => setShowInspector((prev) => !prev)}
		>
			<div className="flex-1 min-h-0 overflow-hidden relative flex flex-col">
				<LayoutStateContext.Provider value={layoutState}>
					<ErrorBoundary resetKey={currentPath}>
						<RouterOutlet />
					</ErrorBoundary>
				</LayoutStateContext.Provider>
			</div>

			{globalError && (
				<ErrorBanner
					message={globalError}
					onClose={() => setGlobalError(null)}
				/>
			)}

			{pendingToolCall && (
				<ToolApprovalDialog
					pendingCall={pendingToolCall}
					onDecision={handleToolDecision}
					onClose={() => setPendingToolCall(null)}
				/>
			)}
		</AppShell>
	);
}

// Child Routes
export const indexRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/",
	component: IndexView,
});

function IndexView() {
	const context = useLayoutState();

	const handleSessionCreated = (sessionId: string) => {
		context.setCurrentSessionId(sessionId);
	};

	return (
		<ChatView
			currentSessionId={context.currentSessionId}
			onSessionCreated={handleSessionCreated}
			initialMessages={context.sessionMessages}
			showInspector={context.showInspector}
			onToggleInspector={() => context.setShowInspector((p) => !p)}
		/>
	);
}

export const chatRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/chat",
	component: IndexView,
});

export const sessionsRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/sessions",
	component: SessionsView,
});

function SessionsView() {
	const context = useLayoutState();
	const navigate = useNavigate();

	const { data: sessions = [] } = useQuery({
		queryKey: ["sessions"],
		queryFn: listSessions,
	});

	const handleSelectSession = (session: Session) => {
		context.setCurrentSessionId(session.id);
		navigate({ to: "/" });
	};

	return (
		<SessionsList
			sessions={sessions}
			activeSessionId={context.currentSessionId}
			onSelectSession={handleSelectSession}
		/>
	);
}

export const toolsRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/tools",
	component: ToolsView,
});

function ToolsView() {
	const { data: toolsConfig = null } = useQuery({
		queryKey: ["tools"],
		queryFn: listTools,
	});

	const { data: configPaths = { tools: null } } = useQuery({
		queryKey: ["configPaths"],
		queryFn: async () => {
			const tools = await getToolsPath().catch(() => null);
			return { tools };
		},
	});

	return <ToolStatusList config={toolsConfig} configPath={configPaths.tools} />;
}

export const workspaceRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/workspace",
	component: WorkspaceView,
});

function WorkspaceView() {
	const { data: workspaceSummary = null, isLoading: isWorkspaceLoading } =
		useQuery({
			queryKey: ["workspace"],
			queryFn: getMapSummary,
		});

	return (
		<WorkspaceMapView
			summary={workspaceSummary}
			isLoading={isWorkspaceLoading}
		/>
	);
}

export const memoryRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/memory",
	component: () => <MemoryView defaultTab="timeline" />,
});

export const memoryTimelineRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/memory/timeline",
	component: () => <MemoryView defaultTab="timeline" />,
});

export const memorySearchRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/memory/search",
	component: () => <MemoryView defaultTab="search" />,
});

export const memoryReviewRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/memory/review",
	component: () => <MemoryView defaultTab="review" />,
});

export const memoryPeopleRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/memory/people",
	component: () => <MemoryView defaultTab="people" />,
});

export const memoryTasksRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/memory/tasks",
	component: () => <MemoryView defaultTab="tasks" />,
});

export const memoryPrivacyRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/memory/privacy",
	component: () => <MemoryView defaultTab="privacy" />,
});

// Store Routes
export const storeRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/store",
	component: () => <StoreView defaultTab="themes" />,
});

export const storeThemesRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/store/themes",
	component: () => <StoreView defaultTab="themes" />,
});

export const storeInstalledRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/store/installed",
	component: () => <StoreView defaultTab="installed" />,
});

export const storeItemRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/store/item/$itemId",
	component: () => <StoreView defaultTab="installed" />,
});

// Settings Routes
const getSettingsPaths = async () => {
	const [profile, preferences, style, tools, contexts, telegram] =
		await Promise.all([
			getProfilePath().catch(() => null),
			getPreferencesPath().catch(() => null),
			getStylePath().catch(() => null),
			getToolsPath().catch(() => null),
			getContextsPath().catch(() => null),
			getTelegramPath().catch(() => null),
		]);
	return { profile, preferences, style, tools, contexts, telegram };
};

function SettingsRouteWrapper({ defaultTab }: { defaultTab: string }) {
	const {
		data: configPaths = {
			profile: null,
			preferences: null,
			style: null,
			tools: null,
			contexts: null,
			telegram: null,
		},
	} = useQuery({
		queryKey: ["configPaths"],
		queryFn: getSettingsPaths,
	});

	return <SettingsView paths={configPaths} defaultTab={defaultTab} />;
}

export const settingsRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/settings",
	component: () => <SettingsRouteWrapper defaultTab="profile" />,
});

export const settingsProfileRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/settings/profile",
	component: () => <SettingsRouteWrapper defaultTab="profile" />,
});

export const settingsStyleRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/settings/response-style",
	component: () => <SettingsRouteWrapper defaultTab="style" />,
});

export const settingsPreferencesRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/settings/preferences",
	component: () => <SettingsRouteWrapper defaultTab="preferences" />,
});

export const settingsContextsRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/settings/contexts",
	component: () => <SettingsRouteWrapper defaultTab="contexts" />,
});

export const settingsAppearanceRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/settings/appearance",
	component: () => <SettingsRouteWrapper defaultTab="appearance" />,
});

export const settingsConfigFilesRoute = createRoute({
	getParentRoute: () => layoutRoute,
	path: "/settings/config-files",
	component: () => <SettingsRouteWrapper defaultTab="paths" />,
});

// Build Route Tree
const routeTree = rootRoute.addChildren([
	layoutRoute.addChildren([
		indexRoute,
		chatRoute,
		sessionsRoute,
		toolsRoute,
		workspaceRoute,
		memoryRoute,
		memoryTimelineRoute,
		memorySearchRoute,
		memoryReviewRoute,
		memoryPeopleRoute,
		memoryTasksRoute,
		memoryPrivacyRoute,
		storeRoute,
		storeThemesRoute,
		storeInstalledRoute,
		storeItemRoute,
		settingsRoute,
		settingsProfileRoute,
		settingsStyleRoute,
		settingsPreferencesRoute,
		settingsContextsRoute,
		settingsAppearanceRoute,
		settingsConfigFilesRoute,
	]),
]);

export const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
	interface Register {
		router: typeof router;
	}
}
