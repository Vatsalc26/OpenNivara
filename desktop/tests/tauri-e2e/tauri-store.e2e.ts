import { clickByText, expectNoText, expectText, waitForApp } from "./helpers";

describe("Tauri Store", () => {
	it("loads real built-in theme contents and shows theme safety details", async () => {
		await waitForApp();
		await clickByText("Store");

		await expectText("OpenNivara Store");
		await expectText("Coding Cyan");
		await expectText("Themes");
		await expectNoText("Add-ons");
		await expectNoText("Quick Prompts");
		await expectNoText("Browser Preview Mode");

		await clickByText("Open Details");
		await expectText("Theme Details");
		await expectText("Data-only theme");
		await expectText("No executable code");
	});
});
