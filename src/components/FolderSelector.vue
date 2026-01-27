<script setup lang="ts">
import { useDownloadStore } from '../stores/downloadStore';
import { computed } from 'vue';

const store = useDownloadStore();
const path = computed(() => store.selectedPath);
const storageInfo = computed(() => store.storageInfo);

async function selectFolder() {
  await store.selectFolder();
}
</script>

<template>
  <div class="folder-selector">
    <button @click="selectFolder">Select Folder</button>
    <div v-if="path" class="path-display">{{ path }}</div>
    <div v-if="storageInfo" class="storage-info">
        <span>Free: {{ (storageInfo.free / 1024 / 1024 / 1024).toFixed(2) }} GB</span>
        <span>Total: {{ (storageInfo.total / 1024 / 1024 / 1024).toFixed(2) }} GB</span>
        <div class="bar">
            <div class="fill" :style="{ width: ((storageInfo.used / storageInfo.total) * 100) + '%' }"></div>
        </div>
    </div>
  </div>
</template>

<style scoped>
.folder-selector {
    border: 1px solid #444;
    padding: 15px;
    border-radius: 8px;
    background: #2a2a2a;
    color: white;
}
.path-display {
    margin: 5px 0;
    font-size: 0.9em;
    color: #aaa;
    word-break: break-all;
}
.storage-info {
    margin-top: 10px;
}
.bar {
    height: 6px;
    background: #555;
    border-radius: 3px;
    margin-top: 5px;
    overflow: hidden;
}
.fill {
    height: 100%;
    background: #4caf50;
}
button {
    background: #007acc;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
}
</style>
