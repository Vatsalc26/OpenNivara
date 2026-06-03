import { clickByText, expectNoText, expectText, waitForApp } from "./helpers";

describe("Tauri smoke", () => {
	it("launches the real desktop app without browser-preview mocks", async () => {
		await waitForApp();

		await expectText("Chat");
		await expectText("SAFE SHELL");
		await expectText("API STATUS");
		await expectText("CONSULT WITH OPENNIVARA");
		await expectNoText("Browser Preview Mode");
		await expectNoText("Missing Command");

		await clickByText("Settings");
		await expectText("ANDROID-LIKE SETTINGS");
	});

	it("opens Memory with audit, graph, and places panels in desktop", async () => {
		await waitForApp();

		await clickByText("Memory");
		await expectText("Life Memory");
		await expectText("Context Compiler Audit");
		await expectNoText("Browser Preview Mode");

		await clickByText("Run");
		await expectText("Runtime:");
		await expectText("Location:");

		await clickByText("Graph");
		await expectText("Graph Index");
		await expectText("Rebuild");

		await clickByText("Places");
		await expectText("Saved Place");
		await expectText("Save Place");
	});
});
