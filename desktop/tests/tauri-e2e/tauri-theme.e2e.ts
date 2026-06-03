import { browser } from "@wdio/globals";

import { clickByText, expectText, waitForApp } from "./helpers";

describe("Tauri Theme", () => {
	it("applies and resets a real installed theme", async () => {
		await waitForApp();
		await clickByText("Store");
		await expectText("Coding Cyan");
		await clickByText("Install Theme");
		await expectText("Installed");

		await clickByText("Settings");
		await clickByText("Appearance");

		await expectText("CURRENT THEME");
		await expectText("Default Obsidian");

		const before = await browser.execute(() =>
			getComputedStyle(document.documentElement)
				.getPropertyValue("--primary")
				.trim(),
		);

		await clickByText("Apply Theme");
		await browser.waitUntil(
			async () => {
				const after = await browser.execute(() =>
					getComputedStyle(document.documentElement)
						.getPropertyValue("--primary")
						.trim(),
				);
				return after !== before;
			},
			{ timeout: 30000, timeoutMsg: "theme CSS variable did not change" },
		);

		await clickByText("Reset to Default");
		await expectText("Default Obsidian");
	});
});
