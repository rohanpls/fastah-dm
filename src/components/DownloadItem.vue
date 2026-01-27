<script setup lang="ts">
import { ref, computed } from 'vue';
import type { DownloadItem } from '../stores/downloadStore';
import { useDownloadStore } from '../stores/downloadStore';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import ConfirmDialog from './ConfirmDialog.vue';

const props = defineProps<{ item: DownloadItem }>();
const store = useDownloadStore();

const showDeleteDialog = ref(false);
const deleteWithFile = ref(false);

const percentage = computed(() => {
    if (!props.item.total) return 0;
    return Math.min((props.item.downloaded / props.item.total) * 100, 100);
});

const speedStr = computed(() => {
    const s = props.item.speed;
    if (s < 1024) return `${s} B/s`;
    if (s < 1024 * 1024) return `${(s / 1024).toFixed(1)} KB/s`;
    return `${(s / 1024 / 1024).toFixed(1)} MB/s`;
});

const sizeStr = computed(() => {
    const d = props.item.downloaded;
    const t = props.item.total;
    const downloaded = d < 1024 * 1024 
        ? `${(d / 1024).toFixed(1)} KB` 
        : `${(d / 1024 / 1024).toFixed(1)} MB`;
    const total = t 
        ? (t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(1)} MB`)
        : '?';
    return `${downloaded} / ${total}`;
});

const statusIcon = computed(() => {
    switch (props.item.status) {
        case 'downloading': return '↓';
        case 'paused': return '⏸';
        case 'completed': return '✓';
        case 'error': return '✕';
        default: return '○';
    }
});

const statusClass = computed(() => `status-${props.item.status}`);

const fullPath = computed(() => {
    const sep = navigator.userAgent.includes("Windows") ? "\\" : "/";
    return `${props.item.path}${sep}${props.item.filename}`;
});

function toggle() {
    if (props.item.status === 'downloading') {
        store.pauseDownload(props.item);
    } else if (['paused', 'error'].includes(props.item.status)) {
        store.resumeDownload(props.item);
    }
}

async function openDirectory() {
    try {
        await revealItemInDir(fullPath.value);
    } catch (e) {
        console.error('Failed to open directory:', e);
    }
}

function openDeleteDialog() {
    deleteWithFile.value = false;
    showDeleteDialog.value = true;
}

async function confirmDelete() {
    await store.removeDownload(props.item, deleteWithFile.value);
    showDeleteDialog.value = false;
}
</script>

<template>
  <div class="download-item glass-panel" :class="statusClass">
    <div class="item-header">
      <div class="file-info">
        <div class="filename" :title="item.filename">{{ item.filename }}</div>
        <div class="location" :title="item.path">{{ item.path }}</div>
        <div class="url" :title="item.url">{{ item.url }}</div>
      </div>
      
      <button class="btn-icon delete-btn" @click="openDeleteDialog" title="Remove">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
    
    <div class="progress-section">
      <div class="progress2" :class="{ 'progress-moved': percentage > 0 }">
        <div class="progress-bar2" :style="{ width: percentage + '%' }"></div>
      </div>
      <span class="percentage">{{ percentage.toFixed(0) }}%</span>
    </div>
    
    <div class="item-footer">
      <div class="status-info">
        <span class="status-badge" :class="statusClass">
          <span class="status-icon">{{ statusIcon }}</span>
          {{ item.status }}
        </span>
        <span v-if="item.status === 'downloading'" class="speed">{{ speedStr }}</span>
        <span class="size">{{ sizeStr }}</span>
      </div>
      
      <div class="controls">
        <!-- Open Directory Button -->
        <button 
          class="btn btn-sm btn-action" 
          @click="openDirectory"
          title="Open Directory"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          </svg>
          Folder
        </button>
        
        <button 
          v-if="item.status !== 'completed'"
          class="btn btn-sm" 
          :class="item.status === 'downloading' ? 'btn-warning' : 'btn-success'"
          @click="toggle"
        >
          <svg v-if="item.status === 'downloading'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="6" y="4" width="4" height="16"></rect>
            <rect x="14" y="4" width="4" height="16"></rect>
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="5 3 19 12 5 21 5 3"></polygon>
          </svg>
          {{ item.status === 'downloading' ? 'Pause' : 'Resume' }}
        </button>
        
        <span v-else class="completed-badge">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
          Completed
        </span>
      </div>
    </div>
    
    <!-- Delete Confirmation Dialog -->
    <ConfirmDialog
      :show="showDeleteDialog"
      title="Remove Download"
      :message="'Remove this download from the list?'"
      confirm-text="Remove"
      :danger="true"
      @confirm="confirmDelete"
      @cancel="showDeleteDialog = false"
    >
      <template #extra>
        <label class="checkbox-label">
          <input type="checkbox" v-model="deleteWithFile" />
          Also delete the downloaded file
        </label>
      </template>
    </ConfirmDialog>
  </div>
</template>

<style scoped>
.download-item {
  position: relative;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.download-item:hover {
  transform: translateY(-2px);
}

.download-item::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, #a855f7, #6366f1, #3b82f6);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.download-item:hover::before {
  opacity: 1;
}

.download-item.status-downloading::before {
  opacity: 1;
  animation: shimmer 2s ease-in-out infinite;
}

@keyframes shimmer {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 10px;
  margin-bottom: 12px;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.filename {
  font-weight: 600;
  font-size: 0.95em;
  color: white;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 2px;
}

.location {
  font-size: 0.75em;
  color: rgba(255, 255, 255, 0.4);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 2px;
}

.url {
  font-size: 0.8em;
  color: rgba(255, 255, 255, 0.5);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.delete-btn {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  border: none;
  background: rgba(239, 68, 68, 0.1);
  color: rgba(239, 68, 68, 0.7);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.delete-btn:hover {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  transform: scale(1.1);
}

.delete-btn:active {
  transform: scale(0.95);
}

.progress-section {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

/* Codepen Progress Bar Style #2 */
.progress2 {
  flex: 1;
  height: 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  overflow: hidden;
  position: relative;
}

.progress-bar2 {
  height: 100%;
  background: linear-gradient(90deg, #a855f7 0%, #6366f1 50%, #3b82f6 100%);
  border-radius: 4px;
  transition: width 0.3s ease-out;
  position: relative;
}

.progress2.progress-moved .progress-bar2 {
  animation: progressAnimation 2s ease-in-out;
}

@keyframes progressAnimation {
  0% {
    width: 0;
    opacity: 0.5;
  }
  100% {
    opacity: 1;
  }
}

.percentage {
  font-size: 0.85em;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.8);
  min-width: 40px;
  text-align: right;
}

.item-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}

.status-info {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 0.75em;
  font-weight: 500;
  text-transform: capitalize;
}

.status-icon {
  font-size: 0.9em;
}

.status-badge.status-downloading {
  background: rgba(59, 130, 246, 0.2);
  color: #60a5fa;
}

.status-badge.status-paused {
  background: rgba(251, 191, 36, 0.2);
  color: #fbbf24;
}

.status-badge.status-completed {
  background: rgba(34, 197, 94, 0.2);
  color: #4ade80;
}

.status-badge.status-error {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

.speed {
  font-size: 0.8em;
  color: #60a5fa;
  font-weight: 500;
}

.size {
  font-size: 0.8em;
  color: rgba(255, 255, 255, 0.5);
}

.controls {
  display: flex;
  gap: 8px;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 999px;
  font-size: 0.8em;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
}

.btn:active {
  transform: scale(0.95);
}

.btn-sm {
  padding: 6px 12px;
  font-size: 0.75em;
}

.btn-warning {
  background: rgba(251, 191, 36, 0.2);
  color: #fbbf24;
  border: 1px solid rgba(251, 191, 36, 0.3);
}

.btn-warning:hover {
  background: rgba(251, 191, 36, 0.3);
}

.btn-success {
  background: rgba(34, 197, 94, 0.2);
  color: #4ade80;
  border: 1px solid rgba(34, 197, 94, 0.3);
}

.btn-success:hover {
  background: rgba(34, 197, 94, 0.3);
}

.btn-action {
  background: rgba(168, 85, 247, 0.2);
  color: #c084fc;
  border: 1px solid rgba(168, 85, 247, 0.3);
}

.btn-action:hover {
  background: rgba(168, 85, 247, 0.3);
}

.completed-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 999px;
  font-size: 0.75em;
  font-weight: 500;
  background: rgba(34, 197, 94, 0.15);
  color: #4ade80;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9em;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  margin-top: 15px;
}

.checkbox-label input {
  accent-color: #a855f7;
}
</style>
