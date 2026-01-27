import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { ref, computed } from "vue";

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
}

export interface StorageInfo {
  total: number;
  used: number;
  free: number;
  mount_point: string;
}

export const useDownloadStore = defineStore("download", () => {
  const downloads = ref<DownloadItem[]>([]);
  const selectedPath = ref<string>("");
  const storageInfo = ref<StorageInfo | null>(null);

  // Getters
  const activeDownloads = computed(() => downloads.value.filter(d => d.status === "downloading"));

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
            status: "downloading"
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
  
  async function selectFolder() {
      const selected = await open({
          directory: true,
          multiple: false,
      });
      
      if (selected) {
          // If multiple=false, it returns string or null (or string[] if multiple=true? check types)
          // Tauri v2 dialog checks...
          // Usually returns string | null for single selection.
          selectedPath.value = selected as string;
          refreshStorage();
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

  return {
    downloads,
    selectedPath,
    storageInfo,
    startDownload,
    pauseDownload,
    resumeDownload,
    selectFolder,
    refreshStorage,
    activeDownloads
  };
});
