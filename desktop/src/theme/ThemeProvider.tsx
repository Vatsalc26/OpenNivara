import type React from "react";
import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useState,
} from "react";
import { getActiveTheme, type OpenNivaraTheme } from "@/api/marketplaceClient";
import { applyOpenNivaraTheme } from "./themeManager";

interface ThemeContextType {
	activeTheme: OpenNivaraTheme | null;
	refreshTheme: () => Promise<void>;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({ children }: { children: React.ReactNode }) {
	const [activeTheme, setActiveTheme] = useState<OpenNivaraTheme | null>(null);

	const refreshTheme = useCallback(async () => {
		try {
			const theme = await getActiveTheme();
			setActiveTheme(theme);
			applyOpenNivaraTheme(theme);
		} catch (err) {
			console.error("Failed to load active theme:", err);
		}
	}, []);

	useEffect(() => {
		refreshTheme();
	}, [refreshTheme]);

	return (
		<ThemeContext.Provider value={{ activeTheme, refreshTheme }}>
			{children}
		</ThemeContext.Provider>
	);
}

export function useOpenNivaraTheme() {
	const context = useContext(ThemeContext);
	if (context === undefined) {
		throw new Error("useOpenNivaraTheme must be used within a ThemeProvider");
	}
	return context;
}
