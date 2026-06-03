import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Options } from "@wdio/types";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const desktopDir = __dirname;
const tauriPort = Number(process.env.TAURI_DRIVER_PORT ?? 4444);
const debugAppPath = path.join(
	desktopDir,
	"src-tauri",
	"target",
	"debug",
	process.platform === "win32" ? "desktop.exe" : "desktop",
);
const fallbackEdgeDriver = path.join(desktopDir, ".tools", "msedgedriver.exe");
const edgeDriverPath =
	process.env.EDGE_DRIVER_PATH ??
	(fs.existsSync(fallbackEdgeDriver) ? fallbackEdgeDriver : undefined);
const tauriDriverPath =
	process.env.TAURI_DRIVER_PATH ??
	path.join(
		os.homedir(),
		".cargo",
		"bin",
		process.platform === "win32" ? "tauri-driver.exe" : "tauri-driver",
	);
const testConfigDir =
	process.env.OPENNIVARA_TEST_CONFIG_DIR ??
	path.join(os.tmpdir(), `opennivara-tauri-e2e-${Date.now()}`);
const artifactsDir = path.join(desktopDir, "test-results", "tauri-e2e");

let tauriDriver: ReturnType<typeof spawn> | undefined;
let tauriDriverShouldExit = false;

function tauriE2eEnv() {
	const env = {
		...process.env,
		OPENNIVARA_TEST_CONFIG_DIR: testConfigDir,
		OPENNIVARA_DISABLE_GPU: "1",
	};
	delete env.GEMINI_API_KEY;
	delete env.GOOGLE_API_KEY;
	return env;
}

function printDiagnostics(label: string) {
	console.log(`[tauri-e2e] ${label}`);
	console.log(`[tauri-e2e] cwd=${desktopDir}`);
	console.log(`[tauri-e2e] OPENNIVARA_TEST_CONFIG_DIR=${testConfigDir}`);
	console.log(`[tauri-e2e] debugAppPath=${debugAppPath}`);
	console.log(`[tauri-e2e] tauriDriverPath=${tauriDriverPath}`);
	console.log(`[tauri-e2e] edgeDriverPath=${edgeDriverPath ?? "<not set>"}`);
}

function runChecked(command: string, args: string[], cwd: string) {
	console.log(`[tauri-e2e] running: ${command} ${args.join(" ")}`);
	console.log(`[tauri-e2e] command cwd=${cwd}`);
	const result = spawnSync(command, args, {
		cwd,
		stdio: "inherit",
		shell: process.platform === "win32",
		env: tauriE2eEnv(),
	});

	if (result.status !== 0) {
		throw new Error(`${command} ${args.join(" ")} failed`);
	}
}

function waitForPort(port: number, timeoutMs = 30000) {
	const startedAt = Date.now();

	return new Promise<void>((resolve, reject) => {
		const tryConnect = () => {
			const socket = net.createConnection(port, "127.0.0.1");
			socket.once("connect", () => {
				socket.destroy();
				resolve();
			});
			socket.once("error", () => {
				socket.destroy();
				if (Date.now() - startedAt > timeoutMs) {
					reject(new Error(`tauri-driver did not open port ${port}`));
				} else {
					setTimeout(tryConnect, 500);
				}
			});
		};

		tryConnect();
	});
}

export const config: Options.Testrunner = {
	runner: "local",
	specs: ["./tests/tauri-e2e/**/*.e2e.ts"],
	maxInstances: 1,
	host: "127.0.0.1",
	port: tauriPort,
	logLevel: "info",
	outputDir: artifactsDir,
	waitforTimeout: 30000,
	connectionRetryTimeout: 180000,
	connectionRetryCount: 3,
	framework: "mocha",
	reporters: ["spec"],
	mochaOpts: {
		ui: "bdd",
		timeout: 120000,
	},
	capabilities: [
		{
			"wdio:maxInstances": 1,
			"tauri:options": {
				application: debugAppPath,
			},
		},
	],
	async onPrepare() {
		fs.mkdirSync(testConfigDir, { recursive: true });
		fs.mkdirSync(artifactsDir, { recursive: true });
		process.env.OPENNIVARA_TEST_CONFIG_DIR = testConfigDir;

		printDiagnostics("onPrepare");

		if (process.env.OPENNIVARA_SKIP_TAURI_E2E_BUILD !== "1") {
			runChecked(
				"bun",
				["run", "tauri", "build", "--debug", "--no-bundle"],
				desktopDir,
			);
		}

		if (!fs.existsSync(debugAppPath)) {
			throw new Error(
				`Debug Tauri binary not found at ${debugAppPath}. Run bun run tauri:build:debug first or unset OPENNIVARA_SKIP_TAURI_E2E_BUILD.`,
			);
		}
	},
	async beforeSession() {
		const args = ["--port", String(tauriPort)];
		if (edgeDriverPath) {
			args.push("--native-driver", edgeDriverPath);
		}

		tauriDriver = spawn(tauriDriverPath, args, {
			cwd: desktopDir,
			env: tauriE2eEnv(),
			stdio: ["ignore", "pipe", "pipe"],
		});
		tauriDriverShouldExit = false;

		const logPath = path.join(artifactsDir, "tauri-driver.log");
		const log = fs.createWriteStream(logPath, { flags: "a" });
		tauriDriver.stdout?.pipe(log);
		tauriDriver.stderr?.pipe(log);

		return new Promise<void>((resolve, reject) => {
			tauriDriver?.once("error", (error) => {
				reject(error);
			});
			tauriDriver?.once("exit", (code) => {
				if (!tauriDriverShouldExit) {
					reject(
						new Error(`tauri-driver exited before tests with code ${code}`),
					);
				}
			});
			waitForPort(tauriPort).then(resolve).catch(reject);
		});
	},
	afterSession() {
		tauriDriverShouldExit = true;
		tauriDriver?.kill();
		tauriDriver = undefined;
	},
	async afterTest(test, _context, result) {
		if (!result.passed) {
			const safeName = test.title.replace(/[^a-z0-9]+/gi, "-").toLowerCase();
			const screenshotPath = path.join(artifactsDir, `${safeName}.png`);
			await browser.saveScreenshot(screenshotPath);
			console.error(`Tauri E2E failed. Config dir: ${testConfigDir}`);
		}
	},
	onComplete() {
		tauriDriverShouldExit = true;
		tauriDriver?.kill();
		if (!process.env.OPENNIVARA_KEEP_TAURI_E2E_CONFIG) {
			fs.rmSync(testConfigDir, { recursive: true, force: true });
		}
	},
};
