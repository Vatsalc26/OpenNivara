import { assign, createMachine } from "xstate";
import type { ContextPreview } from "@/api/opennivaraClient";

export interface InspectorContext {
	preview: ContextPreview | null;
	error: string | null;
	query: string;
	requestId: number;
}

export type InspectorEvent =
	| { type: "RUN"; query: string }
	| { type: "REFRESH" }
	| { type: "RESOLVE"; preview: ContextPreview; requestId: number }
	| { type: "REJECT"; error: string; requestId: number }
	| { type: "TIMEOUT"; requestId: number }
	| { type: "CLOSE" }
	| { type: "RESET" };

export const contextInspectorMachine = createMachine({
	id: "contextInspector",
	types: {} as {
		context: InspectorContext;
		events: InspectorEvent;
	},
	context: {
		preview: null,
		error: null,
		query: "",
		requestId: 0,
	},
	initial: "idle",
	states: {
		idle: {
			on: {
				RUN: {
					target: "loading",
					actions: assign({
						query: ({ event }) => event.query,
						requestId: ({ context }) => context.requestId + 1,
						error: () => null,
					}),
				},
			},
		},
		loading: {
			on: {
				RESOLVE: {
					target: "success",
					guard: ({ context, event }) => event.requestId === context.requestId,
					actions: assign({
						preview: ({ event }) => event.preview,
						error: () => null,
					}),
				},
				REJECT: {
					target: "error",
					guard: ({ context, event }) => event.requestId === context.requestId,
					actions: assign({
						error: ({ event }) => event.error,
					}),
				},
				TIMEOUT: {
					target: "timeout",
					guard: ({ context, event }) => event.requestId === context.requestId,
					actions: assign({
						error: () => "Evaluation timed out. Please try again.",
					}),
				},
				RUN: {
					target: "loading",
					actions: assign({
						query: ({ event }) => event.query,
						requestId: ({ context }) => context.requestId + 1,
						error: () => null,
					}),
				},
				REFRESH: {
					target: "loading",
					actions: assign({
						requestId: ({ context }) => context.requestId + 1,
						error: () => null,
					}),
				},
			},
		},
		success: {
			on: {
				RUN: {
					target: "loading",
					actions: assign({
						query: ({ event }) => event.query,
						requestId: ({ context }) => context.requestId + 1,
						error: () => null,
					}),
				},
				REFRESH: {
					target: "loading",
					actions: assign({
						requestId: ({ context }) => context.requestId + 1,
						error: () => null,
					}),
				},
			},
		},
		error: {
			on: {
				RUN: {
					target: "loading",
					actions: assign({
						query: ({ event }) => event.query,
						requestId: ({ context }) => context.requestId + 1,
						error: () => null,
					}),
				},
				REFRESH: {
					target: "loading",
					actions: assign({
						requestId: ({ context }) => context.requestId + 1,
						error: () => null,
					}),
				},
			},
		},
		timeout: {
			on: {
				RUN: {
					target: "loading",
					actions: assign({
						query: ({ event }) => event.query,
						requestId: ({ context }) => context.requestId + 1,
						error: () => null,
					}),
				},
				REFRESH: {
					target: "loading",
					actions: assign({
						requestId: ({ context }) => context.requestId + 1,
						error: () => null,
					}),
				},
			},
		},
	},
	on: {
		RESET: {
			target: ".idle",
			actions: assign({
				preview: () => null,
				error: () => null,
				query: () => "",
				requestId: () => 0,
			}),
		},
	},
});
