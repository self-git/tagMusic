use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

// 仅传文件名给 LLM（PRD 5.3：不上传音频内容）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseInput {
    path: String,
    file_name: String,
}

/// LLM provider 配置。provider_type 为协议类型：
/// "openai"（覆盖 DeepSeek / OpenAI Compatible）| "anthropic"（Anthropic 兼容）。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    provider_type: String,
    base_url: String,
    api_key: String,
    model: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    path: String,
    title: Option<String>,
    album: Option<String>,
    artist: Option<String>,
    track: Option<u32>,
    confidence: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct LlmItem {
    index: usize,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    album: Option<String>,
    #[serde(default)]
    artist: Option<String>,
    #[serde(default)]
    track: Option<u32>,
    #[serde(default)]
    confidence: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct LlmResults {
    results: Vec<LlmItem>,
}

const SYSTEM_PROMPT: &str = "你是播客元数据提取助手。从用户给出的脏文件名中提取四个字段：title(标题)、album(节目名)、artist(作者)、track(集数，整数或 null)。\n\
要点：\n\
- 剥离平台/网站标识（如 爱发电、知乎、喜马拉雅、小宇宙、b站、公众号 等）及其分隔符（丨 | - · _ [ ] （） 等）。\n\
- title 取核心标题，不含平台后缀。\n\
- 无法确定的字段返回 null，不要臆造。\n\
- track 仅在文件名含明确集数编号时填整数。\n\
- confidence 为 0~1 的置信度。\n\
只输出 JSON 对象，不要任何多余文字或 markdown。";

// few-shot 示例 + 待解析列表，要求返回 {"results":[...]} 结构
fn build_user_prompt(names: &[&str]) -> String {
    let mut s = String::from(
        "示例：\n\
        输入 `QA009：香港金像奖·国产片含男量丨反派影评丨爱发电.mp3`\n\
        输出 {\"index\":0,\"title\":\"QA009：香港金像奖·国产片含男量\",\"album\":\"反派影评\",\"artist\":null,\"track\":9,\"confidence\":0.86}\n\n\
        请按相同规则处理下列文件名（每行为「序号: 文件名」），\
        返回 JSON 对象 {\"results\":[{\"index\":<序号>,\"title\":...,\"album\":...,\"artist\":...,\"track\":...,\"confidence\":...}]}：\n\n",
    );
    for (i, n) in names.iter().enumerate() {
        s.push_str(&format!("{i}: {n}\n"));
    }
    s
}

// 从可能含 markdown 包裹的文本中截取 JSON 对象主体
fn extract_json(s: &str) -> &str {
    match (s.find('{'), s.rfind('}')) {
        (Some(a), Some(b)) if b >= a => &s[a..=b],
        _ => s,
    }
}

// 以原始文件顺序初始化结果，按 LLM 返回的 index 回填，避免乱序/缺失/越界
fn assemble_results(paths: &[String], parsed: LlmResults) -> Vec<ParseResult> {
    let mut out: Vec<ParseResult> = paths
        .iter()
        .map(|p| ParseResult {
            path: p.clone(),
            title: None,
            album: None,
            artist: None,
            track: None,
            confidence: None,
        })
        .collect();

    for item in parsed.results {
        if let Some(slot) = out.get_mut(item.index) {
            slot.title = item.title;
            slot.album = item.album;
            slot.artist = item.artist;
            slot.track = item.track;
            slot.confidence = item.confidence;
        }
    }
    out
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())
}

async fn call_openai(cfg: &ProviderConfig, names: &[&str]) -> Result<String, String> {
    let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
    let body = json!({
        "model": cfg.model,
        "temperature": 0,
        "response_format": { "type": "json_object" },
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": build_user_prompt(names) }
        ]
    });
    let resp = http_client()?
        .post(&url)
        .bearer_auth(&cfg.api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("LLM 请求失败 {status}: {text}"));
    }
    let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    v["choices"][0]["message"]["content"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "LLM 返回缺少 choices[0].message.content".to_string())
}

async fn call_anthropic(cfg: &ProviderConfig, names: &[&str]) -> Result<String, String> {
    let url = format!("{}/v1/messages", cfg.base_url.trim_end_matches('/'));
    let body = json!({
        "model": cfg.model,
        "max_tokens": 4096,
        "system": SYSTEM_PROMPT,
        "messages": [ { "role": "user", "content": build_user_prompt(names) } ]
    });
    let resp = http_client()?
        .post(&url)
        .header("x-api-key", &cfg.api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("LLM 请求失败 {status}: {text}"));
    }
    let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    v["content"][0]["text"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "LLM 返回缺少 content[0].text".to_string())
}

/// 批量解析脏文件名 → 元数据。所有文件名一次性送给 LLM（满足 30 文件/60 秒目标）。
#[tauri::command]
pub async fn parse_filenames(
    files: Vec<ParseInput>,
    config: ProviderConfig,
) -> Result<Vec<ParseResult>, String> {
    if files.is_empty() {
        return Ok(vec![]);
    }
    let names: Vec<&str> = files.iter().map(|f| f.file_name.as_str()).collect();

    let content = match config.provider_type.as_str() {
        "anthropic" => call_anthropic(&config, &names).await?,
        _ => call_openai(&config, &names).await?,
    };

    let parsed: LlmResults = serde_json::from_str(extract_json(&content))
        .map_err(|e| format!("解析 LLM 返回 JSON 失败: {e}; 原文: {content}"))?;

    let paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    Ok(assemble_results(&paths, parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    // 纯 JSON、markdown 代码块包裹、前后带说明文字三种情况都应截出对象主体
    #[test]
    fn extract_json_strips_wrappers() {
        assert_eq!(extract_json(r#"{"a":1}"#), r#"{"a":1}"#);
        assert_eq!(extract_json("```json\n{\"a\":1}\n```"), "{\"a\":1}");
        assert_eq!(
            extract_json("这是结果：{\"results\":[]} 完毕"),
            "{\"results\":[]}"
        );
    }

    // 无花括号时原样返回（交由后续 JSON 解析报错）
    #[test]
    fn extract_json_passthrough_when_no_braces() {
        assert_eq!(extract_json("no json here"), "no json here");
    }

    fn paths() -> Vec<String> {
        vec!["/a.mp3".into(), "/b.mp3".into(), "/c.mp3".into()]
    }

    // mock LLM 返回：乱序 index 应按 index 精确回填到对应文件
    #[test]
    fn assemble_backfills_by_index_out_of_order() {
        let content = r#"{"results":[
            {"index":2,"title":"标题C","album":"节目","artist":null,"track":3,"confidence":0.9},
            {"index":0,"title":"标题A","album":null,"artist":"作者A","track":null,"confidence":0.5}
        ]}"#;
        let parsed: LlmResults = serde_json::from_str(extract_json(content)).unwrap();
        let out = assemble_results(&paths(), parsed);

        assert_eq!(out.len(), 3);
        assert_eq!(out[0].path, "/a.mp3");
        assert_eq!(out[0].title.as_deref(), Some("标题A"));
        assert_eq!(out[0].artist.as_deref(), Some("作者A"));
        assert_eq!(out[2].title.as_deref(), Some("标题C"));
        assert_eq!(out[2].track, Some(3));
        // 未被任何 item 命中的 index 1 保持全 None
        assert_eq!(out[1].path, "/b.mp3");
        assert!(out[1].title.is_none() && out[1].track.is_none());
    }

    // 越界 index 应被安全忽略，不 panic、不影响其他项
    #[test]
    fn assemble_ignores_out_of_bounds_index() {
        let content = r#"{"results":[
            {"index":99,"title":"越界","album":null,"artist":null,"track":null,"confidence":null},
            {"index":1,"title":"标题B","album":null,"artist":null,"track":null,"confidence":null}
        ]}"#;
        let parsed: LlmResults = serde_json::from_str(content).unwrap();
        let out = assemble_results(&paths(), parsed);

        assert_eq!(out.len(), 3);
        assert_eq!(out[1].title.as_deref(), Some("标题B"));
        assert!(out[0].title.is_none() && out[2].title.is_none());
    }

    // 系统提示词与 few-shot 用户提示词应包含解析规则要点与待处理列表
    #[test]
    fn user_prompt_contains_names_and_indices() {
        let names = ["脏文件名一.mp3", "脏文件名二.m4a"];
        let prompt = build_user_prompt(&names);
        assert!(prompt.contains("0: 脏文件名一.mp3"));
        assert!(prompt.contains("1: 脏文件名二.m4a"));
        assert!(prompt.contains("results"));
    }
}
