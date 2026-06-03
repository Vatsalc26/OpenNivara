import { create } from "zustand";

interface InspectorState {
	isOpen: boolean;
	selectedTab: "summary" | "contexts" | "prompt";
	lastPrompt: string;
	setIsOpen: (open: boolean) => void;
	setSelectedTab: (tab: "summary" | "contexts" | "prompt") => void;
	setLastPrompt: (prompt: string) => void;
}

export const useInspectorStore = create<InspectorState>((set) => ({
	isOpen: false,
	selectedTab: "summary",
	lastPrompt: "",
	setIsOpen: (open) => set({ isOpen: open }),
	setSelectedTab: (tab) => set({ selectedTab: tab }),
	setLastPrompt: (prompt) => set({ lastPrompt: prompt }),
}));
