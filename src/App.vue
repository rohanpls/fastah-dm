<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useDownloadStore } from "./stores/downloadStore";
import DownloadItemNew from "./components/DownloadItemNew.vue";
import SettingsModal from "./components/SettingsModal.vue";
import NewDownloadModal from "./components/NewDownloadModal.vue";
import UpdateModal from "./components/UpdateModal.vue";

const store = useDownloadStore();
const searchQuery = ref("");
const currentView = ref<'downloads' | 'settings' | 'new-download'>('downloads');

// Initialize theme
watch(() => store.settings?.theme, (newTheme) => {
  const root = document.documentElement;
  if (newTheme === 'light') {
    root.classList.add('light-mode');
    root.classList.remove('dark-mode');
  } else {
    root.classList.add('dark-mode');
    root.classList.remove('light-mode');
  }
}, { immediate: true });

// Filtered downloads for new UI with search
const searchedDownloads = computed(() => {
  let items = store.filteredDownloads;
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    items = items.filter(d => 
      d.filename.toLowerCase().includes(query) || 
      d.url.toLowerCase().includes(query)
    );
  }
  return items;
});

// Stats for header
const downloadStats = computed(() => {
  const active = store.activeDownloads;
  const totalSpeed = active.reduce((sum, d) => sum + d.speed, 0);
  const speedStr = totalSpeed < 1024 * 1024 
    ? `${(totalSpeed / 1024).toFixed(1)} KB/s`
    : `${(totalSpeed / 1024 / 1024).toFixed(1)} MB/s`;
  return {
    count: active.length,
    speed: speedStr
  };
});

onMounted(() => {
  store.init();
  
  // Check for updates on startup if enabled
  setTimeout(() => {
    if (store.settings?.auto_update_enabled) {
      store.checkForUpdates().then((hasUpdate) => {
        if (hasUpdate && store.settings?.silent_updates) {
          // Auto-install silently
          store.downloadAndInstallUpdate().catch(console.error);
        } else if (hasUpdate) {
          // Show update modal
          store.showUpdateModal = true;
        }
      });
    }
  }, 2000);
});

function showSettings() {
  currentView.value = 'settings';
  // Check for updates when opening settings
  if (store.settings?.auto_update_enabled && !store.isCheckingUpdate) {
    store.checkForUpdates();
  }
}

function showNewDownload() {
  currentView.value = 'new-download';
}

function showDownloads() {
  currentView.value = 'downloads';
}

// Sidebar active state helpers
function isLinkActive(status: string) {
  return currentView.value === 'downloads' && store.filterStatus === status;
}
</script>

<template>
  <div class="app-wrapper-new">
    <div class="app-container-new">
      <!-- Sidebar -->
      <aside class="sidebar">
        <div class="sidebar-brand">
          <div class="sidebar-logo">
            <img src="/fdm-logo.png" alt="FastahDM Logo" />
          </div>
          <h1>FastahDM</h1>
        </div>

        <nav class="sidebar-nav">
          <button 
            class="nav-item" 
            :class="{ active: isLinkActive('all') }"
            @click="store.filterStatus = 'all'; showDownloads()"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="3" width="7" height="7"></rect>
              <rect x="14" y="3" width="7" height="7"></rect>
              <rect x="14" y="14" width="7" height="7"></rect>
              <rect x="3" y="14" width="7" height="7"></rect>
            </svg>
            <span>All</span>
          </button>
          <button 
            class="nav-item" 
            :class="{ active: isLinkActive('active') }"
            @click="store.filterStatus = 'active'; showDownloads()"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="7 10 12 15 17 10"></polyline>
              <line x1="12" y1="15" x2="12" y2="3"></line>
            </svg>
            <span>Active</span>
          </button>
          <button 
            class="nav-item" 
            :class="{ active: isLinkActive('completed') }"
            @click="store.filterStatus = 'completed'; showDownloads()"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
              <polyline points="22 4 12 14.01 9 11.01"></polyline>
            </svg>
            <span>Completed</span>
          </button>
        </nav>

        <div class="sidebar-footer">
          <button 
            class="nav-item settings-button" 
            :class="{ active: currentView === 'settings' }"
            @click="showSettings"
          >
            <div class="icon-wrapper">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3"></circle>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
              </svg>
              <span v-if="store.updateAvailable" class="update-badge"></span>
            </div>
            <span>Settings</span>
          </button>
        </div>
      </aside>

      <!-- Main Content -->
      <main class="main-content-new">
        <!-- Header -->
        <header class="content-header" v-if="currentView === 'downloads'">
          <div class="header-info">
            <h2>Downloads</h2>
            <p v-if="downloadStats.count > 0">{{ downloadStats.count }} active tasks running at {{ downloadStats.speed }}</p>
            <p v-else>No active downloads</p>
          </div>
          <button class="btn-new-download" @click="showNewDownload">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
            <span>New Download</span>
          </button>
        </header>

        <!-- Search Bar -->
        <div v-if="currentView === 'downloads'" class="search-bar">
          <div class="search-input-wrapper">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="8"></circle>
              <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
            <input v-model="searchQuery" type="text" placeholder="Search files..." />
          </div>
        </div>

        <!-- Download List -->
        <div v-if="currentView === 'downloads'" class="download-list-new">
            <DownloadItemNew 
              v-for="item in searchedDownloads" 
              :key="item.id" 
              :item="item" 
            />
          
          <div v-if="searchedDownloads.length === 0" class="empty-state-new">
            <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="7 10 12 15 17 10"></polyline>
              <line x1="12" y1="15" x2="12" y2="3"></line>
            </svg>
            <p>No downloads yet</p>
            <span>Click "New Download" to start</span>
          </div>
        </div>

        <!-- Settings Panel -->
        <div v-if="currentView === 'settings'" class="panel-content page-panel">
          <SettingsModal :show="true" @close="showDownloads" />
        </div>

        <!-- New Download Panel -->
        <div v-if="currentView === 'new-download'" class="panel-content page-panel">
          <NewDownloadModal :show="true" @close="showDownloads" />
        </div>
      </main>
    </div>

    <!-- Update Modal (Global Overlay) -->
    <UpdateModal :show="store.showUpdateModal" @close="store.showUpdateModal = false" />
  </div>
</template>

<style scoped>
/* ============================================
   THEMED UI STYLES
   ============================================ */
.app-wrapper-new {
  height: 100vh;
  width: 100%;
}

.app-container-new {
  height: 100vh;
  display: flex;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  transition: background-color 0.3s ease, color 0.3s ease;
}

.sidebar {
  width: 260px;
  height: 100vh;
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  z-index: 100;
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

.sidebar-brand {
  padding: 32px 24px;
  display: flex;
  align-items: center;
  gap: 16px;
  border-bottom: 1px solid var(--border-color);
}

.sidebar-logo {
  width: 44px;
  height: 44px;
  background: var(--bg-tertiary);
  border-radius: 12px;
  padding: 8px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.sidebar-logo img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.sidebar-brand h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--text-primary);
}

.sidebar-nav {
  flex: 1;
  padding: 24px 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  background: transparent;
  border: none;
  border-radius: 12px;
  color: var(--text-secondary);
  font-size: 0.95rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.nav-item:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--accent-color);
  color: white;
}

.nav-item svg {
  flex-shrink: 0;
}

.sidebar-footer {
  padding: 24px 16px;
  border-top: 1px solid var(--border-color);
}

.settings-button .icon-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.update-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  width: 10px;
  height: 10px;
  background-color: var(--error-color);
  border-radius: 50%;
  border: 2px solid var(--bg-secondary);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.8;
    transform: scale(1.1);
  }
}

.main-content-new {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 32px;
  overflow: hidden;
  position: relative;
}

.content-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.header-info h2 {
  margin: 0 0 8px;
  font-size: 2rem;
  font-weight: 700;
  color: var(--text-primary);
}

.header-info p {
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.95rem;
}

.btn-new-download {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 24px;
  background: var(--accent-color);
  border: 1px solid transparent;
  border-radius: 999px;
  color: white;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-new-download:hover {
  filter: brightness(1.1);
  transform: translateY(-1px);
}

.btn-new-download:active {
  transform: translateY(0);
}

.search-bar {
  margin-bottom: 24px;
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-input-wrapper svg {
  position: absolute;
  left: 18px;
  color: var(--text-secondary);
  pointer-events: none;
}

.search-input-wrapper input {
  width: 100%;
  padding: 14px 20px 14px 50px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  color: var(--text-primary);
  font-size: 0.95rem;
  outline: none;
  transition: all 0.2s;
}

.search-input-wrapper input::placeholder {
  color: var(--text-secondary);
  opacity: 0.7;
}

.search-input-wrapper input:focus {
  border-color: var(--accent-color);
  box-shadow: 0 0 0 2px var(--accent-color-transparent);
}

.download-list-new {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-right: 8px;
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.page-panel {
  /* Ensure panels take full height/width available and handle their own scrolling */
  width: 100%;
  height: 100%;
}

.empty-state-new {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  color: var(--text-secondary);
}

.empty-state-new svg {
  opacity: 0.5;
}

.empty-state-new p {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
}

.empty-state-new span {
  font-size: 0.95rem;
  color: var(--text-secondary);
}
</style>

<style>
/* ============================================
   GLOBAL STYLES & VARIABLES
   ============================================ */

:root {
  /* Default Dark Mode */
  --bg-primary: #111827;
  --bg-secondary: #1f2937;
  --bg-tertiary: #374151;
  --text-primary: #f9fafb;
  --text-secondary: #9ca3af;
  --border-color: #374151;
  
  --accent-color: #3b82f6; /* Blue 500 */
  --accent-color-hover: #2563eb; /* Blue 600 */
  --accent-color-transparent: rgba(59, 130, 246, 0.2);
  
  --success-color: #10b981;
  --error-color: #ef4444;
  --warning-color: #f59e0b;

  font-family: 'Montserrat', -apple-system, BlinkMacSystemFont, sans-serif;
  font-size: 16px;
  line-height: 1.5;
  font-weight: 400;
}

.light-mode {
  --bg-primary: #f3f4f6;
  --bg-secondary: #ffffff;
  --bg-tertiary: #f9fafb;
  --text-primary: #111827;
  --text-secondary: #6b7280;
  --border-color: #e5e7eb;
  
  --accent-color: #3b82f6;
  --accent-color-hover: #2563eb;
  --accent-color-transparent: rgba(59, 130, 246, 0.1);
}

/* Montserrat Font Family */
@font-face {
  font-family: 'Montserrat';
  src: url('/fonts/Montserrat/Montserrat-Light.ttf') format('truetype');
  font-weight: 300;
  font-style: normal;
}

@font-face {
  font-family: 'Montserrat';
  src: url('/fonts/Montserrat/Montserrat-Regular.ttf') format('truetype');
  font-weight: 400;
  font-style: normal;
}

@font-face {
  font-family: 'Montserrat';
  src: url('/fonts/Montserrat/Montserrat-Medium.ttf') format('truetype');
  font-weight: 500;
  font-style: normal;
}

@font-face {
  font-family: 'Montserrat';
  src: url('/fonts/Montserrat/Montserrat-SemiBold.ttf') format('truetype');
  font-weight: 600;
  font-style: normal;
}

@font-face {
  font-family: 'Montserrat';
  src: url('/fonts/Montserrat/Montserrat-Bold.ttf') format('truetype');
  font-weight: 700;
  font-style: normal;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  min-height: 100vh;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* Scrollbar styling */
::-webkit-scrollbar {
  width: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--bg-tertiary);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--text-secondary);
}

/* Selection styling */
::selection {
  background: var(--accent-color-transparent);
  color: var(--accent-color);
}
</style>
