// Centralized mock state storage for browser preview mode
export const mockState = {
	enabledPacks: ["coding_basics"],
	disabledContributions: [] as string[],
	activeThemeId: null as string | null,
	activeThemeSourcePackId: null as string | null,
	memorySettings: {
		schema_version: 1,
		mode: "ask_before_saving",
		pause_memory: false,
		private_chat: false,
		allow_location_memories: false,
		sensitive_approval_required: true,
	},
	memoryItems: [
		{
			id: "mem_mock_bread",
			memory_type: "task",
			title: "Buy bread",
			summary: "User planned to buy bread; completion is not confirmed.",
			details_json: "{}",
			status: "planned",
			confidence: 0.82,
			user_verified: false,
			sensitivity: "normal",
			visibility: "private",
			source_id: "src_mock_bread",
			created_at: "2026-06-02T00:00:00Z",
			updated_at: "2026-06-02T00:00:00Z",
			observed_at: "2026-06-02T00:00:00Z",
			valid_from: null,
			valid_until: null,
			happened_at: null,
			starts_at: null,
			ends_at: null,
			due_at: null,
			completed_at: null,
			timezone: "UTC",
			time_precision: "day",
			natural_time_phrase: "Tuesday",
			recurrence_rule: null,
			superseded_by: null,
			deleted_at: null,
		},
	],
	memoryProposals: [] as any[],
	profile: {
		schema_version: 1,
		identity: {
			display_name: "Developer Pro",
			full_name: "Developer Pro",
			gender: "",
			pronouns: "",
			date_of_birth: "",
			timezone: "",
		},
		location: {
			country: "",
			state_or_region: "",
			city: "",
			living_situation: "",
		},
		languages: {
			preferred_human_language: "English",
			other_human_languages: [],
		},
		technical: {
			coding_level: "Advanced",
			preferred_coding_languages: ["TypeScript", "Rust"],
			current_os: "Windows",
			main_editor: "VS Code",
			secondary_editor: "",
			terminal: "",
		},
		personal: { occupation_or_role: "", education_level: "", interests: [] },
		privacy: {
			send_identity: true,
			send_location: false,
			send_gender: false,
			send_technical: true,
			send_personal: false,
		},
	},
	style: {
		schema_version: 1,
		communication: {
			tone: "Professional",
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
	},
	installedPacks: [
		{
			id: "coding_basics",
			name: "Coding Basics Pack",
			version: "1.0.0",
			installed_at: "2026-06-01T22:00:00Z",
			source: "builtin",
			enabled: true,
		},
	],
	themes: [
		{
			id: "coding_cyan",
			name: "Coding Cyan",
			description: "Sleek high-contrast cyan palette for developer focus.",
			author: "Vatsal Chavda",
			version: "1.0.0",
			source_kind: "builtin",
			installed: true,
			applied: true,
			preview_colors: {
				background: "#0f172a",
				panel: "#111827",
				card: "#1e293b",
				primary: "#06b6d4",
				accent: "#a78bfa",
				success: "#10b981",
				warning: "#f59e0b",
				danger: "#ef4444",
				foreground: "#f8fafc",
				muted: "#64748b",
			},
			safety: {
				data_only: true,
				contains_executable_code: false,
				modifies_tool_security: false,
				requires_network: false,
			},
		},
		{
			id: "calm_focus",
			name: "Calm Focus",
			description: "Low-noise green theme for study sessions.",
			author: "Vatsal Chavda",
			version: "1.0.0",
			source_kind: "builtin",
			installed: false,
			applied: false,
			preview_colors: {
				background: "#0b1120",
				panel: "#102018",
				card: "#12251c",
				primary: "#34d399",
				accent: "#93c5fd",
				success: "#22c55e",
				warning: "#eab308",
				danger: "#f87171",
				foreground: "#ecfdf5",
				muted: "#6b7280",
			},
			safety: {
				data_only: true,
				contains_executable_code: false,
				modifies_tool_security: false,
				requires_network: false,
			},
		},
	],
};

export async function handleBrowserPreviewCommand(
	cmd: string,
	args?: any,
): Promise<any> {
	// -------------------------------------------------------------
	// Core and API clients mocks fallbacks
	// -------------------------------------------------------------
	if (cmd === "ask_opennivara") {
		return {
			session_id: "session_mock_1",
			answer: `Hello! I am OpenNivara running in **Browser Preview Mode**. Since the desktop backend is not active, I am responding with a mock answer. Your message was: "${args.message}"`,
		};
	}
	if (cmd === "list_sessions") {
		return [
			{
				id: "session_mock_1",
				title: "Web Mock Consultation",
				created_at: "2026-06-01T22:00:00Z",
				updated_at: "2026-06-01T22:00:00Z",
				status: "active",
				source_created: "cli",
				active: true,
			},
		];
	}
	if (cmd === "get_session_messages") {
		return [
			{
				id: "msg_1",
				session_id: "session_mock_1",
				role: "user",
				source: "user",
				content: "Hello OpenNivara!",
				created_at: "2026-06-01T22:00:00Z",
				metadata_json: null,
			},
		];
	}
	if (cmd === "check_api_key") {
		return true;
	}
	if (cmd === "check_gemini_key") {
		return {
			available: true,
			source: "browser_preview",
			storage_note: "Browser preview uses mock Gemini availability.",
		};
	}
	if (cmd === "first_run_status") {
		return {
			is_first_run: false,
			required_state_ready: true,
			profile_exists: true,
			style_exists: true,
			preferences_exists: true,
			contexts_exists: true,
			tools_exists: true,
			memory_ready: true,
			marketplace_ready: true,
			skills_ready: true,
			gemini_key: {
				available: true,
				source: "browser_preview",
				storage_note: "Browser preview uses mock Gemini availability.",
			},
		};
	}
	if (cmd === "initialize_clean_first_run") {
		return handleBrowserPreviewCommand("first_run_status");
	}
	if (cmd === "save_gemini_key") {
		return null;
	}
	if (cmd === "map_summary") {
		return "Browser preview workspace summary is mocked.";
	}
	if (cmd === "skills_list") {
		return [];
	}
	if (cmd === "list_tools") {
		return {
			general: {
				enabled: true,
				max_tool_rounds: 5,
				show_tool_activity: true,
			},
			paths: { allowed_roots: [], blocked_patterns: [] },
			tools: {},
		};
	}
	if (cmd === "get_profile") {
		return mockState.profile;
	}
	if (cmd === "save_profile") {
		mockState.profile = args.profile;
		return {};
	}
	if (cmd === "get_style") {
		return mockState.style;
	}
	if (cmd === "save_style") {
		mockState.style = args.style;
		return {};
	}
	if (cmd === "get_preferences") {
		return { schema_version: 1, sections: [] };
	}
	if (cmd === "get_contexts") {
		return { schema_version: 1, contexts: [] };
	}
	if (
		cmd === "memory_status" ||
		cmd === "memory_validate" ||
		cmd === "memory_repair"
	) {
		return {
			db_path: "browser-preview/opennivara_memory.sqlite",
			initialized: true,
			schema_version: 1,
			item_count: mockState.memoryItems.length,
			proposal_count: mockState.memoryProposals.filter(
				(proposal) => proposal.status === "pending",
			).length,
			vector_enabled: false,
		};
	}
	if (cmd === "memory_get_settings") {
		return mockState.memorySettings;
	}
	if (cmd === "memory_save_settings") {
		mockState.memorySettings = args.settings;
		return {};
	}
	if (cmd === "memory_list_items") {
		return mockState.memoryItems;
	}
	if (cmd === "memory_search") {
		const text = String(args.query?.query ?? "").toLowerCase();
		return mockState.memoryItems
			.filter(
				(item) =>
					!text ||
					item.title.toLowerCase().includes(text) ||
					item.summary.toLowerCase().includes(text),
			)
			.map((item) => ({
				item,
				score: 1,
				reason: "browser preview FTS mock",
				answerability: item.status === "planned" ? "planned_only" : "confirmed",
			}));
	}
	if (cmd === "memory_list_facets") {
		return [];
	}
	if (cmd === "memory_graph_status" || cmd === "memory_graph_rebuild") {
		return {
			node_count: mockState.memoryItems.length,
			edge_count: 0,
			index_count: 0,
			validation_errors: [],
		};
	}
	if (cmd === "memory_graph_memory_context") {
		return {
			focus_node_id: `node_memory_items_${args.memoryId}`,
			nodes: mockState.memoryItems
				.filter((item) => item.id === args.memoryId)
				.map((item) => ({
					id: `node_memory_items_${item.id}`,
					node_type: "memory",
					source_table: "memory_items",
					source_id: item.id,
					label: item.title,
					properties_json: "{}",
					sensitivity: item.sensitivity,
					updated_at: item.updated_at,
				})),
			edges: [],
			depth: args.maxDepth ?? 2,
		};
	}
	if (cmd === "runtime_get_context") {
		return browserPreviewRuntimeContext();
	}
	if (cmd === "runtime_get_model_context_info") {
		return browserPreviewModelContextInfo();
	}
	if (cmd === "location_get_context") {
		return browserPreviewLocationContext();
	}
	if (cmd === "location_list_saved_places") {
		return [];
	}
	if (cmd === "location_save_place") {
		return {
			id: "place_preview",
			...args.input,
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString(),
			deleted_at: null,
		};
	}
	if (cmd === "location_delete_saved_place") {
		return {};
	}
	if (cmd === "memory_delete_item") {
		mockState.memoryItems = mockState.memoryItems.filter(
			(item) => item.id !== args.memoryId,
		);
		return {};
	}
	if (cmd === "memory_retract_item") {
		mockState.memoryItems = mockState.memoryItems.map((item) =>
			item.id === args.memoryId ? { ...item, status: "retracted" } : item,
		);
		return {};
	}
	if (cmd === "memory_list_proposals") {
		return mockState.memoryProposals;
	}
	if (cmd === "memory_extract_proposals_for_message") {
		const proposal = {
			id: `proposal_${Date.now()}`,
			source_id: `src_${Date.now()}`,
			proposal_json: JSON.stringify({
				proposed_tasks: [
					{
						title: args.message,
						status: "planned",
						source_quote: args.message,
					},
				],
				confidence: 0.65,
			}),
			sensitivity: "normal",
			confidence: 0.65,
			status: "pending",
			created_at: new Date().toISOString(),
		};
		mockState.memoryProposals.push(proposal);
		return [proposal];
	}
	if (cmd === "memory_approve_proposal") {
		mockState.memoryProposals = mockState.memoryProposals.map((proposal) =>
			proposal.id === args.proposalId
				? { ...proposal, status: "approved" }
				: proposal,
		);
		return {};
	}
	if (cmd === "memory_reject_proposal") {
		mockState.memoryProposals = mockState.memoryProposals.map((proposal) =>
			proposal.id === args.proposalId
				? { ...proposal, status: "rejected" }
				: proposal,
		);
		return {};
	}
	if (cmd === "memory_list_tasks") {
		return mockState.memoryItems
			.filter((item) => item.memory_type === "task")
			.map((item) => ({
				memory_id: item.id,
				task_type: "todo",
				priority: 0,
				status: item.status,
				due_at: item.due_at,
				reminder_at: null,
				completed_at: item.completed_at,
				checklist_json: "[]",
			}));
	}
	if (cmd === "memory_update_task_status") {
		mockState.memoryItems = mockState.memoryItems.map((item) =>
			item.id === args.memoryId ? { ...item, status: args.status } : item,
		);
		return {};
	}
	if (cmd === "memory_compile_context") {
		const includesMemory = String(args.input?.user_message ?? "")
			.toLowerCase()
			.includes("bread");
		return {
			system_policy:
				"Use local-first context and include only relevant memory.",
			current_user_message: args.input?.user_message ?? "",
			recent_conversation_window: "",
			session_summary: "",
			profile_brief: "",
			style_brief: "",
			preference_brief: "",
			memory_brief: includesMemory
				? "- Buy bread (planned, not confirmed)"
				: "",
			task_reminder_brief: "",
			workspace_brief: "",
			route_brief: "",
			raw_prompt: "Browser preview compiled prompt",
			token_budget_report: {
				model_context_limit: 1800,
				reserved_output_tokens: 400,
				input_budget_tokens: 1400,
				estimated_prompt_tokens: includesMemory ? 24 : 8,
				trimmed_sections: [],
				sections: [],
				notes: [],
			},
			audit: {
				id: "audit_preview",
				session_id: null,
				message_id: null,
				user_message: args.input?.user_message ?? "",
				compiled_context_json: "{}",
				included_memory_ids_json: includesMemory ? '["mem_mock_bread"]' : "[]",
				included_task_ids_json: "[]",
				included_workspace_refs_json: "[]",
				token_budget_json: "{}",
				created_at: new Date().toISOString(),
			},
			intent: {
				labels: includesMemory ? ["memory_lookup"] : ["normal_chat"],
				confidence: 0.8,
				reason: includesMemory
					? "Question appears to ask about stored plans"
					: "No memory lookup requested",
			},
			included_memory_ids: includesMemory ? ["mem_mock_bread"] : [],
			included_graph_edge_ids: [],
			runtime_decision: includesMemory
				? "included:relevant_intent"
				: "skipped:not_relevant",
			location_decision: "skipped:not_relevant",
		};
	}

	// -------------------------------------------------------------
	// Addon/Store/Packs new Play-Store layout mocks fallbacks
	// -------------------------------------------------------------
	if (cmd === "marketplace_init") {
		return "Initialized Store (mock)";
	}
	if (cmd === "theme_store_list") {
		return mockState.themes;
	}
	if (cmd === "theme_get_active") {
		const active = mockState.themes.find((theme) => theme.applied);
		if (!active) return null;
		return {
			schema_version: 1,
			id: active.id,
			name: active.name,
			description: active.description,
			colors: active.preview_colors,
			effects: {
				background_gradient: true,
				glow: "medium",
				density: "normal",
			},
		};
	}
	if (cmd === "theme_install_builtin" || cmd === "theme_install_from_path") {
		const themeId = args?.themeId ?? "local_theme";
		mockState.themes = mockState.themes.map((theme) =>
			theme.id === themeId ? { ...theme, installed: true } : theme,
		);
		const theme = mockState.themes.find((item) => item.id === themeId);
		return {
			id: theme?.id ?? themeId,
			name: theme?.name ?? "Local Theme",
			version: theme?.version ?? "1.0.0",
			source_kind: theme?.source_kind ?? "local",
			installed_at: new Date().toISOString(),
			manifest_path: "browser-preview/theme.toml",
		};
	}
	if (cmd === "theme_apply") {
		mockState.themes = mockState.themes.map((theme) => ({
			...theme,
			installed: theme.id === args.themeId ? true : theme.installed,
			applied: theme.id === args.themeId,
		}));
		return {};
	}
	if (cmd === "theme_uninstall") {
		mockState.themes = mockState.themes.map((theme) =>
			theme.id === args.themeId
				? { ...theme, installed: false, applied: false }
				: theme,
		);
		return {};
	}
	if (cmd === "theme_reset") {
		mockState.themes = mockState.themes.map((theme) => ({
			...theme,
			applied: false,
		}));
		return {};
	}
	if (cmd === "marketplace_list_installed_packs") {
		return { schema_version: 1, installed: mockState.installedPacks };
	}
	if (cmd === "marketplace_get_modes") {
		return {
			schema_version: 1,
			active_mode: "default",
			modes: [
				{
					id: "default",
					name: "Default",
					description: "Browser preview default mode",
					enabled_pack_ids: mockState.enabledPacks,
					theme_id: mockState.activeThemeId,
					style_pack_id: mockState.enabledPacks[0] ?? null,
				},
			],
		};
	}
	if (cmd === "marketplace_list_builtin_packs") {
		return [
			{
				id: "coding_basics",
				name: "Coding Basics Pack",
				version: "1.0.0",
				author: "Vatsal Chavda",
				category: "Coding",
				description:
					"Essential style rules, cyan visual theme, and prompt templates for beginner-friendly coding assistance in Rust and TypeScript.",
				risk_level: "low",
			},
			{
				id: "study_coach",
				name: "Study Coach Pack",
				version: "1.1.0",
				author: "EduGroup",
				category: "Study",
				description:
					"Enables interactive active-recall prompt suggestions, learning goal boundaries, and a warm forest-green visual appearance.",
				risk_level: "low",
			},
		];
	}
	if (cmd === "marketplace_preview_pack") {
		return {
			manifest: {
				id: "mock_pack",
				name: "Mock Local Pack",
				version: "1.0.0",
				author: "Local",
				category: "Utility",
				description: "Imported local configuration pack.",
				compatibility: { opennivara_min_version: "0.1.0" },
				contents: { preferences: true, contexts: true, theme: true },
				safety: {
					contains_executable_code: false,
					modifies_tool_permissions: false,
				},
			},
			source_path: args.path,
			warnings: [],
			errors: [],
			additions: {
				preferences_count: 2,
				contexts_count: 1,
				style_presets_count: 0,
				themes_count: 1,
				command_snippets_count: 0,
				workspace_rules_count: 0,
				profile_templates_count: 0,
				tool_presets_count: 0,
			},
			safety_summary: {
				allowed_to_install: true,
				risk_level: "low",
				modifies_tool_permissions: false,
				contains_executable_code: false,
				requires_network: false,
			},
		};
	}
	if (
		cmd === "marketplace_install_pack" ||
		cmd === "marketplace_install_builtin_pack"
	) {
		const newId = args.packId || "study_coach";
		const name =
			newId === "study_coach" ? "Study Coach Pack" : "Coding Basics Pack";
		const packObj = {
			id: newId,
			name,
			version: "1.0.0",
			installed_at: new Date().toISOString(),
			source: "builtin",
			enabled: true,
		};
		if (!mockState.installedPacks.some((p) => p.id === newId)) {
			mockState.installedPacks.push(packObj);
		}
		if (!mockState.enabledPacks.includes(newId)) {
			mockState.enabledPacks.push(newId);
		}
		return packObj;
	}
	if (cmd === "marketplace_uninstall_pack") {
		mockState.installedPacks = mockState.installedPacks.filter(
			(p) => p.id !== args.packId,
		);
		mockState.enabledPacks = mockState.enabledPacks.filter(
			(p) => p !== args.packId,
		);
		return {};
	}
	if (
		cmd === "marketplace_enable_pack" ||
		cmd === "marketplace_toggle_pack_enabled"
	) {
		const enabled = args.enabled !== undefined ? args.enabled : true;
		if (enabled) {
			if (!mockState.enabledPacks.includes(args.packId))
				mockState.enabledPacks.push(args.packId);
		} else {
			mockState.enabledPacks = mockState.enabledPacks.filter(
				(p) => p !== args.packId,
			);
		}
		return {};
	}
	if (cmd === "marketplace_disable_pack") {
		mockState.enabledPacks = mockState.enabledPacks.filter(
			(p) => p !== args.packId,
		);
		return {};
	}
	if (cmd === "marketplace_get_pack_activation_capabilities") {
		return {
			pack_id: args.packId,
			has_theme: true,
			theme_id: args.packId === "study_coach" ? "forest_green" : "coding_cyan",
			theme_name:
				args.packId === "study_coach" ? "Forest Green" : "Coding Cyan",
			has_style: true,
			has_preferences: true,
			has_contexts: true,
			has_command_snippets: true,
			has_workspace_rules: false,
		};
	}
	if (cmd === "marketplace_get_addon_settings") {
		return {
			schema_version: 1,
			active_theme_id: mockState.activeThemeId,
			active_theme_source_pack_id: mockState.activeThemeSourcePackId,
			enabled_packs: mockState.enabledPacks,
			disabled_contributions: mockState.disabledContributions,
		};
	}
	if (cmd === "marketplace_get_active_theme") {
		return null;
	}
	if (cmd === "marketplace_get_active_addon_theme") {
		if (!mockState.activeThemeId) return null;
		return {
			schema_version: 1,
			id: mockState.activeThemeId,
			name:
				mockState.activeThemeId === "study_coach"
					? "Forest Green"
					: mockState.activeThemeId === "coding_basics"
						? "Coding Cyan"
						: "Default Theme",
			description: "Mock theme colors",
			colors: {
				background:
					mockState.activeThemeId === "study_coach" ? "#14532d" : "#0f172a",
				panel: "#1e293b",
				card: "#1e293b",
				primary:
					mockState.activeThemeId === "study_coach" ? "#22c55e" : "#06b6d4",
				accent: "#a78bfa",
				success: "#10b981",
				warning: "#f59e0b",
				danger: "#ef4444",
				foreground: "#f8fafc",
				muted: "#64748b",
			},
			effects: {
				background_gradient: false,
				glow: "low",
				density: "medium",
			},
		};
	}
	if (
		cmd === "marketplace_preview_builtin_pack" ||
		cmd === "marketplace_preview_installed_pack"
	) {
		const packId = args.packId;
		if (packId === "coding_basics") {
			return {
				manifest: {
					schema_version: 1,
					id: "coding_basics",
					name: "Coding Basics Pack",
					version: "1.0.0",
					author: "Vatsal Chavda",
					category: "Coding",
					description:
						"Essential style rules, cyan visual theme, and prompt templates for beginner-friendly coding assistance in Rust and TypeScript.",
					homepage: "https://github.com/Vatsalc26/opennivara",
					license: "MIT",
					compatibility: {
						opennivara_min_version: "0.1.0",
						opennivara_max_version: "",
					},
					contents: {
						preferences: true,
						contexts: true,
						style_presets: true,
						profile_templates: false,
						tool_presets: false,
						workspace_map_rules: false,
						prompt_behaviors: false,
						command_snippets: true,
						theme: true,
					},
					safety: {
						contains_executable_code: false,
						modifies_tool_permissions: false,
						requires_network: false,
						risk_level: "low",
					},
				},
				source_path: "/builtin/coding_basics",
				warnings: [],
				errors: [],
				additions: {
					preferences_count: 2,
					contexts_count: 1,
					style_presets_count: 0,
					themes_count: 1,
					command_snippets_count: 0,
					workspace_rules_count: 0,
					profile_templates_count: 0,
					tool_presets_count: 0,
				},
				safety_summary: {
					allowed_to_install: true,
					risk_level: "low",
					modifies_tool_permissions: false,
					contains_executable_code: false,
					requires_network: false,
				},
			};
		}
		return {
			manifest: {
				schema_version: 1,
				id: "study_coach",
				name: "Study Coach Pack",
				version: "1.1.0",
				author: "EduGroup",
				category: "Study",
				description:
					"Enables interactive active-recall prompt suggestions, learning goal boundaries, and a warm forest-green visual appearance.",
				homepage: "",
				license: "",
				compatibility: {
					opennivara_min_version: "0.1.0",
					opennivara_max_version: "",
				},
				contents: {
					preferences: true,
					contexts: false,
					style_presets: false,
					profile_templates: false,
					tool_presets: false,
					workspace_map_rules: false,
					prompt_behaviors: false,
					command_snippets: true,
					theme: true,
				},
				safety: {
					contains_executable_code: false,
					modifies_tool_permissions: false,
					requires_network: false,
					risk_level: "low",
				},
			},
			source_path: "/builtin/study_coach",
			warnings: [],
			errors: [],
			additions: {
				preferences_count: 1,
				contexts_count: 0,
				style_presets_count: 0,
				themes_count: 1,
				command_snippets_count: 0,
				workspace_rules_count: 0,
				profile_templates_count: 0,
				tool_presets_count: 0,
			},
			safety_summary: {
				allowed_to_install: true,
				risk_level: "low",
				modifies_tool_permissions: false,
				contains_executable_code: false,
				requires_network: false,
			},
		};
	}
	if (cmd === "marketplace_list_installed_themes") {
		return [
			{
				theme_id: "coding_cyan",
				theme_name: "Coding Cyan",
				description: "Sleek high-contrast neon theme for developer focus.",
				source_pack_id: "coding_basics",
				source_pack_name: "Coding Basics Pack",
				pack_enabled: mockState.enabledPacks.includes("coding_basics"),
			},
			{
				theme_id: "calm_focus",
				theme_name: "Calm Focus",
				description: "Relaxing deep forest-green tones for study sessions.",
				source_pack_id: "study_coach",
				source_pack_name: "Study Coach Pack",
				pack_enabled: mockState.enabledPacks.includes("study_coach"),
			},
		];
	}
	if (cmd === "marketplace_save_addon_settings") {
		mockState.activeThemeId = args.settings.active_theme_id;
		mockState.activeThemeSourcePackId =
			args.settings.active_theme_source_pack_id;
		mockState.enabledPacks = args.settings.enabled_packs;
		mockState.disabledContributions = args.settings.disabled_contributions;
		return {};
	}
	if (cmd === "marketplace_toggle_contribution_enabled") {
		const key = `${args.packId}:${args.contributionType}:${args.contributionId}`;
		if (args.enabled) {
			mockState.disabledContributions = mockState.disabledContributions.filter(
				(k) => k !== key,
			);
		} else {
			if (!mockState.disabledContributions.includes(key))
				mockState.disabledContributions.push(key);
		}
		return {};
	}
	if (cmd === "marketplace_set_active_theme") {
		mockState.activeThemeId = args.themeId;
		mockState.activeThemeSourcePackId = args.sourcePackId;
		return {};
	}
	if (cmd === "marketplace_has_legacy_modes") {
		return false;
	}
	if (cmd === "preview_context_for_message") {
		const active = mockState.themes.find((theme) => theme.applied);
		return {
			profile_sent: ["identity.display_name: Developer Pro"],
			style_sent: ["communication.tone: Professional"],
			preferences_sent: [],
			contexts_sent: [],
			contexts_pinned: [],
			contexts_not_sent: [],
			warnings: [],
			final_context_text:
				"You are OpenNivara.\n\nSettings profile, style, preferences, and contexts only.",
			active_theme: active
				? { id: active.id, name: active.name, ui_only: true }
				: null,
		};
	}
	if (cmd === "marketplace_get_effective_settings_preview") {
		return {
			base_preferences: [
				{
					id: "pref_tone",
					enabled: true,
					send_policy: "always",
					description: "Respond inside simple code guidelines.",
					triggers: [],
					required_any: [],
					negative_triggers: [],
					min_score: 0,
					likes: [{ item: "conciseness", strength: 3 }],
					dislikes: [],
					notes: [],
				},
			],
			addon_preferences: [],
			base_contexts: [
				{
					id: "opennivara_context",
					enabled: true,
					kind: "identity",
					send_policy: "always",
					title: "OpenNivara Project context",
					summary: "Context about the active desktop assistant.",
					triggers: [],
					required_any: [],
					negative_triggers: [],
					min_score: 0,
					facts: [],
					rules: [],
				},
			],
			addon_contexts: [],
			addon_quick_prompts: [],
			active_theme_id: mockState.activeThemeId,
			active_theme_name:
				mockState.activeThemeId === "study_coach"
					? "Forest Green"
					: mockState.activeThemeId === "coding_basics"
						? "Coding Cyan"
						: "Default Theme",
			active_theme_source_pack_id: null,
			active_style_pack_id: null,
			active_style_pack_name: null,
			disabled_contributions: mockState.disabledContributions,
			enabled_packs: [],
		};
	}

	// Show warning badge for unhandled command
	console.warn(`[Browser Preview] Missing mock command: "${cmd}"`);

	// Update warning banner to alert about the missing command
	if (typeof document !== "undefined") {
		const banner = document.getElementById("tauri-browser-preview-banner");
		if (banner) {
			banner.style.border = "1px solid rgba(239, 68, 68, 0.5)";
			banner.style.color = "#ef4444";
			banner.innerHTML = `
				<span style="display:inline-block; width:6px; height:6px; border-radius:50%; background:#ef4444;"></span>
				Warning: Missing Command "${cmd}" in Preview!
			`;
		}
	}

	return null;
}

function browserPreviewLocationContext() {
	return {
		status: "unknown",
		latitude: null,
		longitude: null,
		accuracy_meters: null,
		source: "browser_preview",
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

function browserPreviewRuntimeContext() {
	return {
		now_utc: new Date().toISOString(),
		now_local: new Date().toISOString(),
		timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC",
		date_local: new Date().toISOString().slice(0, 10),
		day_of_week: "Tue",
		locale: null,
		calendar_week: null,
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
		location: browserPreviewLocationContext(),
		model: browserPreviewModelContextInfo(),
	};
}

function browserPreviewModelContextInfo() {
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
