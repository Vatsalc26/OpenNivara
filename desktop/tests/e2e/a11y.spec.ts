import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

function criticalViolations(
	results: Awaited<ReturnType<AxeBuilder["analyze"]>>,
) {
	return results.violations.filter(
		(violation) => violation.impact === "critical",
	);
}

test.describe("OpenNivara CLI Config Hub Accessibility Audits", () => {
	test("1. Chat view should not have any critical automatically detectable accessibility issues", async ({
		page,
	}) => {
		await page.goto("/");
		// Wait for the chat page elements to load
		await page.waitForSelector("textarea");

		// Run axe builder on the page
		const results = await new AxeBuilder({ page })
			.withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
			.analyze();

		expect(criticalViolations(results)).toEqual([]);
	});

	test("2. Settings view should not have any critical automatically detectable accessibility issues", async ({
		page,
	}) => {
		await page.goto("/");

		const settingsBtn = page.getByRole("button", {
			name: "Settings",
			exact: true,
		});
		await expect(settingsBtn).toBeVisible();
		await settingsBtn.click();

		await expect(page.getByText("System Config Hub")).toBeVisible();

		const results = await new AxeBuilder({ page })
			.withTags(["wcag2a", "wcag2aa"])
			.analyze();

		expect(criticalViolations(results)).toEqual([]);
	});

	test("3. Store view should not have any critical automatically detectable accessibility issues", async ({
		page,
	}) => {
		await page.goto("/");

		const storeBtn = page.getByRole("button", { name: "Store", exact: true });
		await expect(storeBtn).toBeVisible();
		await storeBtn.click();

		await expect(
			page.getByRole("heading", { name: "Store", exact: true }),
		).toBeVisible();

		const results = await new AxeBuilder({ page })
			.withTags(["wcag2a", "wcag2aa"])
			.analyze();

		expect(criticalViolations(results)).toEqual([]);
	});
});
