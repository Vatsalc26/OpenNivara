import { afterEach, describe, expect, test, vi } from "vitest";

describe("tauriBridge environment routing", () => {
	afterEach(() => {
		vi.resetModules();
		vi.doUnmock("../test/mockTauri");
		vi.doUnmock("./browserPreviewFixtures");
		vi.doUnmock("@tauri-apps/api/core");
		delete (window as any).__TAURI_INTERNALS__;
		document.body.innerHTML = "";
	});

	test("uses test mock registry when running under Vitest", async () => {
		vi.resetModules();
		vi.doMock("../test/mockTauri", () => ({
			isVitest: () => true,
			handleMockedCommand: vi.fn((_cmd: string, args: any) =>
				Promise.resolve({ source: "vitest", args }),
			),
		}));

		const { safeInvoke } = await import("./tauriBridge");

		await expect(safeInvoke("demo", { value: 1 })).resolves.toEqual({
			source: "vitest",
			args: { value: 1 },
		});
	});

	test("injects browser preview banner and routes to browser fixtures outside Tauri", async () => {
		vi.resetModules();
		const handleBrowserPreviewCommand = vi.fn(() =>
			Promise.resolve({ source: "browser" }),
		);
		vi.doMock("../test/mockTauri", () => ({
			isVitest: () => false,
			handleMockedCommand: vi.fn(),
		}));
		vi.doMock("./browserPreviewFixtures", () => ({
			handleBrowserPreviewCommand,
		}));

		const { safeInvoke } = await import("./tauriBridge");

		expect(
			document.getElementById("tauri-browser-preview-banner"),
		).toHaveTextContent("Browser Preview Mode");
		await expect(safeInvoke("demo", { value: 2 })).resolves.toEqual({
			source: "browser",
		});
		expect(handleBrowserPreviewCommand).toHaveBeenCalledWith("demo", {
			value: 2,
		});
	});

	test("routes to real Tauri invoke when internals are present", async () => {
		vi.resetModules();
		(window as any).__TAURI_INTERNALS__ = {};
		const invoke = vi.fn(() => Promise.resolve({ source: "tauri" }));
		vi.doMock("../test/mockTauri", () => ({
			isVitest: () => false,
			handleMockedCommand: vi.fn(),
		}));
		vi.doMock("@tauri-apps/api/core", () => ({
			invoke,
		}));

		const { isTauri, safeInvoke } = await import("./tauriBridge");

		expect(isTauri).toBe(true);
		await expect(safeInvoke("demo", { value: 3 })).resolves.toEqual({
			source: "tauri",
		});
		expect(invoke).toHaveBeenCalledWith("demo", { value: 3 });
	});
});
