import { $, browser, expect } from "@wdio/globals";

import { clickByText, expectText, reloadApp, waitForApp } from "./helpers";

describe("Tauri Settings", () => {
	it("round-trips Preferences fields through real Rust config", async () => {
		await waitForApp();
		await clickByText("Settings");
		await clickByText("Topic Prefs");

		await expectText("BASE TOPIC PREFERENCES");
		await clickByText("Add Preference Section");

		const description = await $(
			"(//input[@placeholder='Description of this topic preference'])[last()]",
		);
		await description.scrollIntoView();
		await description.setValue("Tauri E2E preference description");
		await clickByText("Save Preferences");

		await reloadApp();
		await clickByText("Settings");
		await clickByText("Topic Prefs");
		await browser.execute(() => window.scrollTo(0, document.body.scrollHeight));

		await expect(
			$("input[value='Tauri E2E preference description']"),
		).toBeExisting();
	});

	it("round-trips Response Style toggles through real Rust config", async () => {
		await waitForApp();
		await clickByText("Settings");
		await clickByText("Response Style");

		await expectText("COMMUNICATION STYLE GUIDELINES");
		await expectText("CODING OUTPUT GUIDANCE");
		await expectText("FORMATTING & LAYOUT");
		await expectText("BEHAVIOR & INTEGRITY CONSTRAINTS");

		await $(
			"input[placeholder='e.g. clear, direct, beginner-friendly']",
		).setValue("Tauri E2E direct");
		await clickByText("Save Style Guidelines");

		await reloadApp();
		await clickByText("Settings");
		await clickByText("Response Style");

		await expect(
			$("input[placeholder='e.g. clear, direct, beginner-friendly']"),
		).toHaveValue("Tauri E2E direct");
	});
});
