import * as tauri from "./tauriClient";

export const askOpenNivara = tauri.tauriAskOpenNivara;
export const listSessions = tauri.tauriListSessions;
export const getSessionMessages = tauri.tauriGetSessionMessages;
export const listTools = tauri.tauriListTools;
export const getToolsPath = tauri.tauriGetToolsPath;
export const getProfilePath = tauri.tauriGetProfilePath;
export const getPreferencesPath = tauri.tauriGetPreferencesPath;
export const getStylePath = tauri.tauriGetStylePath;
export const getContextsPath = tauri.tauriGetContextsPath;
export const getTelegramPath = tauri.tauriGetTelegramPath;
export const getMapSummary = tauri.tauriMapSummary;

export const getProfile = tauri.tauriGetProfile;
export const saveProfile = tauri.tauriSaveProfile;
export const getStyle = tauri.tauriGetStyle;
export const saveStyle = tauri.tauriSaveStyle;
export const getPreferences = tauri.tauriGetPreferences;
export const savePreferences = tauri.tauriSavePreferences;
export const getContexts = tauri.tauriGetContexts;
export const saveContexts = tauri.tauriSaveContexts;
export const previewContextForMessage = tauri.tauriPreviewContextForMessage;
export const pinContext = tauri.tauriPinContext;
export const unpinContext = tauri.tauriUnpinContext;
export const checkApiKey = tauri.tauriCheckApiKey;

export type {
	ContextEntry,
	ContextPreview,
	Contexts,
	PreferenceSection,
	Preferences,
	Profile,
	Style,
} from "./tauriClient";
