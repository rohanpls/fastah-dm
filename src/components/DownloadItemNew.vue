<script setup lang="ts">
import { computed } from 'vue';
import { useDownloadStore, type DownloadItem } from '../stores/downloadStore';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

const props = defineProps<{
  item: DownloadItem;
}>();

const emit = defineEmits<{
  (e: 'showNotification', message: string): void;
}>();

const store = useDownloadStore();

// File type icon mapping
const fileTypeIcon = computed(() => {
  const ext = props.item.filename.split('.').pop()?.toLowerCase() || '';
  
  // Video
  if (['mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm'].includes(ext)) {
    return { type: 'video', color: '#E879F9' };
  }
  // Audio
  if (['mp3', 'wav', 'flac', 'aac', 'ogg', 'm4a'].includes(ext)) {
    return { type: 'audio', color: '#22D3EE' };
  }
  // Image
  if (['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg', 'ico'].includes(ext)) {
    return { type: 'image', color: '#34D399' };
  }
  // Document
  if (['pdf', 'doc', 'docx', 'txt', 'rtf', 'odt'].includes(ext)) {
    return { type: 'document', color: '#FB923C' };
  }
  // Archive
  if (['zip', 'rar', '7z', 'tar', 'gz', 'bz2'].includes(ext)) {
    return { type: 'archive', color: '#FBBF24' };
  }
  // Executable
  if (['exe', 'msi', 'dmg', 'app', 'deb', 'rpm'].includes(ext)) {
    return { type: 'executable', color: '#F87171' };
  }
  // Code
  if (['js', 'ts', 'py', 'java', 'cpp', 'c', 'h', 'css', 'html', 'json'].includes(ext)) {
    return { type: 'code', color: '#818CF8' };
  }
  return { type: 'file', color: '#94A3B8' };
});

const isActive = computed(() => ['downloading', 'pending', 'paused'].includes(props.item.status));

const progressPercent = computed(() => {
  if (!props.item.total || props.item.total === 0) return 0;
  return Math.round((props.item.downloaded / props.item.total) * 100);
});

const speedText = computed(() => {
  if (props.item.status !== 'downloading') return '';
  const speed = props.item.speed;
  if (speed < 1024) return `${speed.toFixed(0)} B/s`;
  if (speed < 1024 * 1024) return `${(speed / 1024).toFixed(1)} KB/s`;
  return `${(speed / 1024 / 1024).toFixed(2)} MB/s`;
});

const sizeText = computed(() => {
  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
    return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
  };
  
  if (!props.item.total || props.item.total === 0) {
    return formatSize(props.item.downloaded);
  }
  return `${formatSize(props.item.downloaded)} / ${formatSize(props.item.total)}`;
});

const timeRemaining = computed(() => {
  if (props.item.status !== 'downloading' || props.item.speed === 0 || !props.item.total) return '';
  const remaining = props.item.total - props.item.downloaded;
  const seconds = Math.round(remaining / props.item.speed);
  
  if (seconds < 60) return `${seconds}s remaining`;
  if (seconds < 3600) return `${Math.round(seconds / 60)}m remaining`;
  const hours = Math.floor(seconds / 3600);
  const mins = Math.round((seconds % 3600) / 60);
  return `${hours}h ${mins}m remaining`;
});

const folderPath = computed(() => {
  // Show full directory path
  return props.item.path;
});

// Status badge color
const statusColor = computed(() => {
  switch (props.item.status) {
    case 'completed': return '#34D399';
    case 'downloading': return '#4988C4';
    case 'paused': return '#FBBF24';
    case 'error': return '#F87171';
    default: return '#94A3B8';
  }
});

async function togglePause() {
  if (props.item.status === 'downloading') {
    await store.pauseDownload(props.item);
  } else if (props.item.status === 'paused') {
    await store.resumeDownload(props.item);
  }
}

async function cancelDownload() {
  await store.removeDownload(props.item, false);
}

async function deleteItem() {
  await store.removeDownload(props.item, false);
}

async function openFolder() {
  const sep = navigator.userAgent.includes("Windows") ? "\\" : "/";
  const fullPath = `${props.item.path}${sep}${props.item.filename}`;
  try {
    await revealItemInDir(fullPath);
  } catch (e) {
    console.error("Failed to reveal file in directory:", e);
  }
}

async function copyDownloadLink() {
  const urlToCopy = props.item.downloadType === 'gdrive' && props.item.originalUrl 
    ? props.item.originalUrl 
    : props.item.url;
  
  try {
    await writeText(urlToCopy);
    emit('showNotification', 'Link copied to clipboard');
  } catch (err) {
    console.error('Failed to copy:', err);
  }
}
</script>

<template>
  <div class="download-item-new" :class="{ 'is-completed': item.status === 'completed' }">
    <!-- File Type Icon -->
    <div class="file-icon" :style="{ background: item.downloadType === 'gdrive' ? 'rgba(66, 133, 244, 0.1)' : `${fileTypeIcon.color}20`, color: item.downloadType === 'gdrive' ? '#4285f4' : fileTypeIcon.color }">
      <!-- Google Drive Icon -->
      <svg v-if="item.downloadType === 'gdrive'" viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
        <path d="M7.71 3.5L1.15 15l3.43 6 6.56-11.5L7.71 3.5zm1.42 0l6.56 11.5H22l-6.57-11.5H9.13zM12 9.17L8.57 15h6.86L12 9.17zm-1.29 6.33l-3.43 6h13.72l3.43-6H10.71z"/>
      </svg>
      <!-- Video -->
      <svg v-else-if="fileTypeIcon.type === 'video'" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polygon points="23 7 16 12 23 17 23 7"></polygon>
        <rect x="1" y="5" width="15" height="14" rx="2" ry="2"></rect>
      </svg>
      <!-- Audio -->
      <svg v-else-if="fileTypeIcon.type === 'audio'" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M9 18V5l12-2v13"></path>
        <circle cx="6" cy="18" r="3"></circle>
        <circle cx="18" cy="16" r="3"></circle>
      </svg>
      <!-- Image -->
      <svg v-else-if="fileTypeIcon.type === 'image'" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
        <circle cx="8.5" cy="8.5" r="1.5"></circle>
        <polyline points="21 15 16 10 5 21"></polyline>
      </svg>
      <!-- Document -->
      <svg v-else-if="fileTypeIcon.type === 'document'" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
        <polyline points="14 2 14 8 20 8"></polyline>
        <line x1="16" y1="13" x2="8" y2="13"></line>
        <line x1="16" y1="17" x2="8" y2="17"></line>
        <polyline points="10 9 9 9 8 9"></polyline>
      </svg>
      <!-- Archive -->
      <svg v-else-if="fileTypeIcon.type === 'archive'" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="21 8 21 21 3 21 3 8"></polyline>
        <rect x="1" y="3" width="22" height="5"></rect>
        <line x1="10" y1="12" x2="14" y2="12"></line>
      </svg>
      <!-- Executable -->
      <svg v-else-if="fileTypeIcon.type === 'executable'" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <rect x="4" y="4" width="16" height="16" rx="2" ry="2"></rect>
        <rect x="9" y="9" width="6" height="6"></rect>
        <line x1="9" y1="1" x2="9" y2="4"></line>
        <line x1="15" y1="1" x2="15" y2="4"></line>
        <line x1="9" y1="20" x2="9" y2="23"></line>
        <line x1="15" y1="20" x2="15" y2="23"></line>
        <line x1="20" y1="9" x2="23" y2="9"></line>
        <line x1="20" y1="14" x2="23" y2="14"></line>
        <line x1="1" y1="9" x2="4" y2="9"></line>
        <line x1="1" y1="14" x2="4" y2="14"></line>
      </svg>
      <!-- Code -->
      <svg v-else-if="fileTypeIcon.type === 'code'" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="16 18 22 12 16 6"></polyline>
        <polyline points="8 6 2 12 8 18"></polyline>
      </svg>
      <!-- Default File -->
      <svg v-else xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
        <polyline points="14 2 14 8 20 8"></polyline>
      </svg>
    </div>
    
    <!-- Content -->
    <div class="item-content">
      <div class="item-header">
        <h4 class="filename" :title="item.filename">{{ item.filename }}</h4>
        <span class="status-badge" :style="{ background: `${statusColor}20`, color: statusColor }">
          {{ item.status }}
        </span>
      </div>
      
      <!-- Progress bar for active downloads -->
      <div v-if="isActive" class="progress-container">
        <div class="progress-bar">
          <div 
            class="progress-fill" 
            :style="{ width: `${progressPercent}%` }"
            :class="{ 'paused': item.status === 'paused' }"
          ></div>
        </div>
        <span class="progress-text">{{ progressPercent }}%</span>
      </div>
      
      <!-- Info row -->
      <div class="info-row">
        <span class="folder-path" :title="item.path">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          </svg>
          {{ folderPath }}
        </span>
        <span class="size-info">{{ sizeText }}</span>
        <span v-if="speedText" class="speed-info">{{ speedText }}</span>
        <span v-if="timeRemaining" class="time-info">{{ timeRemaining }}</span>
      </div>
    </div>
    
    <!-- Actions -->
    <div class="item-actions">
      <!-- Copy Link Button -->
      <button 
        class="action-btn" 
        @click="copyDownloadLink"
        title="Copy download link"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
        </svg>
      </button>
      
      <!-- Active download actions -->
      <template v-if="isActive">
        <button 
          class="action-btn" 
          @click="togglePause" 
          :title="item.status === 'paused' ? 'Resume' : 'Pause'"
        >
          <!-- Pause Icon -->
          <svg v-if="item.status === 'downloading'" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="6" y="4" width="4" height="16"></rect>
            <rect x="14" y="4" width="4" height="16"></rect>
          </svg>
          <!-- Play Icon -->
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="5 3 19 12 5 21 5 3"></polygon>
          </svg>
        </button>
        <button class="action-btn danger" @click="cancelDownload" title="Cancel">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </template>
      
      <!-- Completed download actions -->
      <template v-else-if="item.status === 'completed'">
        <button class="action-btn" @click="openFolder" title="Open Directory">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          </svg>
        </button>
        <button class="action-btn danger" @click="deleteItem" title="Delete">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
          </svg>
        </button>
      </template>
      
      <!-- Failed download actions -->
      <template v-else-if="item.status === 'error'">
        <button class="action-btn danger" @click="deleteItem" title="Remove">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
          </svg>
        </button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.download-item-new {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 18px;
  transition: all 0.2s ease;
}

.download-item-new:hover {
  border-color: var(--accent-color);
  box-shadow: 0 2px 8px var(--accent-color-transparent);
}

.download-item-new.is-completed {
  opacity: 0.85;
}

/* File Icon */
.file-icon {
  width: 48px;
  height: 48px;
  border-radius: 999px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

/* Content */
.item-content {
  flex: 1;
  min-width: 0;
}

.item-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.filename {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.status-badge {
  padding: 2px 10px;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: capitalize;
  flex-shrink: 0;
}

/* Progress */
.progress-container {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.progress-bar {
  flex: 1;
  height: 6px;
  background: var(--bg-tertiary);
  border-radius: 999px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent-color);
  border-radius: 999px;
  transition: width 0.3s ease;
}

.progress-fill.paused {
  background: var(--warning-color);
}

.progress-text {
  font-size: 0.8rem;
  color: var(--text-secondary);
  width: 40px;
  text-align: right;
  flex-shrink: 0;
}

/* Info Row */
.info-row {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 0.8rem;
  color: var(--text-secondary);
}

.folder-path {
  display: flex;
  align-items: center;
  gap: 4px;
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.folder-path svg {
  flex-shrink: 0;
  opacity: 0.7;
}

.size-info {
  color: var(--text-secondary);
}

.speed-info {
  color: var(--accent-color);
  font-weight: 500;
}

.time-info {
  color: var(--text-secondary);
  opacity: 0.7;
}

/* Actions */
.item-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.action-btn {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  border: none;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  filter: brightness(1.2);
}

.action-btn.success {
  background: rgba(52, 211, 153, 0.2);
  color: var(--success-color);
}

.action-btn.success:hover {
  background: rgba(52, 211, 153, 0.3);
}

.action-btn.danger {
  background: rgba(248, 113, 113, 0.1);
  color: var(--error-color);
}

.action-btn.danger:hover {
  background: rgba(248, 113, 113, 0.2);
}
</style>
