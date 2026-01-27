<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue';
import { useDownloadStore } from '../stores/downloadStore';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { ask } from '@tauri-apps/plugin-dialog';
import { getVersion } from '@tauri-apps/api/app';

const props = defineProps<{ show: boolean }>();
const emit = defineEmits(['close']);

const store = useDownloadStore();
const theme = ref<'light' | 'dark'>(store.settings?.theme as 'light' | 'dark' || 'dark');
const launchOnStartup = ref(store.settings?.launch_on_startup || false);
const toggleKeybind = ref(store.settings?.toggle_keybind || 'Ctrl+Shift+D');
const capturingKeybind = ref(false);

// Update settings
const autoUpdateEnabled = ref(store.settings?.auto_update_enabled ?? true);
const silentUpdates = ref(store.settings?.silent_updates ?? false);
const appVersion = ref('0.2.0');
const updateStatus = ref<'checking' | 'available' | 'up-to-date' | 'error' | null>(null);

// Original values for change detection
const originalTheme = ref<'light' | 'dark'>('dark');
const originalLaunchOnStartup = ref(false);
const originalToggleKeybind = ref('Ctrl+Shift+D');
const originalAutoUpdateEnabled = ref(true);
const originalSilentUpdates = ref(false);

// Detect if there are unsaved changes
const hasChanges = computed(() => {
  return theme.value !== originalTheme.value ||
         launchOnStartup.value !== originalLaunchOnStartup.value ||
         toggleKeybind.value !== originalToggleKeybind.value ||
         autoUpdateEnabled.value !== originalAutoUpdateEnabled.value ||
         silentUpdates.value !== originalSilentUpdates.value;
});

onMounted(async () => {
  try {
    appVersion.value = await getVersion();
  } catch (e) {
    console.error('Failed to get app version:', e);
  }
});

watch(() => props.show, async (newVal) => {
  if (newVal) {
    // Load current settings
    theme.value = (store.settings?.theme as 'light' | 'dark') || 'dark';
    launchOnStartup.value = store.settings?.launch_on_startup || false;
    toggleKeybind.value = store.settings?.toggle_keybind || 'Ctrl+Shift+D';
    autoUpdateEnabled.value = store.settings?.auto_update_enabled ?? true;
    silentUpdates.value = store.settings?.silent_updates ?? false;
    
    // Store original values
    originalTheme.value = theme.value;
    originalLaunchOnStartup.value = launchOnStartup.value;
    originalToggleKeybind.value = toggleKeybind.value;
    originalAutoUpdateEnabled.value = autoUpdateEnabled.value;
    originalSilentUpdates.value = silentUpdates.value;
    
    // Check actual autostart status
    try {
      launchOnStartup.value = await isEnabled();
      originalLaunchOnStartup.value = launchOnStartup.value;
    } catch (e) {
      console.error('Failed to check autostart status:', e);
    }
    
    // Update status based on store state
    if (store.updateAvailable) {
      updateStatus.value = 'available';
    }
  }
});

async function saveSettings() {
  // Handle autostart
  try {
    if (launchOnStartup.value) {
      await enable();
    } else {
      await disable();
    }
  } catch (e) {
    console.error('Failed to toggle autostart:', e);
  }
  
  await store.updateSettings({ 
    theme: theme.value,
    launch_on_startup: launchOnStartup.value,
    toggle_keybind: toggleKeybind.value || null,
    use_new_ui: true,
    auto_update_enabled: autoUpdateEnabled.value,
    silent_updates: silentUpdates.value
  });
  
  // Update original values after save
  originalTheme.value = theme.value;
  originalLaunchOnStartup.value = launchOnStartup.value;
  originalToggleKeybind.value = toggleKeybind.value;
  originalAutoUpdateEnabled.value = autoUpdateEnabled.value;
  originalSilentUpdates.value = silentUpdates.value;
}

function cancelSettings() {
  // Restore original values
  theme.value = originalTheme.value;
  launchOnStartup.value = originalLaunchOnStartup.value;
  toggleKeybind.value = originalToggleKeybind.value;
  autoUpdateEnabled.value = originalAutoUpdateEnabled.value;
  silentUpdates.value = originalSilentUpdates.value;
}

async function clearHistory() {
  const answer = await ask('Clear all download history? This will not delete downloaded files.', {
    title: 'Confirm',
    kind: 'warning'
  });
  
  if (answer) {
    await store.clearAllHistory();
  }
}

async function checkForUpdatesManually() {
  updateStatus.value = 'checking';
  
  try {
    const hasUpdate = await store.checkForUpdates();
    
    if (hasUpdate) {
      updateStatus.value = 'available';
    } else {
      updateStatus.value = 'up-to-date';
      setTimeout(() => {
        updateStatus.value = null;
      }, 3000);
    }
  } catch (e) {
    console.error('Failed to check for updates:', e);
    updateStatus.value = 'error';
    setTimeout(() => {
      updateStatus.value = null;
    }, 3000);
  }
}

let keydownListener: ((e: KeyboardEvent) => void) | null = null;

function stopKeybindCapture() {
  capturingKeybind.value = false;
  if (keydownListener) {
    window.removeEventListener('keydown', keydownListener);
    keydownListener = null;
  }
}

function startKeybindCapture() {
  if (capturingKeybind.value) {
    stopKeybindCapture();
    return;
  }
  
  capturingKeybind.value = true;
  
  keydownListener = (e: KeyboardEvent) => {
    e.preventDefault();
    
    // Allow cancelling with Esc
    if (e.key === 'Escape') {
      stopKeybindCapture();
      return;
    }
    
    // Build combination string
    const parts: string[] = [];
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.shiftKey) parts.push('Shift');
    if (e.altKey) parts.push('Alt');
    if (e.metaKey) parts.push('Meta');
    
    // Add the actual key if it's not a modifier
    if (!['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
      parts.push(e.key.toUpperCase());
    }
    
    // Only set if we have at least one modifier + one key, 
    // OR if we have Function keys or special keys that might work alone
    if (parts.length > 0) {
      // Don't set if it's just a modifier
      const last = parts[parts.length - 1];
      const isModifierOnly = ['Ctrl', 'Shift', 'Alt', 'Meta'].includes(last);
      
      if (!isModifierOnly) {
         toggleKeybind.value = parts.join('+');
         stopKeybindCapture();
      }
    }
  };
  
  window.addEventListener('keydown', keydownListener);
}
</script>

<template>
  <div class="panel-container">
    <div class="modal-content">
      <div class="modal-header">
        <h2>Settings</h2>
        <div class="header-actions">
          <button class="btn btn-secondary" @click="cancelSettings" :disabled="!hasChanges">Cancel</button>
          <button class="btn btn-primary" @click="saveSettings" :disabled="!hasChanges">Save</button>
          <button class="close-btn btn-icon" @click="emit('close')">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            </div>
          </div>
          
          <div class="modal-body">
            <div class="setting-group">
              <label>Theme</label>
              <div class="theme-selector">
                <button 
                  class="theme-btn" 
                  :class="{ active: theme === 'light' }"
                  @click="theme = 'light'"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="5"></circle>
                    <line x1="12" y1="1" x2="12" y2="3"></line>
                    <line x1="12" y1="21" x2="12" y2="23"></line>
                    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                    <line x1="1" y1="12" x2="3" y2="12"></line>
                    <line x1="21" y1="12" x2="23" y2="12"></line>
                    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                  </svg>
                  <span>Light</span>
                </button>
                <button 
                  class="theme-btn" 
                  :class="{ active: theme === 'dark' }"
                  @click="theme = 'dark'"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                  </svg>
                  <span>Dark</span>
                </button>
              </div>
            </div>

            <div class="setting-group">
              <label class="toggle-label">
                <input type="checkbox" v-model="launchOnStartup" />
                <span>Launch on startup</span>
              </label>
            </div>

            <div class="setting-group">
              <label>Toggle Window Keybind</label>
              <div class="keybind-input-group">
                <input 
                  v-model="toggleKeybind" 
                  type="text" 
                  placeholder="Ctrl+Shift+D"
                  class="glass-input"
                  readonly
                  :class="{ 'capturing': capturingKeybind }"
                />
                <button class="btn" :class="capturingKeybind ? 'btn-danger' : 'btn-primary'" @click="startKeybindCapture">
                  {{ capturingKeybind ? 'listening... (Esc to cancel)' : 'Set' }}
                </button>
              </div>
              <span class="hint">Click "Set" then press your desired key combination</span>
            </div>

            <div class="setting-group">
              <label>Download History</label>
              <button class="btn btn-danger" @click="clearHistory">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="3 6 5 6 21 6"></polyline>
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                </svg>
                Clear All History
              </button>
            </div>

            <div class="divider"></div>

            <!-- Updates Section -->
            <div class="setting-group">
              <label>Updates</label>
              
              <label class="toggle-label">
                <input type="checkbox" v-model="autoUpdateEnabled" />
                <span>Auto-update enabled</span>
              </label>

              <label class="toggle-label" :class="{ disabled: !autoUpdateEnabled }">
                <input type="checkbox" v-model="silentUpdates" :disabled="!autoUpdateEnabled" />
                <span>Silent updates (install without prompts)</span>
              </label>

              <div class="update-check-section">
                <button 
                  class="btn btn-primary" 
                  @click="checkForUpdatesManually"
                  :disabled="updateStatus === 'checking'"
                >
                  <svg v-if="updateStatus === 'checking'" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="spinning">
                    <line x1="12" y1="2" x2="12" y2="6"></line>
                    <line x1="12" y1="18" x2="12" y2="22"></line>
                    <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
                    <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                    <line x1="2" y1="12" x2="6" y2="12"></line>
                    <line x1="18" y1="12" x2="22" y2="12"></line>
                    <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
                    <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
                  </svg>
                  <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="23 4 23 10 17 10"></polyline>
                    <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
                  </svg>
                  {{ updateStatus === 'checking' ? 'Checking...' : 'Check for Updates' }}
                </button>
                
                <div class="update-status" v-if="updateStatus">
                  <span v-if="updateStatus === 'available'" class="status-badge status-available">
                    Update available: {{ store.updateInfo?.version }}
                  </span>
                  <span v-if="updateStatus === 'up-to-date'" class="status-badge status-success">
                    You're up to date!
                  </span>
                  <span v-if="updateStatus === 'error'" class="status-badge status-error">
                    Failed to check for updates
                  </span>
                </div>
              </div>

              <span class="hint">
                Fallback: <a href="https://github.com/rohanpls/fastah-dm/releases" target="_blank" class="link">View releases</a>
              </span>
            </div>

            <div class="divider"></div>

            <div class="about-section">
              <h3>About</h3>
              <p class="app-name">Fastah Download Manager</p>
              <p class="version">Version {{ appVersion }}</p>
              <p class="author">Made with ❤️ by <strong>@rohanpls</strong></p>
            </div>
          </div>
        </div>
      </div>
</template>

<style scoped>
.panel-container {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: flex-start;
  justify-content: flex-start;
  overflow-y: auto;
}

.modal-content {
  width: 100%;
  max-width: 800px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 24px;
  animation: slideUp 0.3s ease-out;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 15px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.modal-header h2 {
  margin: 0;
  font-size: 1.4em;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-actions .btn {
  padding: 8px 16px;
  font-size: 0.85em;
}

.close-btn {
  background: var(--bg-tertiary);
  border: none;
  color: var(--text-secondary);
  width: 32px;
  height: 32px;
  border-radius: 999px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  transform: rotate(90deg);
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.setting-group label {
  font-size: 0.9em;
  color: var(--text-primary);
  font-weight: 500;
}

.glass-input {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 12px 16px;
  color: var(--text-primary);
  font-size: 0.95em;
  transition: all 0.2s ease;
}

.glass-input::placeholder {
  color: var(--text-secondary);
  opacity: 0.7;
}

.glass-input:focus {
  outline: none;
  border-color: var(--accent-color);
  box-shadow: 0 0 0 2px var(--accent-color-transparent);
}

.theme-selector {
  display: flex;
  gap: 12px;
}

.theme-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 12px 20px;
  background: var(--bg-primary);
  border: 2px solid var(--border-color);
  border-radius: 12px;
  color: var(--text-secondary);
  font-size: 0.95em;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.theme-btn:hover {
  border-color: var(--accent-color);
  color: var(--text-primary);
}

.theme-btn.active {
  background: var(--accent-color);
  border-color: var(--accent-color);
  color: white;
}

.glass-input.capturing {
  border-color: #10b981;
  background: rgba(16, 185, 129, 0.1);
  animation: pulse 1s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    border-color: #10b981;
  }
  50% {
    border-color: #34d399;
  }
}

.keybind-input-group {
  display: flex;
  gap: 8px;
  align-items: center;
}

.keybind-input-group .glass-input {
  flex: 1;
}

.keybind-input-group .btn {
  padding: 12px 20px;
  white-space: nowrap;
}

.hint {
  font-size: 0.8em;
  color: var(--text-secondary);
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  font-size: 0.95em;
  color: var(--text-primary);
}

.toggle-label input {
  accent-color: var(--accent-color);
  width: 18px;
  height: 18px;
}

.toggle-label.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.divider {
  height: 1px;
  background: var(--border-color);
  margin: 8px 0;
}

.update-check-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.update-status {
  display: flex;
  align-items: center;
}

.status-badge {
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 0.85em;
  font-weight: 500;
}

.status-available {
  background: rgba(59, 130, 246, 0.2);
  color: var(--accent-color);
}

.status-success {
  background: rgba(16, 185, 129, 0.2);
  color: var(--success-color);
}

.status-error {
  background: rgba(239, 68, 68, 0.2);
  color: var(--error-color);
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.link {
  color: var(--accent-color);
  text-decoration: none;
  transition: opacity 0.2s ease;
}

.link:hover {
  opacity: 0.8;
  text-decoration: underline;
}

.about-section {
  margin-top: 10px;
  padding-top: 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  text-align: center;
}

.about-section h3 {
  margin: 0 0 10px 0;
  font-size: 1em;
  color: var(--text-secondary);
}

.app-name {
  font-size: 1.2em;
  font-weight: 700;
  margin: 0;
  color: var(--accent-color);
}

.version {
  font-size: 0.85em;
  color: var(--text-secondary);
  margin: 4px 0;
}

.author {
  font-size: 0.9em;
  color: var(--text-secondary);
  margin: 8px 0 0 0;
}

.author strong {
  color: var(--accent-color);
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border-radius: 999px;
  font-size: 0.9em;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
}

.btn:active {
  transform: scale(0.95);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}

.btn-primary {
  background: var(--accent-color);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--accent-color-hover);
  transform: translateY(-1px);
}

.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--bg-tertiary);
  filter: brightness(1.1);
}

.btn-danger {
  background: var(--error-color);
  color: white;
  border: none;
}

.btn-danger:hover:not(:disabled) {
  filter: brightness(0.9);
}

/* Transition animations */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-content,
.modal-leave-to .modal-content {
  transform: translateY(20px) scale(0.95);
}
</style>
