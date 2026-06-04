/**
 * ORION — Tauri Bridge
 * Detects if running in Tauri and provides unified API.
 * Falls back to Electron's window.orion if not in Tauri.
 *
 * Every method the frontend calls must exist here.
 * If a Tauri Rust command isn't implemented yet, it falls back to Electron.
 */

const IS_TAURI = window.__TAURI_INTERNALS__ !== undefined;

// Tauri invoke wrapper
async function tauriInvoke(cmd, args = {}) {
    if (!IS_TAURI) return null;
    try {
        return await window.__TAURI__.core.invoke(cmd, args);
    } catch (e) {
        console.warn(`[ORION Bridge] Tauri command '${cmd}' failed:`, e);
        if (window.showError) {
            window.showError(`IPC Error: ${cmd} — ${typeof e === 'string' ? e : (e.message || 'Unknown error')}`);
        }
        return null;
    }
}

// Quick IPC health check
async function ping() {
    try {
        return await window.__TAURI__.core.invoke('get_current_session_id');
    } catch (e) {
        console.error('[ORION] IPC ping failed:', e);
        return null;
    }
}

// Fallback helper — try Electron if not in Tauri
function electronFallback(method, ...args) {
    if (window.orion && typeof window.orion[method] === 'function') {
        return window.orion[method](...args);
    }
    console.warn(`[ORION Bridge] No Electron fallback for '${method}'`);
    return null;
}

// Unified ORION API — works in both Tauri and Electron
window.orionBridge = {
    isTauri: IS_TAURI,

    // ========================================
    // WINDOW CONTROLS
    // ========================================
    minimize: async () => IS_TAURI ? tauriInvoke('minimize_window') : electronFallback('minimize'),
    maximize: async () => IS_TAURI ? tauriInvoke('maximize_window') : electronFallback('maximize'),
    close: async () => IS_TAURI ? tauriInvoke('close_window') : electronFallback('close'),

    // ========================================
    // CHAT
    // ========================================
    _onChunkCallback: null,

    chat: async (message, sessionId) => {
        if (IS_TAURI) {
            const text = await tauriInvoke('chat', { message, session_id: sessionId });
            if (text === null) {
                return { response: '', error: 'IPC call failed — check console' };
            }
            // Wrap raw string in object shape frontend expects
            return { response: text || '' };
        }
        return electronFallback('chat', message, sessionId);
    },

    onChunk: async (callback) => {
        if (IS_TAURI) {
            const bridge = window.orionBridge;
            bridge._onChunkCallback = callback;
            if (window.__TAURI__ && window.__TAURI__.event) {
                if (bridge._chunkUnlisten) bridge._chunkUnlisten();
                if (bridge._errorUnlisten) bridge._errorUnlisten();
                if (bridge._doneUnlisten) bridge._doneUnlisten();

                bridge._chunkUnlisten = await window.__TAURI__.event.listen('chat_chunk', (event) => {
                    if (bridge._onChunkCallback && event.payload && event.payload.chunk) {
                        bridge._onChunkCallback(event.payload.chunk);
                    }
                });

                bridge._errorUnlisten = await window.__TAURI__.event.listen('chat_error', (event) => {
                    console.error('[ORION] Chat error:', event.payload?.error);
                    if (bridge._onChunkCallback) bridge._onChunkCallback(`\n[Error: ${event.payload?.error || 'Unknown'}]`);
                });

                bridge._doneUnlisten = await window.__TAURI__.event.listen('chat_done', () => {
                    // Streaming completed — frontend can trigger final render
                });
            }
            return;
        }
        return electronFallback('onChunk', callback);
    },

    // ========================================
    // SESSIONS
    // ========================================
    getSessions: async () => IS_TAURI ? tauriInvoke('get_sessions') : electronFallback('getSessions'),
    getCurrentSessionId: async () => IS_TAURI ? tauriInvoke('get_current_session_id') : electronFallback('getCurrentSessionId'),
    createSession: async (name, projectId) => IS_TAURI ? tauriInvoke('create_session', { name, project_id: projectId }) : electronFallback('createSession', name, projectId),
    switchSession: async (sessionId) => IS_TAURI ? tauriInvoke('switch_session', { session_id: sessionId }) : electronFallback('switchSession', sessionId),
    renameSession: async (sessionId, newTitle) => IS_TAURI ? tauriInvoke('rename_session', { session_id: sessionId, new_title: newTitle }) : electronFallback('renameSession', sessionId, newTitle),
    deleteSession: async (sessionId) => IS_TAURI ? tauriInvoke('delete_session', { session_id: sessionId }) : electronFallback('deleteSession', sessionId),
    getHistory: async (sessionId) => IS_TAURI ? tauriInvoke('get_history', { session_id: sessionId }) : electronFallback('getHistory', sessionId),
    clearHistory: async (sessionId) => IS_TAURI ? tauriInvoke('clear_history') : electronFallback('clearHistory', sessionId),

    // ========================================
    // SETTINGS & STATS
    // ========================================
    getSettings: async () => IS_TAURI ? tauriInvoke('get_settings') : electronFallback('getSettings'),
    getStats: async () => IS_TAURI ? tauriInvoke('get_stats') : electronFallback('getStats'),
    getMode: async () => IS_TAURI ? tauriInvoke('get_mode') : electronFallback('getMode'),
    setMode: async (mode) => IS_TAURI ? tauriInvoke('set_mode', { mode_name: mode }) : electronFallback('setMode', mode),

    // ========================================
    // GOALS
    // ========================================
    getGoals: async () => IS_TAURI ? tauriInvoke('get_goals') : electronFallback('getGoals'),
    getGoalStats: async () => IS_TAURI ? tauriInvoke('get_goal_stats') : electronFallback('getGoalStats'),
    createGoal: async (title, description, deadline) => IS_TAURI ? tauriInvoke('create_goal', { title: title, description: description || '', priority: 'medium', category: 'general' }) : electronFallback('createGoal', title, description, deadline),
    completeGoal: async (id) => IS_TAURI ? tauriInvoke('complete_goal', { id }) : electronFallback('completeGoal', id),
    deleteGoal: async (id) => IS_TAURI ? tauriInvoke('delete_goal', { id }) : electronFallback('deleteGoal', id),

    // ========================================
    // PROJECTS
    // ========================================
    getProjects: async () => IS_TAURI ? tauriInvoke('get_projects') : electronFallback('getProjects'),
    createProject: async (name, description) => IS_TAURI ? tauriInvoke('create_project', { name, description }) : electronFallback('createProject', name, description),
    updateProject: async (id, name, description) => IS_TAURI ? tauriInvoke('update_project', { id, name, description }) : electronFallback('updateProject', id, name, description),
    deleteProject: async (id) => IS_TAURI ? tauriInvoke('delete_project', { id }) : electronFallback('deleteProject', id),

    // ========================================
    // CLIENTS
    // ========================================
    listClients: async () => IS_TAURI ? tauriInvoke('list_clients') : electronFallback('listClients'),
    getClient: async (id) => IS_TAURI ? tauriInvoke('get_client', { id }) : electronFallback('getClient', id),
    createClient: async (data) => IS_TAURI ? tauriInvoke('create_client', { data }) : electronFallback('createClient', data),
    updateClient: async (id, data) => IS_TAURI ? tauriInvoke('update_client', { id, data }) : electronFallback('updateClient', id, data),
    deleteClient: async (id) => IS_TAURI ? tauriInvoke('delete_client', { id }) : electronFallback('deleteClient', id),

    // ========================================
    // LEADS
    // ========================================
    listLeads: async () => IS_TAURI ? tauriInvoke('list_leads') : electronFallback('listLeads'),
    getLead: async (id) => IS_TAURI ? tauriInvoke('get_lead', { id }) : electronFallback('getLead', id),
    createLead: async (data) => IS_TAURI ? tauriInvoke('create_lead', { data }) : electronFallback('createLead', data),
    updateLead: async (id, data) => IS_TAURI ? tauriInvoke('update_lead', { id, data }) : electronFallback('updateLead', id, data),
    deleteLead: async (id) => IS_TAURI ? tauriInvoke('delete_lead', { id }) : electronFallback('deleteLead', id),

    // ========================================
    // AI / AGENTS
    // ========================================
    getAgents: async () => IS_TAURI ? tauriInvoke('get_agents') : electronFallback('getAgents'),
    getOllama: async () => IS_TAURI ? tauriInvoke('get_ollama') : electronFallback('getOllama'),
    getSuggestions: async () => IS_TAURI ? tauriInvoke('get_suggestions') : electronFallback('getSuggestions'),
    getArgumentStats: async () => IS_TAURI ? tauriInvoke('get_argument_stats') : electronFallback('getArgumentStats'),

    // ========================================
    // MEMORY
    // ========================================
    getMemoryContext: async () => IS_TAURI ? tauriInvoke('get_memory_context') : electronFallback('getMemoryContext'),

    // ========================================
    // USER PROFILE
    // ========================================
    getUserInfo: async () => IS_TAURI ? tauriInvoke('get_user_info') : electronFallback('getUserInfo'),
    setPreference: async (key, value) => IS_TAURI ? tauriInvoke('set_preference', { key, value }) : electronFallback('setPreference', key, value),
    getFacts: async (category) => IS_TAURI ? tauriInvoke('get_facts', { category: category || null }) : electronFallback('getFacts', category),
    saveFact: async (key, value, category) => IS_TAURI ? tauriInvoke('save_fact', { key, value, category: category || null }) : electronFallback('saveFact', key, value, category),
    searchHistory: async (query, limit) => IS_TAURI ? tauriInvoke('search_history', { query, limit: limit || null }) : electronFallback('searchHistory', query, limit),

    // ========================================
    // FOLLOW-UPS
    // ========================================
    getFollowUps: async () => IS_TAURI ? tauriInvoke('get_follow_ups') : electronFallback('getFollowUps'),
    addFollowUp: async (context, remindMinutes) => IS_TAURI ? tauriInvoke('add_follow_up', { context, remind_minutes: remindMinutes }) : electronFallback('addFollowUp', context, remindMinutes),
    dismissFollowUp: async (id) => IS_TAURI ? tauriInvoke('dismiss_follow_up', { id }) : electronFallback('dismissFollowUp', id),

    // ========================================
    // TASK TRACKER
    // ========================================
    setActiveTask: async (task) => IS_TAURI ? tauriInvoke('set_active_task', { task }) : electronFallback('setActiveTask', task),
    getActiveTask: async () => IS_TAURI ? tauriInvoke('get_active_task') : electronFallback('getActiveTask'),
    clearActiveTask: async () => IS_TAURI ? tauriInvoke('clear_active_task') : electronFallback('clearActiveTask'),

    // ========================================
    // OLLAMA
    // ========================================
    checkOllamaHealth: async () => IS_TAURI ? tauriInvoke('check_ollama_health') : electronFallback('checkOllamaHealth'),

    // ========================================
    // VOICE
    // ========================================
    toggleVoiceInput: async () => IS_TAURI ? tauriInvoke('toggle_voice_input') : electronFallback('toggleVoiceInput'),
    toggleVoiceOutput: async () => IS_TAURI ? tauriInvoke('toggle_voice_output') : electronFallback('toggleVoiceOutput'),

    // ========================================
    // FOLDERS
    // ========================================
    getFolders: async () => IS_TAURI ? tauriInvoke('get_folders') : electronFallback('getFolders'),
    selectFolder: async () => IS_TAURI ? tauriInvoke('select_folder') : electronFallback('selectFolder'),
    addFolder: async (path, name) => IS_TAURI ? tauriInvoke('add_folder', { path, name }) : electronFallback('addFolder', path, name),
    setActiveFolder: async (id) => IS_TAURI ? tauriInvoke('set_active_folder', { id }) : electronFallback('setActiveFolder', id),
    getActiveFolder: async () => IS_TAURI ? tauriInvoke('get_active_folder') : electronFallback('getActiveFolder'),
    sendFolderToScrap: async (id) => IS_TAURI ? tauriInvoke('send_folder_to_scrap', { id }) : electronFallback('sendFolderToScrap', id),
    getForgottenFolders: async () => IS_TAURI ? tauriInvoke('get_forgotten_folders') : electronFallback('getForgottenFolders'),
};

console.log(`[ORION Bridge] Running in ${IS_TAURI ? 'TAURI' : 'ELECTRON'} mode — ${Object.keys(window.orionBridge).length - 1} methods loaded`);
