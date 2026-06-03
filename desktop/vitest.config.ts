import path from "node:path";
import { fileURLToPath } from "node:url";
import { storybookTest } from "@storybook/addon-vitest/vitest-plugin";
import react from "@vitejs/plugin-react";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";

const dirname =
	typeof __dirname !== "undefined"
		? __dirname
		: path.dirname(fileURLToPath(import.meta.url));

// More info at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon
export default defineConfig({
	plugins: [react()],
	resolve: {
		alias: {
			"@": path.resolve(dirname, "./src"),
		},
	},
	test: {
		coverage: {
			provider: "v8",
			reporter: ["text", "json", "html"],
			exclude: [
				"src/api/browserPreviewFixtures.ts",
				"src/test/**",
				"tests/**",
				"**/*.test.{ts,tsx}",
				"**/*.spec.{ts,tsx}",
				"**/*.d.ts",
			],
			// Initial thresholds. Future target: 85%+ general, 90%+ critical modules.
			thresholds: {
				statements: 70,
				branches: 60,
				functions: 70,
				lines: 70,
			},
		},
		projects: [
			{
				extends: true,
				test: {
					name: "unit",
					globals: true,
					environment: "jsdom",
					setupFiles: ["./src/test/setup.ts"],
					fileParallelism: false,
					pool: "threads",
					maxWorkers: 1,
					exclude: [
						"node_modules",
						"dist",
						".idea",
						".git",
						".cache",
						"tests/**",
					],
				},
			},
			{
				extends: true,
				plugins: [
					storybookTest({
						configDir: path.join(dirname, ".storybook"),
					}),
				],
				test: {
					name: "storybook",
					browser: {
						enabled: true,
						headless: true,
						provider: playwright({}),
						instances: [{ browser: "chromium" }],
					},
				},
			},
		],
	},
});
