// 设置导出/导入的文件读写：配合前端对话框选定的路径落盘 / 读取 JSON 文本。
// 仅做纯文本读写，加密与结构在前端处理（API Key 经 AES-GCM 加密后再写入）。
use std::fs;

/// 将文本写入指定路径（导出配置 JSON）。
#[tauri::command]
pub fn write_text_file(path: String, contents: String) -> Result<(), String> {
    fs::write(&path, contents).map_err(|e| format!("写入文件失败 {path}: {e}"))
}

/// 读取指定路径的文本内容（导入配置 JSON）。
#[tauri::command]
pub fn read_text_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("读取文件失败 {path}: {e}"))
}
