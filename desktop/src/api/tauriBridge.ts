import { invoke } from "@tauri-apps/api/core";
import { handleMockedCommand, isVitest } from "../test/mockTauri";
import { handleBrowserPreviewCommand } from "./browserPreviewFixtures";

export const isTauri =
	typeof window !== "undefined" &&
	(window as any).__TAURI_INTERNALS__ !== undefined;

// Injected gorgeous warning banner for browser preview mode
if (typeof document !== "undefined" && !isTauri && !isVitest()) {
	const injectBanner = () => {
		if (document.getElementById("tauri-browser-preview-banner")) return;
		const banner = document.createElement("div");
		banner.id = "tauri-browser-preview-banner";
		banner.style.position = "fixed";
		banner.style.top = "12px";
		banner.style.left = "50%";
		banner.style.transform = "translateX(-50%)";
		banner.style.zIndex = "9999";
		banner.style.padding = "6px 14px";
		banner.style.borderRadius = "12px";
		banner.style.background = "rgba(15, 23, 42, 0.7)";
		banner.style.backdropFilter = "blur(12px)";
		banner.style.border = "1px solid rgba(244, 63, 94, 0.25)";
		banner.style.color = "#f43f5e";
		banner.style.fontSize = "10px";
		banner.style.fontWeight = "bold";
		banner.style.textTransform = "uppercase";
		banner.style.letterSpacing = "0.05em";
		banner.style.boxShadow = "0 4px 20px rgba(0, 0, 0, 0.5)";
		banner.style.pointerEvents = "none";
		banner.style.fontFamily = "system-ui, -apple-system, sans-serif";
		banner.style.display = "flex";
		banner.style.alignItems = "center";
		banner.style.gap = "6px";
		banner.innerHTML = `
      <span style="display:inline-block; width:6px; height:6px; border-radius:50%; background:#f43f5e;"></span>
      Browser Preview Mode — using mock data
    `;
		document.body.appendChild(banner);
	};

	if (document.body) {
		injectBanner();
	} else {
		window.addEventListener("DOMContentLoaded", injectBanner);
	}
}

export async function safeInvoke<T>(cmd: string, args?: any): Promise<T> {
	if (isVitest()) {
		return handleMockedCommand(cmd, args);
	}
	if (!isTauri) {
		return handleBrowserPreviewCommand(cmd, args);
	}
	return invoke<T>(cmd, args);
}
