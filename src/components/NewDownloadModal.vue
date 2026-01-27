<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useDownloadStore } from '../stores/downloadStore';

const props = defineProps<{ show: boolean }>();
const emit = defineEmits(['close']);

const store = useDownloadStore();
const url = ref('');

// Duplicate file dialog state
const showDuplicateDialog = ref(false);
const duplicateFilename = ref('');

// Format bytes to human readable
function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  if (bytes < 1024 * 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`;
  return `${(bytes / 1024 / 1024 / 1024 / 1024).toFixed(1)} TB`;
}

const storageDisplay = computed(() => {
  if (!store.storageInfo) return null;
  return {
    used: formatBytes(store.storageInfo.used),
    free: formatBytes(store.storageInfo.free),
    total: formatBytes(store.storageInfo.total),
    percentage: Math.round((store.storageInfo.used / store.storageInfo.total) * 100)
  };
});

watch(() => props.show, (newVal) => {
  if (newVal) {
    url.value = '';
  }
});

async function handleSubmit() {
  if (!url.value || !store.selectedPath) return;
  
  const filename = url.value.split('/').pop()?.split('?')[0] || `file_${Date.now()}`;
  
  try {
    const exists = await store.checkFileExists(filename);
    if (exists) {
      duplicateFilename.value = filename;
      showDuplicateDialog.value = true;
      return;
    }
    
    await store.startDownload(url.value, filename);
    url.value = '';
    emit('close');
  } catch (e: any) {
    alert('Failed to start download: ' + e);
  }
}

async function handleDuplicateReplace() {
  showDuplicateDialog.value = false;
  try {
    await store.startDownload(url.value, duplicateFilename.value);
    url.value = '';
    emit('close');
  } catch (e: any) {
    alert('Failed to start download: ' + e);
  }
}

async function handleDuplicateRename() {
  showDuplicateDialog.value = false;
  try {
    const newFilename = await store.findAvailableFilename(duplicateFilename.value);
    await store.startDownload(url.value, newFilename);
    url.value = '';
    emit('close');
  } catch (e: any) {
    alert('Failed to start download: ' + e);
  }
}

function handleDuplicateCancel() {
  showDuplicateDialog.value = false;
  duplicateFilename.value = '';
}
</script>

<template>
  <div class="panel-container">
    <div class="glass-modal">
      <div class="modal-header">
        <h3>
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="8" x2="12" y2="16"></line>
                <line x1="8" y1="12" x2="16" y2="12"></line>
              </svg>
              New Download
            </h3>
            <button class="close-btn" @click="emit('close')">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
          
          <div class="modal-body">
            <!-- URL Input -->
            <div class="form-group">
              <label>URL</label>
              <div class="input-with-icon">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                  <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
                </svg>
                <input 
                  v-model="url" 
                  type="url" 
                  placeholder="https://example.com/file.zip"
                  @keydown.enter="handleSubmit"
                />
              </div>
            </div>
            
            <!-- Save Location -->
            <div class="form-group">
              <label>Save to</label>
              <div class="folder-selector">
                <div class="input-with-icon flex-1">
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                  </svg>
                  <input 
                    :value="store.selectedPath || 'Select folder...'" 
                    readonly
                  />
                </div>
                <button class="browse-btn" @click="store.selectFolder">
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="1"></circle>
                    <circle cx="19" cy="12" r="1"></circle>
                    <circle cx="5" cy="12" r="1"></circle>
                  </svg>
                </button>
              </div>
              
              <!-- Storage Info -->
              <div v-if="storageDisplay" class="storage-info">
                <div class="storage-bar">
                  <div class="storage-fill" :style="{ width: storageDisplay.percentage + '%' }"></div>
                </div>
                <div class="storage-text">
                  <span>{{ storageDisplay.free }} free</span>
                  <span>of {{ storageDisplay.total }}</span>
                </div>
              </div>
            </div>
          </div>
          
          <div class="modal-footer">
            <button class="btn btn-secondary" @click="emit('close')">Cancel</button>
            <button 
              class="btn btn-primary" 
              @click="handleSubmit"
              :disabled="!url || !store.selectedPath"
            >
              Start Download
            </button>
          </div>
        </div>
        
        <!-- Duplicate File Dialog -->
        <div v-if="showDuplicateDialog" class="duplicate-dialog glass-modal">
          <h3>File Already Exists</h3>
          <p>A file named <strong>"{{ duplicateFilename }}"</strong> already exists.</p>
          <div class="dialog-actions">
            <button class="btn btn-secondary" @click="handleDuplicateCancel">Cancel</button>
            <button class="btn btn-warning" @click="handleDuplicateReplace">Replace</button>
            <button class="btn btn-primary" @click="handleDuplicateRename">Keep Both</button>
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

.glass-modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 20px;
  padding: 24px;
  width: 100%;
  max-width: 800px;
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
  margin-bottom: 24px;
}

.modal-header h3 {
  margin: 0;
  font-size: 1.25em;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-header h3 svg {
  color: var(--accent-color);
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px;
  border-radius: 999px;
  transition: all 0.2s ease;
}

.close-btn:hover {
  color: var(--text-primary);
  background: var(--bg-tertiary);
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-size: 0.875em;
  font-weight: 500;
  color: var(--text-primary);
}

.input-with-icon {
  position: relative;
  display: flex;
  align-items: center;
}

.input-with-icon svg {
  position: absolute;
  left: 12px;
  color: var(--text-secondary);
  pointer-events: none;
}

.input-with-icon input {
  width: 100%;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 18px;
  padding: 10px 12px 10px 40px;
  color: var(--text-primary);
  font-size: 0.9em;
  transition: all 0.2s ease;
}

.input-with-icon input::placeholder {
  color: var(--text-secondary);
  opacity: 0.7;
}

.input-with-icon input:focus {
  outline: none;
  border-color: var(--accent-color);
  box-shadow: 0 0 0 2px var(--accent-color-transparent);
}

.input-with-icon input[readonly] {
  cursor: default;
  pointer-events: none;
}

.folder-selector {
  display: flex;
  gap: 8px;
}

.folder-selector .flex-1 {
  flex: 1;
}

.browse-btn {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 0 12px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.browse-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  filter: brightness(1.1);
}

.storage-info {
  margin-top: 8px;
}

.storage-bar {
  height: 4px;
  background: var(--bg-tertiary);
  border-radius: 999px;
  overflow: hidden;
}

.storage-fill {
  height: 100%;
  background: var(--accent-color);
  border-radius: 999px;
  transition: width 0.3s ease;
}

.storage-text {
  display: flex;
  justify-content: space-between;
  font-size: 0.75em;
  color: var(--text-secondary);
  margin-top: 4px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border-radius: 999px;
  font-size: 0.875em;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
}

.btn-primary {
  background: var(--accent-color);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--accent-color-hover);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: transparent;
  color: var(--text-secondary);
}

.btn-secondary:hover {
  color: var(--text-primary);
  background: var(--bg-tertiary);
}

.btn-warning {
  background: var(--warning-color);
  color: white;
}

.btn-warning:hover {
  filter: brightness(0.9);
}

.duplicate-dialog {
  position: absolute;
  max-width: 400px;
  text-align: center;
}

.duplicate-dialog h3 {
  margin: 0 0 12px;
  font-size: 1.1em;
  color: var(--text-primary);
}

.duplicate-dialog p {
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.9em;
}

.duplicate-dialog strong {
  color: var(--accent-color);
}

.dialog-actions {
  display: flex;
  gap: 12px;
  margin-top: 20px;
  justify-content: center;
}

/* Transitions */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .glass-modal,
.modal-leave-to .glass-modal {
  transform: translateY(20px) scale(0.95);
}
</style>
