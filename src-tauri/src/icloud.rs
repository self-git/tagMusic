use serde::Serialize;

/// 单个文件的 iCloud 状态，返回给前端（camelCase 对齐 TS 端 ICloudStatus）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ICloudStatus {
    path: String,
    /// 是否为 iCloud 托管文件
    is_ubiquitous: bool,
    /// 是否已下载到本地（非 iCloud 文件恒为 true）
    is_downloaded: bool,
}

/// 批量检查文件的 iCloud 状态。
#[tauri::command]
pub fn check_icloud_status(paths: Vec<String>) -> Vec<ICloudStatus> {
    paths
        .into_iter()
        .map(|path| {
            let is_ubiquitous = platform::is_ubiquitous(&path);
            // 非 iCloud 文件无需下载，直接视为已就绪
            let is_downloaded = if is_ubiquitous {
                platform::is_downloaded(&path)
            } else {
                true
            };
            ICloudStatus {
                path,
                is_ubiquitous,
                is_downloaded,
            }
        })
        .collect()
}

/// 触发 iCloud 文件下载（若尚未下载）。下载是异步的，前端通过轮询 check_icloud_status 获取进度。
#[tauri::command]
pub fn start_icloud_download(path: String) -> Result<(), String> {
    platform::start_download(&path)
}

#[cfg(target_os = "macos")]
mod platform {
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2_foundation::{
        NSFileManager, NSString, NSURLUbiquitousItemDownloadingStatusCurrent,
        NSURLUbiquitousItemDownloadingStatusDownloaded, NSURLUbiquitousItemDownloadingStatusKey,
        NSURL,
    };

    pub fn is_ubiquitous(path: &str) -> bool {
        let Some(url) = NSURL::from_file_path(path) else {
            return false;
        };
        NSFileManager::defaultManager().isUbiquitousItemAtURL(&url)
    }

    pub fn is_downloaded(path: &str) -> bool {
        let Some(url) = NSURL::from_file_path(path) else {
            return true;
        };
        let mut value: Option<Retained<AnyObject>> = None;
        // SAFETY: 传入合法的 key 静态常量与可写出参；失败时保守按已下载处理
        let res = unsafe {
            url.getResourceValue_forKey_error(&mut value, NSURLUbiquitousItemDownloadingStatusKey)
        };
        if res.is_err() {
            return true;
        }
        let Some(obj) = value else {
            return true;
        };
        let Ok(status) = obj.downcast::<NSString>() else {
            return true;
        };
        let status = status.to_string();
        // SAFETY: 读取 Foundation 导出的状态字符串常量
        let current = unsafe { NSURLUbiquitousItemDownloadingStatusCurrent.to_string() };
        let downloaded = unsafe { NSURLUbiquitousItemDownloadingStatusDownloaded.to_string() };
        status == current || status == downloaded
    }

    pub fn start_download(path: &str) -> Result<(), String> {
        let Some(url) = NSURL::from_file_path(path) else {
            return Err(format!("无效的文件路径: {path}"));
        };
        NSFileManager::defaultManager()
            .startDownloadingUbiquitousItemAtURL_error(&url)
            .map_err(|e| e.localizedDescription().to_string())
    }
}

// 非 macOS 平台桩实现：保持跨平台可编译（PRD: 代码层不写 Mac 专属 API 的硬依赖）
#[cfg(not(target_os = "macos"))]
mod platform {
    pub fn is_ubiquitous(_path: &str) -> bool {
        false
    }

    pub fn is_downloaded(_path: &str) -> bool {
        true
    }

    pub fn start_download(_path: &str) -> Result<(), String> {
        Ok(())
    }
}
