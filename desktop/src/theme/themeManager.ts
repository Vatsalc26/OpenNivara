import type { OpenNivaraTheme } from "@/api/marketplaceClient";

function hexToAlpha(hex: string, opacity: number): string {
	if (!hex?.startsWith("#") || hex.length !== 7) {
		return hex;
	}
	const alphaHex = Math.round(opacity * 255)
		.toString(16)
		.padStart(2, "0");
	return `${hex}${alphaHex}`;
}

/**
 * Applies a OpenNivaraTheme configuration to the document's root element by mapping
 * standard semantic tokens to over 20 CSS custom properties, with smart fallbacks.
 */
export function applyOpenNivaraTheme(theme: OpenNivaraTheme | null) {
	const root = document.documentElement;

	const variables = [
		"--background",
		"--foreground",
		"--card",
		"--card-foreground",
		"--popover",
		"--popover-foreground",
		"--primary",
		"--primary-foreground",
		"--secondary",
		"--secondary-foreground",
		"--muted",
		"--muted-foreground",
		"--accent",
		"--accent-foreground",
		"--destructive",
		"--border",
		"--input",
		"--ring",
		"--sidebar",
		"--sidebar-foreground",
		"--sidebar-primary",
		"--sidebar-primary-foreground",
		"--sidebar-accent",
		"--sidebar-accent-foreground",
		"--sidebar-border",
		"--opennivara-panel",
		"--opennivara-success",
		"--opennivara-warning",
		"--opennivara-danger",
		"--opennivara-titlebar",
		"--opennivara-statusbar",
		"--opennivara-bg-glow-1",
		"--opennivara-bg-glow-2",
		"--opennivara-hover",
		"--opennivara-selected",
		"--opennivara-glow",
		"--opennivara-card-elevated",
	];

	if (!theme) {
		// Clear all theme variables to fall back to index.css defaults
		variables.forEach((v) => root.style.removeProperty(v));
		return;
	}

	const c = theme.colors;

	// Derive secondary, border, etc. with smart fallbacks
	const derivedSecondary = c.panel || c.card;

	// If muted is a hex color (e.g. #71717a), we can append "33" for ~20% opacity border lines
	const derivedBorder =
		c.muted.startsWith("#") && c.muted.length === 7 ? `${c.muted}33` : c.muted;

	const derivedSidebarBorder = derivedBorder;

	// Set Core CSS variables
	root.style.setProperty("--background", c.background);
	root.style.setProperty("--foreground", c.foreground);
	root.style.setProperty("--card", c.card);
	root.style.setProperty("--card-foreground", c.foreground);
	root.style.setProperty("--popover", c.card);
	root.style.setProperty("--popover-foreground", c.foreground);
	root.style.setProperty("--primary", c.primary);
	root.style.setProperty("--primary-foreground", c.background); // usually background contrasts nicely with primary
	root.style.setProperty("--secondary", derivedSecondary);
	root.style.setProperty("--secondary-foreground", c.foreground);
	root.style.setProperty("--muted", c.muted);
	root.style.setProperty("--muted-foreground", c.muted);
	root.style.setProperty("--accent", c.accent);
	root.style.setProperty("--accent-foreground", c.foreground);
	root.style.setProperty("--destructive", c.danger);

	// Derived utility tokens
	root.style.setProperty("--border", derivedBorder);
	root.style.setProperty("--input", c.card);
	root.style.setProperty("--ring", c.primary);

	// Sidebar elements
	root.style.setProperty("--sidebar", c.panel);
	root.style.setProperty("--sidebar-foreground", c.foreground);
	root.style.setProperty("--sidebar-primary", c.primary);
	root.style.setProperty("--sidebar-primary-foreground", c.background);
	root.style.setProperty("--sidebar-accent", c.card);
	root.style.setProperty("--sidebar-accent-foreground", c.foreground);
	root.style.setProperty("--sidebar-border", derivedSidebarBorder);

	// Custom OpenNivara specifics
	root.style.setProperty("--opennivara-panel", c.panel);
	root.style.setProperty("--opennivara-success", c.success);
	root.style.setProperty("--opennivara-warning", c.warning);
	root.style.setProperty("--opennivara-danger", c.danger);
	root.style.setProperty("--opennivara-titlebar", c.panel);
	root.style.setProperty("--opennivara-statusbar", c.panel);

	// Dynamic vibrant background glow and interactive variables
	root.style.setProperty("--opennivara-bg-glow-1", hexToAlpha(c.accent, 0.15));
	root.style.setProperty("--opennivara-bg-glow-2", hexToAlpha(c.primary, 0.05));
	root.style.setProperty("--opennivara-hover", hexToAlpha(c.primary, 0.1));
	root.style.setProperty("--opennivara-selected", hexToAlpha(c.primary, 0.15));
	root.style.setProperty("--opennivara-glow", hexToAlpha(c.primary, 0.2));
	root.style.setProperty("--opennivara-card-elevated", hexToAlpha(c.card, 0.9));
}
