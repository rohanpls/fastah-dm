<script setup lang="ts">
import { computed } from 'vue';
import type { DownloadItem } from '../stores/downloadStore';
import { useDownloadStore } from '../stores/downloadStore';

const props = defineProps<{ item: DownloadItem }>();
const store = useDownloadStore();

const percentage = computed(() => {
    if (!props.item.total) return 0;
    return (props.item.downloaded / props.item.total) * 100;
});

const speedStr = computed(() => {
    const s = props.item.speed;
    if (s < 1024) return `${s} B/s`;
    if (s < 1024 * 1024) return `${(s / 1024).toFixed(1)} KB/s`;
    return `${(s / 1024 / 1024).toFixed(1)} MB/s`;
});

function toggle() {
    if (props.item.status === 'downloading') {
        store.pauseDownload(props.item);
    } else if (['paused', 'error'].includes(props.item.status)) {
        store.resumeDownload(props.item);
    }
}
</script>

<template>
  <div class="download-item">
    <div class="info">
        <div class="url" :title="item.url">{{ item.url }}</div>
        <div class="status">
            {{ item.status }} 
            <span v-if="item.status === 'downloading'"> - {{ speedStr }}</span>
             - {{ (item.downloaded / 1024 / 1024).toFixed(1) }} MB / {{ item.total ? (item.total / 1024 / 1024).toFixed(1) + ' MB' : '?' }}
        </div>
    </div>
    <div class="progress-bar">
        <div class="fill" :style="{ width: percentage + '%' }"></div>
    </div>
    <div class="controls">
        <button @click="toggle" :disabled="item.status === 'completed'" class="action-btn">
            {{ item.status === 'downloading' ? 'Pause' : 'Resume' }}
        </button>
    </div>
  </div>
</template>

<style scoped>
.download-item {
    background: #333;
    padding: 12px;
    margin-bottom: 10px;
    border-radius: 8px;
    color: white;
    display: flex;
    flex-direction: column;
    gap: 8px;
}
.url {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 0.95em;
}
.progress-bar {
    height: 8px;
    background: #555;
    border-radius: 4px;
    overflow: hidden;
}
.fill {
    height: 100%;
    background: #2196f3;
    transition: width 0.2s ease-out;
}
.status {
    font-size: 0.8em;
    color: #bbb;
}
.controls {
    display: flex;
    justify-content: flex-end;
}
.action-btn {
    background: #444;
    border: 1px solid #555;
    color: white;
    padding: 6px 12px;
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.2s;
}
.action-btn:hover:not(:disabled) {
    background: #555;
}
.action-btn:disabled {
    opacity: 0.5;
    cursor: default;
}
</style>
