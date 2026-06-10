// ORION 3.0 — New UI JavaScript
// Sidebar toggle, divider drag, project-first workflow

let currentViewName = 'dashboard';
let currentProjectId = null;

// Selection tracking for keyboard shortcuts
let selectedProjectId = null;
let selectedSessionId = null;
let lastCtrlDTime = 0;
const CTRL_D_DOUBLE_MS = 800; // Double-tap window for Ctrl+D

// Modal functions are defined in renderer.js (loaded before this file)

// ========================================
// INITIALIZATION
// ========================================

document.addEventListener('DOMContentLoaded', () => {
    console.log('ORION 3.0 UI initialized');

    setupSidebarToggle();
    setupRightPanelToggle();
    setupDividerDrag();
    setupChatDividerDrag();
    setupNavHandlers();
    setupKeyboardShortcuts();

    // Show dashboard on load (plain start — no auto-loading last session)
    switchView('dashboard');

    // Load sidebar data after renderer initializes
    setTimeout(() => {
        loadDashboardStats();
        loadRightSidebar();
    }, 500);

    // Refresh right sidebar periodically
    setInterval(loadRightSidebar, 30000);
});

// ========================================
// SIDEBAR TOGGLE
// ========================================

function setupSidebarToggle() {
    const toggle = document.getElementById('sidebar-toggle');
    const sidebar = document.getElementById('sidebar-left');

    if (toggle && sidebar) {
        toggle.addEventListener('click', () => {
            sidebar.classList.toggle('collapsed');
            sidebar.classList.toggle('expanded');
        });
    }
}

// ========================================
// RIGHT PANEL TOGGLE
// ========================================

function setupRightPanelToggle() {
    const toggle = document.getElementById('right-toggle');
    const panel = document.getElementById('sidebar-right');
    const divider = document.getElementById('divider-right');

    if (toggle && panel) {
        toggle.addEventListener('click', () => {
            toggleRightPanel();
        });
    }
}

// Global function for top-bar toggle button
function toggleRightPanel() {
    const panel = document.getElementById('sidebar-right');
    const divider = document.getElementById('divider-right');

    if (panel) {
        const isCollapsed = panel.classList.contains('collapsed');
        if (isCollapsed) {
            panel.classList.remove('collapsed');
            if (divider) divider.classList.remove('hidden');
        } else {
            panel.classList.add('collapsed');
            if (divider) divider.classList.add('hidden');
        }
    }
}

// ========================================
// DIVIDER DRAG (Left & Right horizontal)
// ========================================

function setupDividerDrag() {
    const leftDivider = document.getElementById('divider-left');
    const rightDivider = document.getElementById('divider-right');
    const sidebar = document.getElementById('sidebar-left');
    const rightPanel = document.getElementById('sidebar-right');

    if (leftDivider && sidebar) {
        setupHorizontalDrag(leftDivider, (delta) => {
            const currentWidth = sidebar.offsetWidth;
            const newWidth = Math.max(52, Math.min(250, currentWidth + delta));
            sidebar.style.width = newWidth + 'px';
            sidebar.style.minWidth = newWidth + 'px';

            // Toggle collapsed/expanded class based on width
            if (newWidth > 80) {
                sidebar.classList.remove('collapsed');
                sidebar.classList.add('expanded');
            } else {
                sidebar.classList.remove('expanded');
                sidebar.classList.add('collapsed');
            }
        });
    }

    if (rightDivider && rightPanel) {
        setupHorizontalDrag(rightDivider, (delta) => {
            const currentWidth = rightPanel.offsetWidth;
            const newWidth = Math.max(0, Math.min(350, currentWidth - delta));
            rightPanel.style.width = newWidth + 'px';
            rightPanel.style.minWidth = newWidth + 'px';

            if (newWidth < 20) {
                rightPanel.classList.add('collapsed');
                rightDivider.classList.add('hidden');
            }
        }, true); // reverse = true for right divider
    }
}

function setupHorizontalDrag(element, onDrag, reverse = false) {
    let startX = 0;
    let isDragging = false;

    const onMouseDown = (e) => {
        e.preventDefault();
        isDragging = true;
        startX = e.clientX;
        element.classList.add('dragging');
        document.body.style.cursor = 'col-resize';
        document.body.classList.add('resizing');

        document.addEventListener('mousemove', onMouseMove);
        document.addEventListener('mouseup', onMouseUp);
    };

    const onMouseMove = (e) => {
        if (!isDragging) return;
        const delta = reverse ? startX - e.clientX : e.clientX - startX;
        startX = e.clientX;
        onDrag(delta);
    };

    const onMouseUp = () => {
        isDragging = false;
        element.classList.remove('dragging');
        document.body.style.cursor = '';
        document.body.classList.remove('resizing');
        document.removeEventListener('mousemove', onMouseMove);
        document.removeEventListener('mouseup', onMouseUp);
    };

    element.addEventListener('mousedown', onMouseDown);
}

// ========================================
// CHAT DIVIDER DRAG (Vertical)
// ========================================

function setupChatDividerDrag() {
    const divider = document.getElementById('chat-divider');
    const chatSection = document.getElementById('chat-section');
    const viewContainer = document.querySelector('.view-container');

    if (!divider || !chatSection || !viewContainer) return;

    let startY = 0;
    let isDragging = false;

    divider.addEventListener('mousedown', (e) => {
        e.preventDefault();
        isDragging = true;
        startY = e.clientY;
        document.body.style.cursor = 'row-resize';
        document.body.classList.add('resizing');

        document.addEventListener('mousemove', onMouseMove);
        document.addEventListener('mouseup', onMouseUp);
    });

    const onMouseMove = (e) => {
        if (!isDragging) return;
        const delta = startY - e.clientY;
        startY = e.clientY;

        const currentHeight = chatSection.offsetHeight;
        const newHeight = Math.max(120, Math.min(window.innerHeight * 0.7, currentHeight + delta));

        chatSection.style.height = newHeight + 'px';
        chatSection.style.minHeight = newHeight + 'px';
        chatSection.style.maxHeight = newHeight + 'px';
    };

    const onMouseUp = () => {
        isDragging = false;
        document.body.style.cursor = '';
        document.body.classList.remove('resizing');
        document.removeEventListener('mousemove', onMouseMove);
        document.removeEventListener('mouseup', onMouseUp);
    };
}

// ========================================
// NAV HANDLERS
// ========================================

function setupNavHandlers() {
    document.querySelectorAll('.nav-item[data-view]').forEach(item => {
        item.addEventListener('click', () => {
            const view = item.dataset.view;
            if (view) switchView(view);
        });
    });
}

// ========================================
// KEYBOARD SHORTCUTS
// ========================================

function setupKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
        // Don't handle shortcuts when typing in input fields
        const tag = e.target.tagName;
        if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

        // Ctrl+R — Rename selected item
        if (e.ctrlKey && e.key === 'r') {
            e.preventDefault();
            handleRename();
            return;
        }

        // Ctrl+D — Delete (double-tap)
        if (e.ctrlKey && e.key === 'd') {
            e.preventDefault();
            handleDeleteDoubleTap();
            return;
        }
    });
}

function handleRename() {
    if (currentViewName === 'projects' && selectedProjectId) {
        renameProject(selectedProjectId);
    } else if ((currentViewName === 'project-detail') && selectedSessionId) {
        renameSessionFromView(selectedSessionId);
    }
}

function handleDeleteDoubleTap() {
    const now = Date.now();
    if (now - lastCtrlDTime < CTRL_D_DOUBLE_MS) {
        // Double-tap confirmed — delete
        lastCtrlDTime = 0;
        if (currentViewName === 'projects' && selectedProjectId) {
            deleteProject(selectedProjectId);
        } else if (currentViewName === 'project-detail' && selectedSessionId) {
            deleteSessionFromView(selectedSessionId);
        }
    } else {
        // First tap — show warning
        lastCtrlDTime = now;
        if (typeof logSystem === 'function') {
            logSystem('Press Ctrl+D again to confirm delete');
        }
    }
}

function selectProject(projectId, element) {
    selectedProjectId = projectId;
    selectedSessionId = null;
    // Update visual selection
    document.querySelectorAll('.project-card').forEach(c => c.classList.remove('selected'));
    if (element) element.classList.add('selected');
}

function selectSession(sessionId, element) {
    selectedSessionId = sessionId;
    selectedProjectId = null;
    // Update visual selection
    document.querySelectorAll('.session-card').forEach(c => c.classList.remove('selected'));
    if (element) element.classList.add('selected');
}

async function renameProject(projectId) {
    const projects = await window.orionBridge.getProjects();
    const project = projects.find(p => p.id === projectId);
    if (!project) return;
    const newName = await orionPrompt('Rename Project', 'Enter new name:', project.name);
    if (!newName?.trim() || newName.trim() === project.name) return;
    await window.orionBridge.updateProject(projectId, newName.trim(), project.description || '');
    loadProjectsView();
    if (typeof logSystem === 'function') logSystem(`Project renamed to "${newName.trim()}"`);
}

async function deleteProject(projectId) {
    const projects = await window.orionBridge.getProjects();
    const project = projects.find(p => p.id === projectId);
    if (!project) return;
    if (project.id === 1) {
        await orionAlert('ORION', 'Cannot delete the default General project');
        return;
    }
    if (!await orionConfirm('Delete Project', `Delete project "${project.name}"? All its chats will be moved to General.`)) return;
    await window.orionBridge.deleteProject(projectId);
    selectedProjectId = null;
    loadProjectsView();
    if (typeof logSystem === 'function') logSystem(`Project "${project.name}" deleted`);
}

// ========================================
// VIEW SWITCHING
// ========================================

function switchView(viewName) {
    if (!viewName || viewName === currentViewName) return;
    console.log('Switching to view:', viewName);

    // Update nav items
    document.querySelectorAll('.nav-item').forEach(item => {
        item.classList.remove('active');
        if (item.dataset.view === viewName) {
            item.classList.add('active');
        }
    });

    // Show/hide views
    document.querySelectorAll('.view').forEach(view => {
        view.classList.remove('active');
    });

    const targetView = document.getElementById('view-' + viewName);
    if (targetView) {
        targetView.classList.add('active');
    }

    currentViewName = viewName;
    selectedProjectId = null;
    selectedSessionId = null;

    // Load view-specific data
    switch (viewName) {
        case 'dashboard': loadDashboardStats(); break;
        case 'projects': loadProjectsView(); break;
        case 'systems': loadSystemsView(); break;
        case 'tasks': loadTasksView(); break;
        case 'analytics': loadAnalyticsView(); break;
        case 'settings': loadSettingsView(); break;
        case 'clients': loadClientsView(); break;
        case 'leads': loadLeadsView(); break;
        case 'calendar': loadCalendarView(); break;
    }

    if (typeof logSystem === 'function') {
        logSystem(`Switched to ${viewName}`);
    }
}

// ========================================
// DASHBOARD
// ========================================

async function loadDashboardStats() {
    if (!window.orionBridge) return;
    try {
        const stats = await window.orionBridge.getStats();
        const goals = await window.orionBridge.getGoals();
        const sessions = await window.orionBridge.getSessions();

        const elSessions = document.getElementById('dash-sessions');
        const elMessages = document.getElementById('dash-messages');
        const elGoals = document.getElementById('dash-goals');

        if (elSessions) elSessions.textContent = stats.totalSessions || sessions.length || 0;
        if (elMessages) elMessages.textContent = stats.totalMessages || 0;
        if (elGoals) elGoals.textContent = (goals && goals.length) || 0;

        // Load recent sessions
        const recentList = document.getElementById('recent-sessions-list');
        if (recentList && sessions.length > 0) {
            recentList.innerHTML = '';
            const recent = sessions.slice(0, 5);
            recent.forEach(session => {
                const item = document.createElement('div');
                item.className = 'recent-session-item';
                item.onclick = () => openSession(session.id);
                const time = session.updated_at
                    ? new Date(session.updated_at).toLocaleDateString()
                    : '';
                const titleSpan = document.createElement('span');
                titleSpan.className = 'recent-session-title';
                titleSpan.textContent = session.title || 'New Chat';
                const timeSpan = document.createElement('span');
                timeSpan.className = 'recent-session-time';
                timeSpan.textContent = time;
                item.appendChild(titleSpan);
                item.appendChild(timeSpan);
                recentList.appendChild(item);
            });
        } else if (recentList) {
            recentList.innerHTML = '<div class="empty-view">No chats yet</div>';
        }
    } catch (e) {
        console.error('Dashboard stats error:', e);
    }
}

// ========================================
// PROJECTS VIEW
// ========================================

async function loadProjectsView() {
    if (!window.orionBridge) return;
    const container = document.getElementById('projects-container');
    if (!container) return;

    try {
        const projects = await window.orionBridge.getProjects();

        if (!projects || projects.length === 0) {
            container.innerHTML = '<div class="empty-view">No projects yet. Click "+ New Project" to create one.</div>';
            return;
        }

        container.innerHTML = '';

        const allSessions = await window.orionBridge.getSessions();

        for (const project of projects) {
            const sessions = allSessions.filter(s => s.project_id === project.id);

            const card = document.createElement('div');
            card.className = 'project-card';
            card.onclick = (e) => {
                selectProject(project.id, card);
                openProject(project.id, project.name);
            };
            const header = document.createElement('div');
            header.className = 'project-card-header';
            const nameSpan = document.createElement('span');
            nameSpan.className = 'project-card-name';
            nameSpan.textContent = project.name;
            const countSpan = document.createElement('span');
            countSpan.className = 'project-card-count';
            countSpan.textContent = sessions.length + ' chats';
            header.appendChild(nameSpan);
            header.appendChild(countSpan);
            card.appendChild(header);
            if (project.description) {
                const descDiv = document.createElement('div');
                descDiv.className = 'project-card-desc';
                descDiv.textContent = project.description;
                card.appendChild(descDiv);
            }
            const metaDiv = document.createElement('div');
            metaDiv.className = 'project-card-meta';
            metaDiv.textContent = 'Created ' + new Date(project.created_at).toLocaleDateString();
            card.appendChild(metaDiv);
            container.appendChild(card);
        }
    } catch (e) {
        console.error('Projects view error:', e);
        container.innerHTML = '<div class="empty-view">Error loading projects</div>';
    }
}

async function openProject(projectId, projectName) {
    currentProjectId = projectId;

    // Update detail view title
    const titleEl = document.getElementById('project-detail-title');
    if (titleEl) titleEl.textContent = projectName;

    // Switch to project detail view
    document.querySelectorAll('.view').forEach(v => v.classList.remove('active'));
    const detailView = document.getElementById('view-project-detail');
    if (detailView) detailView.classList.add('active');
    currentViewName = 'project-detail';

    // Load sessions for this project
    await loadProjectSessions(projectId);
}

async function loadProjectSessions(projectId) {
    const container = document.getElementById('project-sessions-list');
    if (!container) return;

    try {
        // Get all sessions and filter by project_id
        const allSessions = await window.orionBridge.getSessions();
        const sessions = allSessions.filter(s => s.project_id === projectId);

        if (!sessions || sessions.length === 0) {
            container.innerHTML = '<div class="empty-view">No chats in this project yet. Click "+ New Chat" to start one.</div>';
            return;
        }

        container.innerHTML = '';

        sessions.forEach(session => {
            const card = document.createElement('div');
            card.className = 'session-card';
            card.onclick = (e) => {
                selectSession(session.id, card);
                openSession(session.id);
            };

            const date = session.updated_at
                ? new Date(session.updated_at).toLocaleDateString()
                : '';

            const info = document.createElement('div');
            info.className = 'session-card-info';
            const titleSpan = document.createElement('span');
            titleSpan.className = 'session-card-title';
            titleSpan.textContent = session.title || 'New Chat';
            const metaSpan = document.createElement('span');
            metaSpan.className = 'session-card-meta';
            metaSpan.textContent = date;
            info.appendChild(titleSpan);
            info.appendChild(metaSpan);
            card.appendChild(info);

            const actions = document.createElement('div');
            actions.className = 'session-card-actions';
            const renameBtn = document.createElement('button');
            renameBtn.textContent = '✏️';
            renameBtn.title = 'Rename';
            renameBtn.onclick = (e) => { e.stopPropagation(); renameSessionFromView(session.id); };
            const delBtn = document.createElement('button');
            delBtn.className = 'delete-btn';
            delBtn.textContent = '🗑️';
            delBtn.title = 'Delete';
            delBtn.onclick = (e) => { e.stopPropagation(); deleteSessionFromView(session.id); };
            actions.appendChild(renameBtn);
            actions.appendChild(delBtn);
            card.appendChild(actions);
            container.appendChild(card);
        });
    } catch (e) {
        console.error('Project sessions error:', e);
        container.innerHTML = '<div class="empty-view">Error loading sessions</div>';
    }
}

async function openSession(sessionId) {
    if (window.switchSession) {
        await window.switchSession(sessionId);
    } else if (window.orionBridge) {
        await window.orionBridge.switchSession(sessionId);
        document.getElementById('messages').innerHTML = '';
        if (window.loadHistory) await window.loadHistory();
    }
}

async function renameSessionFromView(sessionId) {
    const newTitle = await orionPrompt('Rename Chat', 'Enter new name:');
    if (newTitle?.trim()) {
        await window.orionBridge.renameSession(sessionId, newTitle.trim());
        if (currentProjectId) await loadProjectSessions(currentProjectId);
    }
}

async function deleteSessionFromView(sessionId) {
    if (!await orionConfirm('Delete Chat', 'Delete this chat permanently?')) return;
    await window.orionBridge.deleteSession(sessionId);
    if (currentProjectId) await loadProjectSessions(currentProjectId);
}

async function createChatInCurrentProject() {
    if (!currentProjectId) return;
    const sessionId = await window.orionBridge.createSession('New Chat', currentProjectId);
    await window.orionBridge.switchSession(sessionId);
    document.getElementById('messages').innerHTML = '';
    addMessage('orion', "New session started, Abhi. What's the goal?");
    // Refresh the sessions list so the new session appears
    await loadProjectSessions(currentProjectId);
    if (typeof logSystem === 'function') logSystem('New chat created in project');
}

async function showCreateProject() {
    const name = await orionPrompt('New Project', 'Project name:');
    if (!name?.trim()) return;
    const description = await orionPrompt('New Project', 'Project description (optional):') || '';
    await window.orionBridge.createProject(name.trim(), description);
    await loadProjectsView();
    if (typeof logSystem === 'function') logSystem(`Project "${name}" created`);
}

// ========================================
// VIEW DATA LOADING
// ========================================

async function loadTasksView() {
    if (!window.orionBridge) return;
    try {
        const goals = await window.orionBridge.getGoals();
        const container = document.getElementById('tasks-list');
        if (!container) return;

        if (!goals || goals.length === 0) {
            container.innerHTML = '<div class="empty-view">No tasks yet. Click "+ New Task" to create one.</div>';
            return;
        }

        container.innerHTML = '';
        goals.forEach(goal => {
            const card = document.createElement('div');
            card.className = 'task-card';
            const header = document.createElement('div');
            header.className = 'task-header';
            const titleSpan = document.createElement('span');
            titleSpan.className = 'task-title';
            titleSpan.textContent = goal.title;
            const statusSpan = document.createElement('span');
            statusSpan.className = 'task-status ' + goal.status;
            statusSpan.textContent = goal.status;
            header.appendChild(titleSpan);
            header.appendChild(statusSpan);
            card.appendChild(header);
            if (goal.deadline) {
                const deadlineDiv = document.createElement('div');
                deadlineDiv.className = 'task-deadline';
                deadlineDiv.textContent = 'Due: ' + new Date(goal.deadline).toLocaleString();
                card.appendChild(deadlineDiv);
            }
            const actions = document.createElement('div');
            actions.className = 'task-actions';
            const completeBtn = document.createElement('button');
            completeBtn.className = 'task-btn complete';
            completeBtn.textContent = '✓ Complete';
            completeBtn.onclick = () => window.orionBridge.completeGoal(goal.id).then(() => loadTasksView());
            const delBtn = document.createElement('button');
            delBtn.className = 'task-btn delete';
            delBtn.textContent = '✕ Delete';
            delBtn.onclick = () => window.orionBridge.deleteGoal(goal.id).then(() => loadTasksView());
            actions.appendChild(completeBtn);
            actions.appendChild(delBtn);
            card.appendChild(actions);
            container.appendChild(card);
        });
    } catch (e) {
        console.error('Tasks view error:', e);
    }
}

async function loadSystemsView() {
    if (!window.orionBridge) return;
    try {
        const agents = await window.orionBridge.getAgents();
        const ollama = await window.orionBridge.getOllama();
        const stats = await window.orionBridge.getStats();
        const modeInfo = await window.orionBridge.getMode();

        const elBrain = document.getElementById('sys-current-brain');
        if (elBrain) elBrain.textContent = 'Brain: ' + (modeInfo?.display || 'N/A');

        const elOllamaStatus = document.getElementById('sys-ollama-status');
        const elOllamaModels = document.getElementById('sys-ollama-models');
        if (elOllamaStatus) {
            if (ollama && ollama.available) {
                elOllamaStatus.textContent = '● CONNECTED';
                elOllamaStatus.className = 'sys-status status-ok';
            } else {
                elOllamaStatus.textContent = '● OFFLINE';
                elOllamaStatus.className = 'sys-status';
            }
        }
        if (elOllamaModels) elOllamaModels.textContent = 'Models: ' + (ollama?.availableModels?.length || 0);

        const elMemory = document.getElementById('sys-memory-stats');
        if (elMemory) elMemory.textContent = `Messages: ${stats.totalMessages || 0}, Sessions: ${stats.totalSessions || 0}`;

        const elAgents = document.getElementById('sys-agents-list');
        if (elAgents) {
            if (agents && agents.agents) {
                elAgents.textContent = agents.agents.map(a => a.name).join(', ');
            } else {
                elAgents.textContent = 'Coder, Business, Scheduler';
            }
        }
    } catch (e) {
        console.error('Systems view error:', e);
    }
}

async function loadAnalyticsView() {
    if (!window.orionBridge) return;
    try {
        const stats = await window.orionBridge.getStats();
        const goalStats = await window.orionBridge.getGoalStats();

        const sets = document.getElementById('ana-total-sessions');
        const msgs = document.getElementById('ana-total-messages');
        const act = document.getElementById('ana-active-goals');
        const comp = document.getElementById('ana-completed-goals');

        if (sets) sets.textContent = stats.totalSessions || 0;
        if (msgs) msgs.textContent = stats.totalMessages || 0;
        if (act) act.textContent = goalStats?.active || 0;
        if (comp) comp.textContent = goalStats?.completed || 0;
    } catch (e) {
        console.error('Analytics view error:', e);
    }
}

async function loadSettingsView() {
    if (!window.orionBridge) return;
    try {
        const modeInfo = await window.orionBridge.getMode();
        const ollama = await window.orionBridge.getOllama();

        const elMode = document.getElementById('setting-current-mode');
        if (elMode) elMode.textContent = modeInfo?.display || 'ORION';

        const elOllama = document.getElementById('setting-ollama-status');
        if (elOllama) {
            if (ollama && ollama.available) {
                elOllama.textContent = 'CONNECTED';
                elOllama.className = 'status-ok';
            } else {
                elOllama.textContent = 'OFFLINE';
                elOllama.className = 'status-offline';
            }
        }
    } catch (e) {
        console.error('Settings view error:', e);
    }
}

// ========================================
// CLIENTS VIEW
// ========================================

async function loadClientsView() {
    const container = document.getElementById('clients-list');
    if (!container) return;

    try {
        const clients = await window.orionBridge.listClients();
        if (!clients || clients.length === 0) {
            container.innerHTML = '<div class="empty-view">No clients yet. Click "+ Add Client" to add one.</div>';
            return;
        }

        container.innerHTML = '';
        clients.forEach(client => {
            const card = document.createElement('div');
            card.className = 'client-card';

            const header = document.createElement('div');
            header.className = 'client-header';
            const avatar = document.createElement('div');
            avatar.className = 'client-avatar';
            avatar.textContent = (client.name || 'C')[0].toUpperCase();
            const info = document.createElement('div');
            info.className = 'client-info';
            const nameDiv = document.createElement('div');
            nameDiv.className = 'client-name';
            nameDiv.textContent = client.name;
            const emailDiv = document.createElement('div');
            emailDiv.className = 'client-email';
            emailDiv.textContent = client.email || 'No email';
            info.appendChild(nameDiv);
            info.appendChild(emailDiv);
            const statusSpan = document.createElement('span');
            statusSpan.className = 'client-status ' + (client.status || 'active');
            statusSpan.textContent = client.status || 'active';
            header.appendChild(avatar);
            header.appendChild(info);
            header.appendChild(statusSpan);
            card.appendChild(header);

            if (client.phone || client.notes) {
                const meta = document.createElement('div');
                meta.className = 'client-meta';
                if (client.phone) {
                    const phoneSpan = document.createElement('span');
                    phoneSpan.textContent = '📱 ' + client.phone;
                    meta.appendChild(phoneSpan);
                }
                if (client.notes) {
                    const notesSpan = document.createElement('span');
                    notesSpan.textContent = '📝 ' + client.notes.substring(0, 50) + (client.notes.length > 50 ? '...' : '');
                    meta.appendChild(notesSpan);
                }
                card.appendChild(meta);
            }

            const actions = document.createElement('div');
            actions.className = 'client-actions';
            const editBtn = document.createElement('button');
            editBtn.className = 'task-btn';
            editBtn.textContent = '✎ Edit';
            editBtn.onclick = () => editClient(client.id);
            const delBtn = document.createElement('button');
            delBtn.className = 'task-btn delete';
            delBtn.textContent = '✕ Delete';
            delBtn.onclick = () => deleteClient(client.id);
            actions.appendChild(editBtn);
            actions.appendChild(delBtn);
            card.appendChild(actions);

            container.appendChild(card);
        });
    } catch (e) {
        console.error('Clients view error:', e);
        container.innerHTML = '<div class="empty-view">Error loading clients</div>';
    }
}

function openClientModal(clientId = null) {
    const modal = document.getElementById('client-modal');
    const title = document.getElementById('client-modal-title');
    const idInput = document.getElementById('client-id');

    idInput.value = clientId || '';
    title.textContent = clientId ? 'Edit Client' : 'Add Client';

    if (clientId) {
        window.orionBridge.getClient(clientId).then(client => {
            if (client) {
                document.getElementById('client-name').value = client.name || '';
                document.getElementById('client-email').value = client.email || '';
                document.getElementById('client-phone').value = client.phone || '';
                document.getElementById('client-notes').value = client.notes || '';
            }
        });
    } else {
        document.getElementById('client-name').value = '';
        document.getElementById('client-email').value = '';
        document.getElementById('client-phone').value = '';
        document.getElementById('client-notes').value = '';
    }

    modal.classList.remove('hidden');
}

function closeClientModal() {
    document.getElementById('client-modal').classList.add('hidden');
}

async function saveClient() {
    const id = document.getElementById('client-id').value;
    const name = document.getElementById('client-name').value.trim();
    const email = document.getElementById('client-email').value.trim();
    const phone = document.getElementById('client-phone').value.trim();
    const notes = document.getElementById('client-notes').value.trim();

    if (!name) { await orionAlert('ORION', 'Client name is required'); return; }

    try {
        if (id) {
            await window.orionBridge.updateClient(parseInt(id), { name, email, phone, notes });
        } else {
            await window.orionBridge.createClient({ name, email, phone, notes });
        }
        closeClientModal();
        loadClientsView();
        if (typeof logSystem === 'function') logSystem(id ? `Client updated: ${name}` : `Client added: ${name}`);
    } catch (e) {
        console.error('Save client error:', e);
        await orionAlert('ORION', 'Error saving client');
    }
}

async function editClient(clientId) { openClientModal(clientId); }

async function deleteClient(clientId) {
    if (!await orionConfirm('Delete Client', 'Delete this client?')) return;
    try {
        await window.orionBridge.deleteClient(clientId);
        loadClientsView();
        if (typeof logSystem === 'function') logSystem('Client deleted');
    } catch (e) {
        console.error('Delete client error:', e);
    }
}

// ========================================
// LEADS VIEW
// ========================================

async function loadLeadsView() {
    const stages = ['cold', 'warm', 'hot', 'converted'];
    try {
        const leads = await window.orionBridge.listLeads();

        stages.forEach(stage => {
            const container = document.getElementById('leads-' + stage);
            if (!container) return;

            const stageLeads = leads.filter(l => l.stage === stage);
            if (stageLeads.length === 0) {
                container.innerHTML = '<div class="empty-view" style="font-size:11px;padding:8px">No leads</div>';
                return;
            }

            container.innerHTML = '';
            stageLeads.forEach(lead => {
                const card = document.createElement('div');
                card.className = 'lead-card';
                const nameDiv = document.createElement('div');
                nameDiv.className = 'lead-name';
                nameDiv.textContent = lead.name;
                card.appendChild(nameDiv);
                const companyDiv = document.createElement('div');
                companyDiv.className = 'lead-company';
                companyDiv.textContent = lead.company || 'No company';
                card.appendChild(companyDiv);
                if (lead.value > 0) {
                    const valueDiv = document.createElement('div');
                    valueDiv.className = 'lead-value';
                    valueDiv.textContent = '₹' + lead.value.toLocaleString();
                    card.appendChild(valueDiv);
                }
                const actions = document.createElement('div');
                actions.className = 'lead-actions';
                const editBtn = document.createElement('button');
                editBtn.textContent = '✎';
                editBtn.title = 'Edit';
                editBtn.onclick = () => editLead(lead.id);
                const moveBtn = document.createElement('button');
                moveBtn.textContent = '→';
                moveBtn.title = 'Move';
                moveBtn.onclick = () => moveLead(lead.id, stage);
                const delBtn = document.createElement('button');
                delBtn.textContent = '✕';
                delBtn.title = 'Delete';
                delBtn.onclick = () => deleteLead(lead.id);
                actions.appendChild(editBtn);
                actions.appendChild(moveBtn);
                actions.appendChild(delBtn);
                card.appendChild(actions);
                container.appendChild(card);
            });
        });
    } catch (e) {
        console.error('Leads view error:', e);
    }
}

function openLeadModal(leadId = null) {
    const modal = document.getElementById('lead-modal');
    const title = document.getElementById('lead-modal-title');
    const idInput = document.getElementById('lead-id');

    idInput.value = leadId || '';
    title.textContent = leadId ? 'Edit Lead' : 'Add Lead';

    if (leadId) {
        window.orionBridge.getLead(leadId).then(lead => {
            if (lead) {
                document.getElementById('lead-name').value = lead.name || '';
                document.getElementById('lead-company').value = lead.company || '';
                document.getElementById('lead-email').value = lead.email || '';
                document.getElementById('lead-phone').value = lead.phone || '';
                document.getElementById('lead-stage').value = lead.stage || 'cold';
                document.getElementById('lead-value').value = lead.value || '';
                document.getElementById('lead-notes').value = lead.notes || '';
            }
        });
    } else {
        document.getElementById('lead-name').value = '';
        document.getElementById('lead-company').value = '';
        document.getElementById('lead-email').value = '';
        document.getElementById('lead-phone').value = '';
        document.getElementById('lead-stage').value = 'cold';
        document.getElementById('lead-value').value = '';
        document.getElementById('lead-notes').value = '';
    }

    modal.classList.remove('hidden');
}

function closeLeadModal() {
    document.getElementById('lead-modal').classList.add('hidden');
}

async function saveLead() {
    const id = document.getElementById('lead-id').value;
    const name = document.getElementById('lead-name').value.trim();
    const company = document.getElementById('lead-company').value.trim();
    const email = document.getElementById('lead-email').value.trim();
    const phone = document.getElementById('lead-phone').value.trim();
    const stage = document.getElementById('lead-stage').value;
    const value = parseFloat(document.getElementById('lead-value').value) || 0;
    const notes = document.getElementById('lead-notes').value.trim();

    if (!name) { await orionAlert('ORION', 'Lead name is required'); return; }

    try {
        if (id) {
            await window.orionBridge.updateLead(parseInt(id), { name, company, email, phone, stage, value, notes });
        } else {
            await window.orionBridge.createLead({ name, company, email, phone, stage, value, notes });
        }
        closeLeadModal();
        loadLeadsView();
        if (typeof logSystem === 'function') logSystem(id ? `Lead updated: ${name}` : `Lead added: ${name}`);
    } catch (e) {
        console.error('Save lead error:', e);
        await orionAlert('ORION', 'Error saving lead');
    }
}

async function editLead(leadId) { openLeadModal(leadId); }

async function moveLead(leadId, currentStage) {
    const stages = ['cold', 'warm', 'hot', 'converted'];
    const nextIndex = stages.indexOf(currentStage) + 1;
    const nextStage = stages[nextIndex] || stages[0];
    try {
        await window.orionBridge.updateLead(leadId, { stage: nextStage });
        loadLeadsView();
    } catch (e) {
        console.error('Move lead error:', e);
    }
}

async function deleteLead(leadId) {
    if (!await orionConfirm('Delete Lead', 'Delete this lead?')) return;
    try {
        await window.orionBridge.deleteLead(leadId);
        loadLeadsView();
        if (typeof logSystem === 'function') logSystem('Lead deleted');
    } catch (e) {
        console.error('Delete lead error:', e);
    }
}

// ========================================
// CALENDAR VIEW
// ========================================

async function loadCalendarView() {
    try {
        const goals = await window.orionBridge.getGoals();
        const calendarGrid = document.getElementById('calendar-grid');

        if (calendarGrid) {
            const now = new Date();
            const year = now.getFullYear();
            const month = now.getMonth();
            const firstDay = new Date(year, month, 1).getDay();
            const daysInMonth = new Date(year, month + 1, 0).getDate();
            const today = now.getDate();

            let html = `<div class="calendar-header">
                <span class="calendar-title">${now.toLocaleString('default', { month: 'long', year: 'numeric' })}</span>
            </div>`;
            html += '<div class="calendar-weekdays">';
            ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'].forEach(d => {
                html += `<div>${d}</div>`;
            });
            html += '</div>';
            html += '<div class="calendar-days">';

            for (let i = 0; i < firstDay; i++) {
                html += '<div class="calendar-day empty"></div>';
            }

            const goalsByDay = {};
            (goals || []).forEach(g => {
                if (g.deadline) {
                    const d = new Date(g.deadline).getDate();
                    if (!goalsByDay[d]) goalsByDay[d] = [];
                    goalsByDay[d].push(g);
                }
            });

            for (let day = 1; day <= daysInMonth; day++) {
                const dayGoals = goalsByDay[day] || [];
                const isToday = day === today;
                html += `<div class="calendar-day ${isToday ? 'today' : ''} ${dayGoals.length > 0 ? 'has-goals' : ''}">
                    <span class="day-number">${day}</span>
                    ${dayGoals.slice(0, 2).map(g => `<div class="day-goal" title="${g.title}">${g.title.substring(0, 15)}</div>`).join('')}
                    ${dayGoals.length > 2 ? `<div class="day-more">+${dayGoals.length - 2} more</div>` : ''}
                </div>`;
            }

            html += '</div>';
            calendarGrid.innerHTML = html;
        }
    } catch (e) {
        console.error('Calendar view error:', e);
    }
}

// ========================================
// RIGHT SIDEBAR
// ========================================

async function loadRightSidebar() {
    if (!window.orionBridge) return;
    try {
        loadSessionInfo();
        loadSystemStatus();
        loadRecentActivity();
    } catch (e) {
        console.error('Sidebar load error:', e);
    }
}

async function loadSessionInfo() {
    try {
        const modeInfo = await window.orionBridge.getMode();
        let sessionId = await window.orionBridge.getCurrentSessionId();
        const history = sessionId ? await window.orionBridge.getHistory(sessionId) : [];

        const elMode = document.getElementById('right-session-mode');
        const elMsgs = document.getElementById('right-session-msgs');

        if (elMode) elMode.textContent = modeInfo?.display || 'ORION';
        if (elMsgs) elMsgs.textContent = (history && history.length) || 0;
    } catch (e) {
        console.error('Session info error:', e);
    }
}

async function loadSystemStatus() {
    try {
        const modeInfo = await window.orionBridge.getMode();
        const elAI = document.getElementById('right-status-ai');
        if (elAI) elAI.textContent = '● ' + (modeInfo?.display || 'Online');
    } catch (e) {
        console.error('System status error:', e);
    }
}

async function loadRecentActivity() {
    try {
        let sessionId = await window.orionBridge.getCurrentSessionId();
        const history = sessionId ? await window.orionBridge.getHistory(sessionId) : [];
        const container = document.getElementById('activity-list');
        if (!container) return;

        if (!history || history.length === 0) {
            container.innerHTML = '<div class="activity-item"><span class="activity-time">Now</span><span class="activity-text">ORION initialized</span></div>';
            return;
        }

        container.innerHTML = '';
        const recent = history.slice(-5).reverse();
        recent.forEach(msg => {
            const item = document.createElement('div');
            item.className = 'activity-item';
            const time = msg.timestamp ? new Date(msg.timestamp).toLocaleTimeString() : 'recent';
            const prefix = msg.role === 'assistant' ? '◆ ' : '● ';
            const text = (prefix + (msg.content || '')).substring(0, 50);
            const timeSpan = document.createElement('span');
            timeSpan.className = 'activity-time';
            timeSpan.textContent = time;
            const textSpan = document.createElement('span');
            textSpan.className = 'activity-text';
            textSpan.textContent = text;
            item.appendChild(timeSpan);
            item.appendChild(textSpan);
            container.appendChild(item);
        });
    } catch (e) {
        console.error('Activity error:', e);
    }
}

// ========================================
// TASKS
// ========================================

async function createNewTask() {
    const taskName = await orionPrompt('New Task', 'Enter task name:');
    if (!taskName) return;
    const deadline = await orionPrompt('New Task', 'Enter deadline (YYYY-MM-DD HH:MM) or leave empty:');
    try {
        if (window.orionBridge && window.orionBridge.createGoal) {
            await window.orionBridge.createGoal(taskName.trim(), '', deadline || null);
            if (typeof logSystem === 'function') logSystem(`Task created: ${taskName}`);
            loadTasksView();
        }
    } catch (e) {
        console.error('Create task error:', e);
    }
}

// ========================================
// SETTINGS TOGGLE
// ========================================

function toggleSettings() {
    switchView('settings');
}

// ========================================
// EXPOSE GLOBALS
// ========================================

window.switchView = switchView;
window.openClientModal = openClientModal;
window.closeClientModal = closeClientModal;
window.saveClient = saveClient;
window.editClient = editClient;
window.deleteClient = deleteClient;
window.openLeadModal = openLeadModal;
window.closeLeadModal = closeLeadModal;
window.saveLead = saveLead;
window.editLead = editLead;
window.moveLead = moveLead;
window.deleteLead = deleteLead;
window.loadTasksView = loadTasksView;
window.loadClientsView = loadClientsView;
window.loadLeadsView = loadLeadsView;
window.loadCalendarView = loadCalendarView;
window.loadAnalyticsView = loadAnalyticsView;
window.loadSettingsView = loadSettingsView;
window.loadSystemsView = loadSystemsView;
window.loadDashboardStats = loadDashboardStats;
window.loadProjectsView = loadProjectsView;
window.openProject = openProject;
window.openSession = openSession;
window.createChatInCurrentProject = createChatInCurrentProject;
window.showCreateProject = showCreateProject;
window.createNewTask = createNewTask;
window.toggleSettings = toggleSettings;
window.renameProject = renameProject;
window.deleteProject = deleteProject;
window.renameSessionFromView = renameSessionFromView;
window.deleteSessionFromView = deleteSessionFromView;
