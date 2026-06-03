import { spawn } from "node:child_process";

process.env.OPENNIVARA_SKIP_TAURI_E2E_BUILD = "1";

const child = spawn(
	"wdio",
	[
		"run",
		"wdio.tauri.conf.ts",
		"--maxInstances=1",
		"--spec",
		"tests/tauri-e2e/tauri-smoke.e2e.ts",
	],
	{
		stdio: "inherit",
		shell: process.platform === "win32",
		env: process.env,
	},
);

child.on("exit", (code, signal) => {
	if (signal) {
		process.kill(process.pid, signal);
		return;
	}
	process.exit(code ?? 1);
});
