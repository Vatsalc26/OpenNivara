type MockHandler = (args: any) => any | Promise<any>;

const mockRegistry = new Map<string, MockHandler>();
const allowedMissingCommands = new Set<string>();

const defaultStyle = {
	schema_version: 1,
	communication: {
		tone: "Helpful",
		detail_level: "Detailed",
		use_examples: true,
		use_step_by_step: true,
		avoid_unexplained_jargon: true,
		ask_fewer_questions: false,
		prefer_actionable_answers: true,
	},
	coding: {
		show_simple_solution_first: true,
		explain_after_code: true,
		prefer_mvp_architecture: true,
		avoid_overengineering: true,
		use_beginner_comments: false,
	},
	formatting: {
		use_markdown: true,
		use_short_sections: true,
		include_next_step: true,
		avoid_long_walls_of_text: true,
	},
	behavior: {
		be_honest_about_uncertainty: true,
		do_not_pretend_to_have_done_things: true,
		do_not_reveal_private_context_unless_relevant: true,
	},
};

const defaultProfile = {
	schema_version: 1,
	identity: {
		display_name: "OpenNivara User",
		full_name: "OpenNivara User",
		gender: "Not Specified",
		pronouns: "they/them",
		date_of_birth: "",
		timezone: "UTC",
	},
	location: {
		country: "United States",
		state_or_region: "California",
		city: "San Francisco",
		living_situation: "",
	},
	languages: {
		preferred_human_language: "English",
		other_human_languages: [],
	},
	technical: {
		coding_level: "Advanced",
		preferred_coding_languages: ["Rust", "TypeScript"],
		current_os: "Linux",
		main_editor: "VS Code",
		secondary_editor: "",
		terminal: "bash",
	},
	personal: {
		occupation_or_role: "Software Engineer",
		education_level: "Bachelor's",
		interests: [],
	},
	privacy: {
		send_identity: true,
		send_location: false,
		send_gender: false,
		send_technical: true,
		send_personal: false,
	},
};

export function mockTauriCommand(cmd: string, response: any | MockHandler) {
	let handler: MockHandler;
	if (typeof response === "function") {
		handler = response;
	} else {
		handler = () => response;
	}

	mockRegistry.set(cmd, (args) => {
		const res = handler(args);
		if (cmd === "get_style" && res && typeof res === "object") {
			return {
				...defaultStyle,
				...res,
				communication: {
					...defaultStyle.communication,
					...(res.communication || {}),
				},
				coding: {
					...defaultStyle.coding,
					...(res.coding || {}),
				},
				formatting: {
					...defaultStyle.formatting,
					...(res.formatting || {}),
				},
				behavior: {
					...defaultStyle.behavior,
					...(res.behavior || {}),
				},
			};
		}
		if (cmd === "get_profile" && res && typeof res === "object") {
			return {
				...defaultProfile,
				...res,
				identity: {
					...defaultProfile.identity,
					...(res.identity || {}),
				},
				location: {
					...defaultProfile.location,
					...(res.location || {}),
				},
				languages: {
					...defaultProfile.languages,
					...(res.languages || {}),
				},
				technical: {
					...defaultProfile.technical,
					...(res.technical || {}),
				},
				personal: {
					...defaultProfile.personal,
					...(res.personal || {}),
				},
				privacy: {
					...defaultProfile.privacy,
					...(res.privacy || {}),
				},
			};
		}
		return res;
	});
}

export function allowMissingTauriCommand(cmd: string) {
	allowedMissingCommands.add(cmd);
}

export function clearTauriMocks() {
	mockRegistry.clear();
	allowedMissingCommands.clear();
	setupDefaultTauriMocks();
}

function setupDefaultTauriMocks() {
	mockTauriCommand("theme_get_active", null);
	mockTauriCommand("theme_store_list", []);
	mockTauriCommand("theme_list_installed", []);
	mockTauriCommand("theme_get_appearance_settings", {
		schema_version: 1,
		active_theme_id: null,
		active_theme_source: null,
	});
	mockTauriCommand("theme_reset", {});
	mockTauriCommand("theme_apply", {});
	mockTauriCommand("theme_install_builtin", {
		id: "mock_theme",
		name: "Mock Theme",
		version: "1.0.0",
		source_kind: "builtin",
		installed_at: "2026-06-01T00:00:00Z",
		manifest_path: "mock/theme.toml",
	});
	mockTauriCommand("theme_uninstall", {});
	mockTauriCommand("marketplace_get_active_addon_theme", null);
	mockTauriCommand("marketplace_get_effective_settings_preview", {
		base_preferences: [],
		addon_preferences: [],
		base_contexts: [],
		addon_contexts: [],
		addon_quick_prompts: [],
		active_theme_id: null,
		active_theme_name: null,
		active_theme_source_pack_id: null,
		disabled_contributions: [],
		enabled_packs: [],
	});
	mockTauriCommand("get_style", {
		schema_version: 1,
		communication: {
			tone: "Helpful",
			detail_level: "Detailed",
			use_examples: true,
			use_step_by_step: true,
			avoid_unexplained_jargon: true,
			ask_fewer_questions: false,
			prefer_actionable_answers: true,
		},
		coding: {
			show_simple_solution_first: true,
			explain_after_code: true,
			prefer_mvp_architecture: true,
			avoid_overengineering: true,
			use_beginner_comments: false,
		},
		formatting: {
			use_markdown: true,
			use_short_sections: true,
			include_next_step: true,
			avoid_long_walls_of_text: true,
		},
		behavior: {
			be_honest_about_uncertainty: true,
			do_not_pretend_to_have_done_things: true,
			do_not_reveal_private_context_unless_relevant: true,
		},
	});
	mockTauriCommand("get_profile", {
		schema_version: 1,
		identity: {
			display_name: "OpenNivara User",
			full_name: "OpenNivara User",
			gender: "Not Specified",
			pronouns: "they/them",
			date_of_birth: "",
			timezone: "UTC",
		},
		location: {
			country: "United States",
			state_or_region: "California",
			city: "San Francisco",
			living_situation: "",
		},
		languages: {
			preferred_human_language: "English",
			other_human_languages: [],
		},
		technical: {
			coding_level: "Advanced",
			preferred_coding_languages: ["Rust", "TypeScript"],
			current_os: "Linux",
			main_editor: "VS Code",
			secondary_editor: "",
			terminal: "bash",
		},
		personal: {
			occupation_or_role: "Software Engineer",
			education_level: "Bachelor's",
			interests: [],
		},
		privacy: {
			send_identity: true,
			send_location: false,
			send_gender: false,
			send_technical: true,
			send_personal: false,
		},
	});
	mockTauriCommand("check_api_key", true);
	mockTauriCommand("marketplace_status", {
		marketplace_dir: "C:\\mock\\marketplace",
		installed_count: 0,
		enabled_count: 0,
		disabled_count: 0,
		modes_count: 0,
		active_mode_id: "Default",
		active_mode_name: "Default",
		missing_pack_ids: [],
		builtin_packs_available: [],
		builtin_resource_path_checked: "C:\\mock\\resources",
		builtin_resource_path_exists: true,
		disabled_packs_in_active_mode: [],
	});
	mockTauriCommand("marketplace_has_legacy_modes", false);
	mockTauriCommand("get_preferences", {
		schema_version: 1,
		sections: [],
	});
	mockTauriCommand("get_contexts", {
		schema_version: 1,
		contexts: [],
	});
	mockTauriCommand("preview_context_for_message", {
		profile_sent: [],
		style_sent: [],
		preferences_sent: [],
		contexts_sent: [],
		contexts_pinned: [],
		contexts_not_sent: [],
		warnings: [],
		final_context_text: "Settings-only context preview.",
		active_theme: null,
	});
	mockTauriCommand("memory_status", {
		db_path: "mock/opennivara_memory.sqlite",
		initialized: true,
		schema_version: 1,
		item_count: 0,
		proposal_count: 0,
		vector_enabled: false,
	});
	mockTauriCommand("memory_validate", {
		db_path: "mock/opennivara_memory.sqlite",
		initialized: true,
		schema_version: 1,
		item_count: 0,
		proposal_count: 0,
		vector_enabled: false,
	});
	mockTauriCommand("memory_repair", {
		db_path: "mock/opennivara_memory.sqlite",
		initialized: true,
		schema_version: 1,
		item_count: 0,
		proposal_count: 0,
		vector_enabled: false,
	});
	mockTauriCommand("memory_get_settings", {
		schema_version: 1,
		mode: "ask_before_saving",
		pause_memory: false,
		private_chat: false,
		allow_location_memories: false,
		sensitive_approval_required: true,
	});
	mockTauriCommand("memory_save_settings", {});
	mockTauriCommand("memory_list_items", []);
	mockTauriCommand("memory_search", []);
	mockTauriCommand("memory_list_facets", []);
	mockTauriCommand("memory_graph_status", {
		node_count: 0,
		edge_count: 0,
		index_count: 0,
		validation_errors: [],
	});
	mockTauriCommand("memory_graph_rebuild", {
		node_count: 0,
		edge_count: 0,
		index_count: 0,
		validation_errors: [],
	});
	mockTauriCommand("memory_graph_memory_context", {
		focus_node_id: "",
		nodes: [],
		edges: [],
		depth: 2,
	});
	mockTauriCommand("runtime_get_context", defaultRuntimeContext());
	mockTauriCommand("runtime_get_model_context_info", defaultModelContextInfo());
	mockTauriCommand("location_get_context", defaultLocationContext());
	mockTauriCommand("location_list_saved_places", []);
	mockTauriCommand("location_save_place", (args: any) => ({
		id: "place_mock",
		...args.input,
		created_at: "2026-06-02T00:00:00Z",
		updated_at: "2026-06-02T00:00:00Z",
		deleted_at: null,
	}));
	mockTauriCommand("location_delete_saved_place", {});
	mockTauriCommand("memory_delete_item", {});
	mockTauriCommand("memory_retract_item", {});
	mockTauriCommand("memory_list_proposals", []);
	mockTauriCommand("memory_extract_proposals_for_message", []);
	mockTauriCommand("memory_approve_proposal", {});
	mockTauriCommand("memory_reject_proposal", {});
	mockTauriCommand("memory_list_tasks", []);
	mockTauriCommand("memory_update_task_status", {});
	mockTauriCommand("memory_compile_context", {
		system_policy: "",
		current_user_message: "",
		recent_conversation_window: "",
		session_summary: "",
		profile_brief: "",
		style_brief: "",
		preference_brief: "",
		memory_brief: "",
		task_reminder_brief: "",
		workspace_brief: "",
		route_brief: "",
		raw_prompt: "",
		token_budget_report: {
			model_context_limit: 1800,
			reserved_output_tokens: 400,
			input_budget_tokens: 1400,
			estimated_prompt_tokens: 0,
			trimmed_sections: [],
			sections: [],
			notes: [],
		},
		audit: {},
		intent: {
			labels: ["normal_chat"],
			confidence: 1,
			reason: "mock",
		},
		included_memory_ids: [],
		included_graph_edge_ids: [],
		runtime_decision: "skipped:not_relevant",
		location_decision: "skipped:not_relevant",
	});
	mockTauriCommand("marketplace_get_modes", {
		active_mode: "Default",
		modes: [],
	});
}

// Initial default mocks setup
setupDefaultTauriMocks();

function defaultLocationContext() {
	return {
		status: "unknown",
		latitude: null,
		longitude: null,
		accuracy_meters: null,
		source: "unknown",
		captured_at: null,
		freshness_seconds: null,
		timezone_hint: null,
		city: null,
		region: null,
		country: null,
		label: null,
		permission_state: "denied",
		privacy_level: "disabled",
	};
}

function defaultRuntimeContext() {
	return {
		now_utc: "2026-06-02T00:00:00Z",
		now_local: "2026-06-02T00:00:00Z",
		timezone: "UTC",
		date_local: "2026-06-02",
		day_of_week: "Tue",
		locale: null,
		calendar_week: 23,
		relative_date_context: {
			today_start: "2026-06-02T00:00:00Z",
			today_end: "2026-06-02T23:59:59Z",
			tomorrow_start: "2026-06-03T00:00:00Z",
			tomorrow_end: "2026-06-03T23:59:59Z",
			yesterday_start: "2026-06-01T00:00:00Z",
			yesterday_end: "2026-06-01T23:59:59Z",
			current_week_start: "2026-06-01T00:00:00Z",
			current_week_end: "2026-06-07T23:59:59Z",
			next_week_start: "2026-06-08T00:00:00Z",
			next_week_end: "2026-06-14T23:59:59Z",
			current_month_start: "2026-06-01T00:00:00Z",
			current_month_end: "2026-06-30T23:59:59Z",
			next_month_start: "2026-07-01T00:00:00Z",
			next_month_end: "2026-07-31T23:59:59Z",
		},
		location: defaultLocationContext(),
		model: defaultModelContextInfo(),
	};
}

function defaultModelContextInfo() {
	return {
		provider: "gemini",
		model_name: "gemini-2.5-flash",
		context_window_tokens: 1_000_000,
		default_reserved_output_tokens: 8192,
		supports_token_counting: false,
		supports_usage_metadata: true,
		tokenizer_strategy: "local_estimate",
	};
}

export function handleMockedCommand(cmd: string, args?: any): Promise<any> {
	const handler = mockRegistry.get(cmd);
	if (handler) {
		try {
			const res = handler(args);
			return Promise.resolve(res);
		} catch (err) {
			return Promise.reject(err);
		}
	}

	if (allowedMissingCommands.has(cmd)) {
		return Promise.resolve(null);
	}

	const errorMsg = `MockTauri error: Missing mock implementation for command "${cmd}". Call mockTauriCommand("${cmd}", response) in your test.`;
	console.error(errorMsg);
	return Promise.reject(new Error(errorMsg));
}

export function isVitest(): boolean {
	return (
		typeof process !== "undefined" &&
		(process.env.NODE_ENV === "test" ||
			(globalThis as any).__VITEST__ !== undefined)
	);
}
