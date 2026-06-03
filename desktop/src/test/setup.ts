import "@testing-library/jest-dom/vitest";
import React from "react";
import { vi } from "vitest";

// Mock ScrollArea to render as a simple div under tests (using createElement to avoid JSX in .ts file)
vi.mock("@/components/ui/scroll-area", () => ({
	ScrollArea: ({ children, className, ...props }: any) =>
		React.createElement("div", { className, ...props }, children),
	ScrollBar: () => null,
}));

// Mock ResizeObserver which is missing in jsdom
class ResizeObserverMock {
	observe() {}
	unobserve() {}
	disconnect() {}
}

global.ResizeObserver = ResizeObserverMock;

// Mock IntersectionObserver which is missing in jsdom
class IntersectionObserverMock {
	readonly root: Element | null = null;
	readonly rootMargin: string = "";
	readonly thresholds: ReadonlyArray<number> = [];
	observe() {}
	unobserve() {}
	disconnect() {}
	takeRecords() {
		return [];
	}
}

global.IntersectionObserver = IntersectionObserverMock as any;

// Stub scrollIntoView for JSDOM
window.HTMLElement.prototype.scrollIntoView = () => {};
