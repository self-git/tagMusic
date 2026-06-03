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

/// 用户可自定义的解析配置（PRD v2 B 项）：留空/缺省时回落到默认常量。
/// 仅放开 system prompt / few-shot / temperature；结构契约（results 数组）仍由代码掌控。
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParseConfig {
    #[serde(default)]
    system_prompt: Option<String>,
    #[serde(default)]
    few_shot: Option<String>,
    #[serde(default)]
    temperature: Option<f64>,
}

// 字符串覆盖：仅当非空白时采用用户值，否则回落默认
fn resolved<'a>(opt: &'a Option<String>, default: &'a str) -> &'a str {
    match opt {
        Some(s) if !s.trim().is_empty() => s.as_str(),
        _ => default,
    }
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

// 默认 system prompt（用户可在设置中覆盖，留空回落于此）
pub const DEFAULT_SYSTEM_PROMPT: &str = "你是播客元数据提取助手。从用户给出的脏文件名中提取四个字段：title(标题)、album(节目名)、artist(作者)、track(集数，整数或 null)。\n\
要点：\n\
- 剥离平台/网站标识（如 爱发电、知乎、喜马拉雅、小宇宙、b站、公众号 等）及其分隔符（丨 | - · _ [ ] （） 等）。\n\
- title 取核心标题，不含平台后缀。\n\
- 无法确定的字段返回 null，不要臆造。\n\
- track 仅在文件名含明确集数编号时填整数。\n\
- confidence 为 0~1 的置信度。\n\
只输出 JSON 对象，不要任何多余文字或 markdown。";

// 默认 few-shot 示例（用户可在设置中覆盖，留空回落于此）
pub const DEFAULT_FEW_SHOT: &str = "示例：\n\
输入 `QA009：香港金像奖·国产片含男量丨反派影评丨爱发电.mp3`\n\
输出 {\"index\":0,\"title\":\"QA009：香港金像奖·国产片含男量\",\"album\":\"反派影评\",\"artist\":null,\"track\":9,\"confidence\":0.86}";

// 默认解析温度（用户可覆盖）
pub const DEFAULT_TEMPERATURE: f64 = 0.0;

// few-shot 示例（可自定义）+ 代码固定的结构契约 + 待解析列表，要求返回 {"results":[...]} 结构。
// 结构指令与列表恒由代码控制，防止用户改坏 index 回填链路（PRD v2 B3）。
fn build_user_prompt(few_shot: &str, names: &[&str]) -> String {
    let mut s = String::with_capacity(few_shot.len() + names.len() * 32 + 256);
    s.push_str(few_shot);
    s.push_str(
        "\n\n请按相同规则处理下列文件名（每行为「序号: 文件名」），\
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

async fn call_openai(
    cfg: &ProviderConfig,
    system: &str,
    user: &str,
    temperature: f64,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
    let body = json!({
        "model": cfg.model,
        "temperature": temperature,
        "response_format": { "type": "json_object" },
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
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

async fn call_anthropic(
    cfg: &ProviderConfig,
    system: &str,
    user: &str,
    temperature: f64,
) -> Result<String, String> {
    let url = format!("{}/v1/messages", cfg.base_url.trim_end_matches('/'));
    let body = json!({
        "model": cfg.model,
        "max_tokens": 4096,
        "temperature": temperature,
        "system": system,
        "messages": [ { "role": "user", "content": user } ]
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

// 按 provider 协议分发一次 chat 调用，返回模型输出文本
async fn chat(
    cfg: &ProviderConfig,
    system: &str,
    user: &str,
    temperature: f64,
) -> Result<String, String> {
    match cfg.provider_type.as_str() {
        "anthropic" => call_anthropic(cfg, system, user, temperature).await,
        _ => call_openai(cfg, system, user, temperature).await,
    }
}

/// 批量解析脏文件名 → 元数据。所有文件名一次性送给 LLM（满足 30 文件/60 秒目标）。
#[tauri::command]
pub async fn parse_filenames(
    files: Vec<ParseInput>,
    config: ProviderConfig,
    parse_config: Option<ParseConfig>,
) -> Result<Vec<ParseResult>, String> {
    if files.is_empty() {
        return Ok(vec![]);
    }
    let names: Vec<&str> = files.iter().map(|f| f.file_name.as_str()).collect();

    let pc = parse_config.unwrap_or_default();
    let system = resolved(&pc.system_prompt, DEFAULT_SYSTEM_PROMPT);
    let few_shot = resolved(&pc.few_shot, DEFAULT_FEW_SHOT);
    let temperature = pc.temperature.unwrap_or(DEFAULT_TEMPERATURE);
    // 是否使用了自定义提示词（决定解析失败时是否追加"恢复默认"引导，PRD v2 B5）
    let customized = pc.system_prompt.is_some() || pc.few_shot.is_some();

    let user = build_user_prompt(few_shot, &names);
    let content = chat(&config, system, &user, temperature).await?;

    let parsed: LlmResults = serde_json::from_str(extract_json(&content)).map_err(|e| {
        let mut msg = format!("解析 LLM 返回 JSON 失败: {e}; 原文: {content}");
        if customized {
            msg.push_str("\n提示：当前使用了自定义解析提示词，可在设置中恢复默认提示词后重试。");
        }
        msg
    })?;

    let paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    Ok(assemble_results(&paths, parsed))
}

// ===== 封面 AI 文本匹配（v2 A2）：仅用候选图片文件名 + 已解析 title/album，不读像素 =====

// 低于此置信度视为"无明显匹配"，留空不写封面（PRD v2 A4）
const COVER_MATCH_MIN_CONFIDENCE: f64 = 0.5;

const COVER_SYSTEM_PROMPT: &str =
    "你是封面匹配助手。为每个音频从其候选封面图片中选出最匹配的一张。\n\
判断依据：候选图片的文件名与音频的标题(title)、节目名(album)的相关度。\n\
- 若某候选文件名明显对应该音频（如同名、含集数、含节目名）优先选它。\n\
- 通用名（cover/folder/封面）作为节目级封面可作次选。\n\
- 若没有任何明显匹配，chosen 返回 null。\n\
只输出 JSON 对象，不要任何多余文字或 markdown。";

/// 单个音频的封面匹配输入：已解析的 title/album + 候选图片完整路径列表
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverMatchInput {
    path: String,
    title: Option<String>,
    album: Option<String>,
    candidates: Vec<String>,
}

/// 封面匹配结果：选中的图片完整路径（无匹配为 null）+ 置信度
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverMatchResult {
    path: String,
    chosen: Option<String>,
    confidence: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct LlmCoverItem {
    index: usize,
    #[serde(default)]
    chosen: Option<usize>,
    #[serde(default)]
    confidence: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct LlmCoverResults {
    results: Vec<LlmCoverItem>,
}

fn basename(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

// 列出待匹配音频 + 其候选图片文件名（带序号），要求返回 {results:[{index,chosen,confidence}]}
fn build_cover_prompt(items: &[CoverMatchInput]) -> String {
    let mut s = String::from(
        "为下列每个音频，从其候选封面中选出最匹配的一张。\
        返回 JSON 对象 {\"results\":[{\"index\":<音频序号>,\"chosen\":<候选序号|null>,\"confidence\":0~1}]}：\n\n",
    );
    for (i, item) in items.iter().enumerate() {
        let title = item.title.as_deref().unwrap_or("(无)");
        let album = item.album.as_deref().unwrap_or("(无)");
        s.push_str(&format!(
            "音频 {i}：标题「{title}」节目「{album}」\n  候选：\n"
        ));
        for (ci, cand) in item.candidates.iter().enumerate() {
            s.push_str(&format!("    {ci}: {}\n", basename(cand)));
        }
    }
    s
}

// 以原始顺序初始化结果，按 LLM 返回的 index/chosen 回填为完整图片路径；越界/低置信留空
fn assemble_cover_results(
    items: &[CoverMatchInput],
    parsed: LlmCoverResults,
) -> Vec<CoverMatchResult> {
    let mut out: Vec<CoverMatchResult> = items
        .iter()
        .map(|it| CoverMatchResult {
            path: it.path.clone(),
            chosen: None,
            confidence: None,
        })
        .collect();

    for item in parsed.results {
        let Some(input) = items.get(item.index) else {
            continue;
        };
        let slot = &mut out[item.index];
        slot.confidence = item.confidence;
        let confident = item
            .confidence
            .map(|c| c >= COVER_MATCH_MIN_CONFIDENCE)
            .unwrap_or(false);
        if let Some(ci) = item.chosen {
            if confident {
                if let Some(p) = input.candidates.get(ci) {
                    slot.chosen = Some(p.clone());
                }
            }
        }
    }
    out
}

/// 为多个音频批量匹配封面（纯文本，复用解析的 provider 配置）。无候选的音频直接留空。
#[tauri::command]
pub async fn match_covers(
    items: Vec<CoverMatchInput>,
    config: ProviderConfig,
) -> Result<Vec<CoverMatchResult>, String> {
    // 过滤出有候选图片的音频；全无候选则直接返回空匹配
    let with_candidates: Vec<&CoverMatchInput> = items
        .iter()
        .filter(|it| !it.candidates.is_empty())
        .collect();
    if with_candidates.is_empty() {
        return Ok(items
            .iter()
            .map(|it| CoverMatchResult {
                path: it.path.clone(),
                chosen: None,
                confidence: None,
            })
            .collect());
    }

    let user = build_cover_prompt(&items);
    let content = chat(&config, COVER_SYSTEM_PROMPT, &user, 0.0).await?;
    let parsed: LlmCoverResults = serde_json::from_str(extract_json(&content))
        .map_err(|e| format!("解析封面匹配 JSON 失败: {e}; 原文: {content}"))?;
    Ok(assemble_cover_results(&items, parsed))
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

    // few-shot 用户提示词应包含 few-shot 段、结构契约与待处理列表
    #[test]
    fn user_prompt_contains_names_and_indices() {
        let names = ["脏文件名一.mp3", "脏文件名二.m4a"];
        let prompt = build_user_prompt(DEFAULT_FEW_SHOT, &names);
        assert!(prompt.contains("0: 脏文件名一.mp3"));
        assert!(prompt.contains("1: 脏文件名二.m4a"));
        assert!(prompt.contains("results"));
        // 结构契约指令恒由代码注入
        assert!(prompt.contains("序号: 文件名"));
    }

    // 自定义 few-shot 应替换默认段，但结构契约指令仍在
    #[test]
    fn user_prompt_uses_custom_few_shot_but_keeps_contract() {
        let names = ["x.mp3"];
        let prompt = build_user_prompt("我的自定义示例", &names);
        assert!(prompt.contains("我的自定义示例"));
        assert!(!prompt.contains("反派影评"));
        assert!(prompt.contains("results"));
    }

    // resolved：非空白用户值优先；None/空白回落默认
    #[test]
    fn resolved_prefers_nonblank_then_falls_back() {
        assert_eq!(resolved(&Some("自定义".to_string()), "默认"), "自定义");
        assert_eq!(resolved(&Some("   ".to_string()), "默认"), "默认");
        assert_eq!(resolved(&None, "默认"), "默认");
    }

    fn cover_items() -> Vec<CoverMatchInput> {
        vec![
            CoverMatchInput {
                path: "/dir/ep1.mp3".into(),
                title: Some("第一集".into()),
                album: Some("节目".into()),
                candidates: vec!["/dir/ep1.jpg".into(), "/dir/cover.png".into()],
            },
            CoverMatchInput {
                path: "/dir/ep2.mp3".into(),
                title: None,
                album: None,
                candidates: vec!["/dir/cover.png".into()],
            },
        ]
    }

    // 高置信选中应映射为候选完整路径；低置信留空
    #[test]
    fn cover_results_map_chosen_and_respect_confidence() {
        let content = r#"{"results":[
            {"index":0,"chosen":0,"confidence":0.9},
            {"index":1,"chosen":0,"confidence":0.3}
        ]}"#;
        let parsed: LlmCoverResults = serde_json::from_str(content).unwrap();
        let out = assemble_cover_results(&cover_items(), parsed);

        assert_eq!(out[0].chosen.as_deref(), Some("/dir/ep1.jpg"));
        // 低于阈值 → 留空，但置信度仍透传
        assert_eq!(out[1].chosen, None);
        assert_eq!(out[1].confidence, Some(0.3));
    }

    // 越界候选序号 / null chosen 安全留空
    #[test]
    fn cover_results_handle_out_of_bounds_and_null() {
        let content = r#"{"results":[
            {"index":0,"chosen":99,"confidence":0.95},
            {"index":1,"chosen":null,"confidence":0.95}
        ]}"#;
        let parsed: LlmCoverResults = serde_json::from_str(content).unwrap();
        let out = assemble_cover_results(&cover_items(), parsed);
        assert_eq!(out[0].chosen, None);
        assert_eq!(out[1].chosen, None);
    }

    // 提示词应含候选图片文件名（仅文件名，不含目录）
    #[test]
    fn cover_prompt_lists_candidate_basenames() {
        let prompt = build_cover_prompt(&cover_items());
        assert!(prompt.contains("ep1.jpg"));
        assert!(prompt.contains("cover.png"));
        assert!(!prompt.contains("/dir/"));
        assert!(prompt.contains("results"));
    }
}
