import { expect, test } from "@playwright/test";

test.describe("OpenNivara CLI Config Hub E2E Tests", () => {
	test.beforeEach(async ({ page }) => {
		// Navigate to the app (running locally on port 1420)
		await page.goto("/");
	});

	test("should load the home screen and display the browser preview mock banner", async ({
		page,
	}) => {
		// Assert the browser mock banner is visible on screen
		const banner = page.locator("#tauri-browser-preview-banner");
		await expect(banner).toBeVisible();
		await expect(banner).toContainText(
			"Browser Preview Mode — using mock data",
		);
		await expect(
			page.getByRole("heading", { name: "OpenNivara Alpha" }),
		).toBeVisible();
	});

	test("should navigate to the Store and view theme details modal", async ({
		page,
	}) => {
		// Click on the Store navigation button in Sidebar
		const storeBtn = page.getByRole("button", { name: "Store", exact: true });
		await expect(storeBtn).toBeVisible();
		await storeBtn.click();

		// Verify Store view header is rendered
		await expect(
			page.getByRole("heading", { name: "Store", exact: true }),
		).toBeVisible();

		await expect(
			page.getByRole("button", { name: "Themes", exact: true }),
		).toBeVisible();
		await expect(
			page.getByRole("button", { name: "Installed Themes" }),
		).toBeVisible();
		await expect(
			page.getByRole("button", { name: "Add-ons" }),
		).not.toBeVisible();
		await expect(
			page.getByRole("button", { name: "Quick Prompts" }),
		).not.toBeVisible();

		// Click "Open Details" on Coding Cyan
		const detailsBtn = page
			.locator("div")
			.filter({ hasText: /^Coding Cyan/ })
			.getByRole("button", { name: "Open Details" });
		if ((await detailsBtn.count()) > 0) {
			await detailsBtn.first().click();
		} else {
			// Fallback selector if needed
			await page.getByRole("button", { name: "Open Details" }).first().click();
		}

		await expect(page.getByText("Theme Details")).toBeVisible();
		await expect(page.getByText("Data-only theme")).toBeVisible();
		await expect(page.getByText("No executable code")).toBeVisible();

		// Close the modal
		await page.getByRole("button", { name: "Close theme details" }).click();
		await expect(page.getByText("Theme Details")).not.toBeVisible();
	});

	test("should navigate to Settings and switch categories", async ({
		page,
	}) => {
		// Click on Settings in Sidebar
		const settingsBtn = page.getByRole("button", {
			name: "Settings",
			exact: true,
		});
		await expect(settingsBtn).toBeVisible();
		await settingsBtn.click();

		// Verify Android-like Settings sidebar is rendered with Profile active by default
		await expect(page.getByText("System Config Hub")).toBeVisible();
		await expect(
			page.getByRole("heading", { name: "User Identity" }),
		).toBeVisible();

		// Switch to Response Style
		const responseStyleBtn = page.getByRole("button", {
			name: "Response Style",
		});
		await responseStyleBtn.click();
		await expect(
			page.getByText("Communication Style Guidelines"),
		).toBeVisible();
		await expect(page.getByText("Formatting & Layout")).toBeVisible();

		// Switch to Topic Prefs
		const topicPrefsBtn = page.getByRole("button", { name: "Topic Prefs" });
		await topicPrefsBtn.click();
		await expect(
			page.getByRole("heading", { name: "Base Topic Preferences" }),
		).toBeVisible();
		await expect(page.getByText("Coding Basics Pack")).not.toBeVisible();

		// Switch to Appearance
		const appearanceBtn = page.getByRole("button", { name: "Appearance" });
		await appearanceBtn.click();
		await expect(page.getByText("Installed Themes")).toBeVisible();
		// Coding Cyan should be displayed
		await expect(page.getByText("Coding Cyan").first()).toBeVisible();

		await expect(
			page.getByRole("button", { name: "Quick Prompts" }),
		).not.toBeVisible();
		await expect(
			page.getByRole("button", { name: "Add-ons" }),
		).not.toBeVisible();
	});
});
