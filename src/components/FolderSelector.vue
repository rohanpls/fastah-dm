<script setup lang="ts">
import { useDownloadStore } from '../stores/downloadStore';
import { computed } from 'vue';

const store = useDownloadStore();
const path = computed(() => store.selectedPath);
const storageInfo = computed(() => store.storageInfo);

const usagePercent = computed(() => {
  if (!storageInfo.value) return 0;
  return (storageInfo.value.used / storageInfo.value.total) * 100;
});

const usageColor = computed(() => {
  if (usagePercent.value > 90) return '#ef4444';
  if (usagePercent.value > 75) return '#f59e0b';
  return '#4ade80';
});

async function selectFolder() {
  await store.selectFolder();
}
</script>

<template>
  <div class="folder-selector glass-panel">
    <div class="selector-header">
      <div class="folder-icon">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
      </div>
      
      <div class="folder-info">
        <span class="label">Download Location</span>
        <div v-if="path" class="path-display">{{ path }}</div>
        <div v-else class="path-placeholder">No folder selected</div>
      </div>
      
      <button class="btn btn-select" @click="selectFolder">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
          <polyline points="17 8 12 3 7 8"></polyline>
          <line x1="12" y1="3" x2="12" y2="15"></line>
        </svg>
        {{ path ? 'Change' : 'Select Folder' }}
      </button>
    </div>
    
    <div v-if="storageInfo" class="storage-section">
      <div class="storage-bar">
        <div class="storage-fill" :style="{ width: usagePercent + '%', background: usageColor }"></div>
      </div>
      
      <div class="storage-stats">
        <div class="stat">
          <span class="stat-value" :style="{ color: usageColor }">
            {{ (storageInfo.used / 1024 / 1024 / 1024).toFixed(1) }} GB
          </span>
          <span class="stat-label">Used</span>
        </div>
        <div class="stat">
          <span class="stat-value">
            {{ (storageInfo.free / 1024 / 1024 / 1024).toFixed(1) }} GB
          </span>
          <span class="stat-label">Free</span>
        </div>
        <div class="stat">
          <span class="stat-value">
            {{ (storageInfo.total / 1024 / 1024 / 1024).toFixed(1) }} GB
          </span>
          <span class="stat-label">Total</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.folder-selector {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.selector-header {
  display: flex;
  align-items: center;
  gap: 14px;
}

.folder-icon {
  width: 44px;
  height: 44px;
  background: rgba(168, 85, 247, 0.3);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #a855f7;
  flex-shrink: 0;
}

.folder-info {
  flex: 1;
  min-width: 0;
}

.label {
  font-size: 0.75em;
  color: rgba(255, 255, 255, 0.5);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 500;
}

.path-display {
  font-size: 0.9em;
  color: white;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

.path-placeholder {
  font-size: 0.9em;
  color: rgba(255, 255, 255, 0.4);
  margin-top: 2px;
}

.btn-select {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  border-radius: 999px;
  font-size: 0.85em;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid rgba(168, 85, 247, 0.3);
  background: rgba(168, 85, 247, 0.15);
  color: #c084fc;
  flex-shrink: 0;
}

.btn-select:hover {
  background: rgba(168, 85, 247, 0.25);
  border-color: rgba(168, 85, 247, 0.5);
  transform: translateY(-1px);
}

.btn-select:active {
  transform: translateY(0) scale(0.98);
}

.storage-section {
  padding-top: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.storage-bar {
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 12px;
}

.storage-fill {
  height: 100%;
  border-radius: 3px;
  transition: all 0.5s ease-out;
}

.storage-stats {
  display: flex;
  justify-content: space-between;
  gap: 20px;
}

.stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.stat-value {
  font-size: 1em;
  font-weight: 600;
  color: white;
}

.stat-label {
  font-size: 0.7em;
  color: rgba(255, 255, 255, 0.4);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-top: 2px;
}
</style>
