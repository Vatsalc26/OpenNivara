import { describe, expect, test } from "vitest";
import { createActor } from "xstate";
import { contextInspectorMachine } from "./contextInspectorMachine";

const preview = {
	active_packs: ["coding_basics"],
	profile_sent: [],
	style_sent: [],
	preferences_sent: [],
	contexts_pinned: [],
	contexts_sent: [],
	final_context_text: "Effective prompt",
} as any;

describe("contextInspectorMachine", () => {
	test("runs a query and resolves the matching request", () => {
		const actor = createActor(contextInspectorMachine).start();

		actor.send({ type: "RUN", query: "hello" });
		expect(actor.getSnapshot().value).toBe("loading");
		expect(actor.getSnapshot().context.query).toBe("hello");
		expect(actor.getSnapshot().context.requestId).toBe(1);

		actor.send({ type: "RESOLVE", preview, requestId: 1 });

		expect(actor.getSnapshot().value).toBe("success");
		expect(actor.getSnapshot().context.preview).toBe(preview);
		expect(actor.getSnapshot().context.error).toBeNull();
	});

	test("ignores stale responses from older requests", () => {
		const actor = createActor(contextInspectorMachine).start();

		actor.send({ type: "RUN", query: "first" });
		actor.send({ type: "RUN", query: "second" });
		actor.send({ type: "RESOLVE", preview, requestId: 1 });

		expect(actor.getSnapshot().value).toBe("loading");
		expect(actor.getSnapshot().context.preview).toBeNull();

		actor.send({ type: "RESOLVE", preview, requestId: 2 });

		expect(actor.getSnapshot().value).toBe("success");
		expect(actor.getSnapshot().context.query).toBe("second");
	});

	test("handles reject, timeout, refresh, and reset transitions", () => {
		const actor = createActor(contextInspectorMachine).start();

		actor.send({ type: "RUN", query: "hello" });
		actor.send({ type: "REJECT", error: "Preview failed", requestId: 1 });
		expect(actor.getSnapshot().value).toBe("error");
		expect(actor.getSnapshot().context.error).toBe("Preview failed");

		actor.send({ type: "REFRESH" });
		expect(actor.getSnapshot().value).toBe("loading");
		expect(actor.getSnapshot().context.requestId).toBe(2);

		actor.send({ type: "TIMEOUT", requestId: 2 });
		expect(actor.getSnapshot().value).toBe("timeout");
		expect(actor.getSnapshot().context.error).toBe(
			"Evaluation timed out. Please try again.",
		);

		actor.send({ type: "RESET" });
		expect(actor.getSnapshot().value).toBe("idle");
		expect(actor.getSnapshot().context).toEqual({
			preview: null,
			error: null,
			query: "",
			requestId: 0,
		});
	});
});
