use sysinfo::Disks;
use std::path::Path;
use serde::Serialize;

#[derive(Serialize)]
pub struct StorageInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub mount_point: String,
}

pub fn get_disk_info(path_str: &str) -> Option<StorageInfo> {
    let disks = Disks::new_with_refreshed_list();
    let path = Path::new(path_str);

    // Find the disk that contains the path
    // We look for the longest matching mount point
    let best_match = disks.list().iter()
        .filter(|disk| path.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len());

    if let Some(disk) = best_match {
        Some(StorageInfo {
            total: disk.total_space(),
            used: disk.total_space() - disk.available_space(),
            free: disk.available_space(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
        })
    } else {
        None
    }
}
