import { create } from "zustand";

interface UiState {
	sidebarCollapsed: boolean;
	commandPaletteOpen: boolean;
	browserPreviewDismissed: boolean;
	lastSettingsRoute: string;
	storeLayoutMode: "grid" | "list";
	setSidebarCollapsed: (collapsed: boolean) => void;
	setCommandPaletteOpen: (open: boolean) => void;
	setBrowserPreviewDismissed: (dismissed: boolean) => void;
	setLastSettingsRoute: (route: string) => void;
	setStoreLayoutMode: (mode: "grid" | "list") => void;
}

export const useUiStore = create<UiState>((set) => ({
	sidebarCollapsed: false,
	commandPaletteOpen: false,
	browserPreviewDismissed: false,
	lastSettingsRoute: "/settings/profile",
	storeLayoutMode: "grid",
	setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
	setCommandPaletteOpen: (open) => set({ commandPaletteOpen: open }),
	setBrowserPreviewDismissed: (dismissed) =>
		set({ browserPreviewDismissed: dismissed }),
	setLastSettingsRoute: (route) => set({ lastSettingsRoute: route }),
	setStoreLayoutMode: (mode) => set({ storeLayoutMode: mode }),
}));
