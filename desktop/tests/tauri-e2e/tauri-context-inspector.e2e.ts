import { $, browser } from "@wdio/globals";

import { clickByText, expectNoText, expectText, waitForApp } from "./helpers";

describe("Tauri Context Inspector", () => {
	it("evaluates hello without hanging and shows UI-only theme metadata", async () => {
		await waitForApp();
		await clickByText("Chat");
		await clickByText("Inspect Context");

		await expectText("CONTEXT INSPECTOR");
		await browser.waitUntil(
			async () => {
				const bodyText = await $("body").getText();
				return !bodyText.includes("Running selector evaluation...");
			},
			{
				timeout: 45000,
				timeoutMsg: "Context Inspector spinner did not finish",
			},
		);
		await expectText("ACTIVE VISUAL THEME");
		await clickByText("Raw Prompt");
		await expectText("Raw Prompt");
		await expectNoText("Enabled Add-ons");
		await expectNoText("Packs in this Mode");
		await expectNoText("Loadout");
	});
});
