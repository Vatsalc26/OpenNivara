import { $, expect } from "@wdio/globals";

import { clickByText, expectText, waitForApp } from "./helpers";

describe("Tauri Chat", () => {
	it("sends a real desktop chat request without browser-preview mocks", async () => {
		await waitForApp();
		await clickByText("Chat");

		const input = await $(
			"textarea[placeholder='Ask OpenNivara a question about your files or workspace...']",
		);
		await expect(input).toBeExisting();
		await input.setValue("hello");
		await $("button[aria-label='Send message']").click();

		await expectText("OPENNIVARA ASSISTANT");
		await expectText("hello");
		await clickByText("Inspect Context");
		await expectText("CONTEXT INSPECTOR");
	});
});
