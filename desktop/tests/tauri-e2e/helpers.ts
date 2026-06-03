import { $, browser, expect } from "@wdio/globals";

export async function waitForApp() {
	await $("body").waitForExist({ timeout: 30000 });
	const deadline = Date.now() + 30000;
	while (Date.now() < deadline) {
		const text = await $("body").getText();
		if (text.includes("OPENNIVARA")) return;
		await browser.pause(500);
	}

	const diagnostics = await browser.execute(() => ({
		href: window.location.href,
		readyState: document.readyState,
		bodyText: document.body.innerText,
		rootHtml: document.getElementById("root")?.innerHTML ?? null,
		bodyHtml: document.body.innerHTML.slice(0, 500),
	}));
	throw new Error(
		`OpenNivara app did not render: ${JSON.stringify(diagnostics)}`,
	);
}

function xpathLiteral(value: string) {
	if (!value.includes("'")) {
		return `'${value}'`;
	}
	return `concat('${value.replace(/'/g, `', "'", '`)}')`;
}

export async function clickByText(text: string) {
	const literal = xpathLiteral(text);
	const target = await $(
		`//*[self::button or self::a or @role='button'][contains(normalize-space(.), ${literal})]`,
	);
	await target.waitForClickable({ timeout: 30000 });
	await target.click();
}

export async function expectText(text: string) {
	await browser.waitUntil(
		async () => {
			const bodyText = await $("body").getText();
			return bodyText.includes(text);
		},
		{ timeout: 30000, timeoutMsg: `Expected body to contain ${text}` },
	);
}

export async function expectNoText(text: string) {
	const bodyText = await $("body").getText();
	expect(bodyText).not.toContain(text);
}

export async function reloadApp() {
	await browser.refresh();
	await waitForApp();
}
