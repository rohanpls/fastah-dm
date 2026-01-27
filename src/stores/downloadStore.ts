import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { remove } from "@tauri-apps/plugin-fs";
import { check, Update } from "@tauri-apps/plugin-updater";
import { ref, computed, watch } from "vue";

export interface DownloadItem {
  id: string;
  url: string;
  path: string; // Folder path
  filename: string;
  total: number | null;
  downloaded: number;
  speed: number;
  status: "pending" | "downloading" | "paused" | "error" | "completed";
  error?: string;
  etag?: string;
  createdAt?: string;
  updatedAt?: string;
}

export interface StorageInfo {
  total: number;
  used: number;
  free: number;
  mount_point: string;
}

export interface AppSettings {
  wallpaper_url: string | null;
  theme: 'light' | 'dark' | null;
  default_download_path: string | null;
  author: string;
  launch_on_startup: boolean;
  toggle_keybind: string | null;
  use_new_ui: boolean;
  auto_update_enabled: boolean;
  silent_updates: boolean;
}

interface DownloadHistoryItem {
  id: string;
  url: string;
  path: string;
  filename: string;
  total: number | null;
  downloaded: number;
  status: string;
  etag: string | null;
  created_at: string;
  updated_at: string;
}

interface DownloadHistory {
  items: DownloadHistoryItem[];
}

export const useDownloadStore = defineStore("download", () => {
  const downloads = ref<DownloadItem[]>([]);
  const selectedPath = ref<string>("");
  const storageInfo = ref<StorageInfo | null>(null);
  const settings = ref<AppSettings | null>(null);
  const initialized = ref(false);
  const filterStatus = ref<'all' | 'active' | 'completed'>('all');

  // Update state
  const updateAvailable = ref(false);
  const updateInfo = ref<Update | null>(null);
  const isCheckingUpdate = ref(false);
  const updateDownloadProgress = ref(0);
  const showUpdateModal = ref(false);

  // Getters
  const activeDownloads = computed(() => downloads.value.filter(d => d.status === "downloading"));
  const completedDownloads = computed(() => downloads.value.filter(d => d.status === "completed"));
  
  // Filtered downloads based on current filter
  const filteredDownloads = computed(() => {
    switch (filterStatus.value) {
      case 'active':
        return downloads.value.filter(d => ['downloading', 'paused', 'pending', 'error'].includes(d.status));
      case 'completed':
        return downloads.value.filter(d => d.status === 'completed');
      default:
        return downloads.value;
    }
  });

  // Initialize store - load settings and history
  async function init() {
    if (initialized.value) return;
    
    try {
      // Load settings
      settings.value = await invoke<AppSettings>("load_settings");
      
      // Load download history
      const history = await invoke<DownloadHistory>("load_download_history");
      if (history.items && history.items.length > 0) {
        downloads.value = history.items.map(item => ({
          id: item.id,
          url: item.url,
          path: item.path,
          filename: item.filename,
          total: item.total,
          downloaded: item.downloaded,
          speed: 0,
          status: item.status as DownloadItem["status"],
          etag: item.etag || undefined,
          createdAt: item.created_at,
          updatedAt: item.updated_at
        }));
      }
      
      // Restore default download path if set
      if (settings.value?.default_download_path) {
        selectedPath.value = settings.value.default_download_path;
        refreshStorage();
      }
      
      initialized.value = true;
    } catch (e) {
      console.error("Failed to initialize store:", e);
    }
  }

  // Watch downloads and save history on changes
  watch(downloads, async () => {
    if (!initialized.value) return;
    await saveHistory();
  }, { deep: true });

  // Save download history
  async function saveHistory() {
    try {
      const history: DownloadHistory = {
        items: downloads.value.map(d => ({
          id: d.id,
          url: d.url,
          path: d.path,
          filename: d.filename,
          total: d.total,
          downloaded: d.downloaded,
          status: d.status,
          etag: d.etag || null,
          created_at: d.createdAt || new Date().toISOString(),
          updated_at: new Date().toISOString()
        }))
      };
      await invoke("save_download_history", { history });
    } catch (e) {
      console.error("Failed to save history:", e);
    }
  }

  // Update settings
  async function updateSettings(newSettings: Partial<AppSettings>) {
    try {
      const updated = {
        ...settings.value,
        ...newSettings,
        author: "@rohanpls"
      } as AppSettings;
      
      await invoke("save_settings", { settings: updated });
      settings.value = updated;
    } catch (e) {
      console.error("Failed to save settings:", e);
      throw e;
    }
  }

  // Listeners
  listen<any>("download://progress", (event) => {
    const { id, downloaded, total, speed } = event.payload;
    const item = downloads.value.find((d) => d.id === id);
    if (item) {
      item.downloaded = downloaded;
      item.total = total;
      item.speed = speed;
      if (item.status !== "paused" && item.status !== "error") {
        item.status = "downloading";
      }
    }
  });

  listen<string>("download://complete", (event) => {
    const id = event.payload;
    const item = downloads.value.find((d) => d.id === id);
    if (item) {
      item.status = "completed";
      item.speed = 0;
      refreshStorage();
    }
  });

  listen<any>("download://error", (event) => {
    const [id, error] = event.payload;
    const item = downloads.value.find((d) => d.id === id);
    if (item) {
      item.status = "error";
      item.error = error;
      item.speed = 0;
    }
  });
  
  listen<string>("download://paused", (event) => {
     const id = event.payload;
     const item = downloads.value.find(d => d.id === id);
     if(item) {
         item.status = "paused";
         item.speed = 0;
     }
  });

  // Actions
  async function startDownload(url: string, filename: string) {
    if (!selectedPath.value) throw new Error("No folder selected");
    
    // Construct full path (basic)
    const sep = navigator.userAgent.includes("Windows") ? "\\" : "/";
    const fullPath = `${selectedPath.value}${sep}${filename}`; 

    try {
        const id = await invoke<string>("download_file", { url, savePath: fullPath });
        
        downloads.value.push({
            id,
            url,
            path: selectedPath.value,
            filename,
            total: null,
            downloaded: 0,
            speed: 0,
            status: "downloading",
            createdAt: new Date().toISOString()
        });
    } catch (e: any) {
        console.error("Failed to start", e);
        throw e;
    }
  }

  async function pauseDownload(item: DownloadItem) {
      try {
          await invoke("pause_download", { id: item.id });
          // Event listener will update status, but we can optimistically set it
          item.status = "paused";
      } catch (e) {
          console.error(e);
      }
  }

  async function resumeDownload(item: DownloadItem) {
      const sep = navigator.userAgent.includes("Windows") ? "\\" : "/";
      const fullPath = `${item.path}${sep}${item.filename}`;

      try {
          const newId = await invoke<string>("download_file", { url: item.url, savePath: fullPath });
          item.id = newId;
          item.status = "downloading";
          item.error = undefined;
      } catch (e: any) {
          item.error = e.toString();
          item.status = "error";
      }
  }

  async function removeDownload(item: DownloadItem, deleteFile: boolean = false) {
    // If downloading, pause first
    if (item.status === "downloading") {
      try {
        await invoke("pause_download", { id: item.id });
      } catch (e) {
        console.error("Failed to pause before removing:", e);
      }
    }
    
    // Optionally delete the file
    if (deleteFile) {
      const sep = navigator.userAgent.includes("Windows") ? "\\" : "/";
      const fullPath = `${item.path}${sep}${item.filename}`;
      try {
        await remove(fullPath);
      } catch (e) {
        console.error("Failed to delete file:", e);
      }
    }
    
    // Remove from list
    const index = downloads.value.findIndex(d => d.id === item.id);
    if (index !== -1) {
      downloads.value.splice(index, 1);
    }
  }

  async function clearAllHistory() {
    // Pause all active downloads first
    for (const item of downloads.value) {
      if (item.status === "downloading") {
        try {
          await invoke("pause_download", { id: item.id });
        } catch (e) {
          console.error(e);
        }
      }
    }
    
    downloads.value = [];
    
    try {
      await invoke("clear_download_history");
    } catch (e) {
      console.error("Failed to clear history:", e);
    }
  }
  
  async function selectFolder() {
      const selected = await open({
          directory: true,
          multiple: false,
      });
      
      if (selected) {
          selectedPath.value = selected as string;
          refreshStorage();
          
          // Save as default path
          if (settings.value) {
            updateSettings({ default_download_path: selected as string });
          }
      }
  }
  
  async function refreshStorage() {
      if (!selectedPath.value) return;
      try {
          const info = await invoke<StorageInfo>("get_system_storage", { path: selectedPath.value });
          storageInfo.value = info;
      } catch (e) {
          console.error(e);
      }
  }

  // Check if a file exists at the given path
  async function checkFileExists(filename: string): Promise<boolean> {
    if (!selectedPath.value) return false;
    const sep = navigator.userAgent.includes("Windows") ? "\\" : "/";
    const fullPath = `${selectedPath.value}${sep}${filename}`;
    try {
      return await invoke<boolean>("file_exists", { path: fullPath });
    } catch (e) {
      console.error("Failed to check file existence:", e);
      return false;
    }
  }

  // Generate a unique filename with suffix (e.g., file(1).txt, file(2).txt)
  function generateUniqueFilename(filename: string, suffix: number): string {
    const lastDot = filename.lastIndexOf(".");
    if (lastDot === -1) {
      return `${filename}(${suffix})`;
    }
    const name = filename.substring(0, lastDot);
    const ext = filename.substring(lastDot);
    return `${name}-${suffix}${ext}`;
  }

  // Find next available filename
  async function findAvailableFilename(filename: string): Promise<string> {
    let currentFilename = filename;
    let suffix = 1;
    
    while (await checkFileExists(currentFilename)) {
      currentFilename = generateUniqueFilename(filename, suffix);
      suffix++;
      if (suffix > 100) break; // Safety limit
    }
    
    return currentFilename;
  }

  // Check for updates
  async function checkForUpdates(): Promise<boolean> {
    if (isCheckingUpdate.value) return false;
    
    isCheckingUpdate.value = true;
    updateAvailable.value = false;
    updateInfo.value = null;
    
    try {
      const update = await check();
      
      if (update) {
        updateAvailable.value = true;
        updateInfo.value = update;
        console.log(`Update available: ${update.version} (current: ${update.currentVersion})`);
        return true;
      } else {
        console.log("No updates available");
        return false;
      }
    } catch (error) {
      console.error("Failed to check for updates:", error);
      return false;
    } finally {
      isCheckingUpdate.value = false;
    }
  }

  // Download and install update
  async function downloadAndInstallUpdate() {
    if (!updateInfo.value) return;
    
    try {
      updateDownloadProgress.value = 0;
      
      // Download and install (will automatically restart the app)
      await updateInfo.value.downloadAndInstall();
      
      updateDownloadProgress.value = 100;
      console.log("Update installed, app will restart...");
    } catch (error) {
      console.error("Failed to install update:", error);
      throw error;
    }
  }

  return {
    downloads,
    selectedPath,
    storageInfo,
    settings,
    filterStatus,
    updateAvailable,
    updateInfo,
    isCheckingUpdate,
    updateDownloadProgress,
    showUpdateModal,
    init,
    startDownload,
    pauseDownload,
    resumeDownload,
    removeDownload,
    clearAllHistory,
    selectFolder,
    refreshStorage,
    updateSettings,
    activeDownloads,
    completedDownloads,
    filteredDownloads,
    checkFileExists,
    generateUniqueFilename,
    findAvailableFilename,
    checkForUpdates,
    downloadAndInstallUpdate
  };
});
