// BioPhys Autonomous App Generator & Exporter
// Code-to-Disk, Interactive Live Sandbox HTML Bundler, Standalone Exporter

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeneratedAppMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub app_type: String, // "svelte", "html", "react"
    pub main_file: String,
    pub created_at: String,
    pub bundle_html: String,
}

pub struct AppGenerator;

impl AppGenerator {
    pub fn get_apps_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("biophys_generated_apps");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    /// 1. Code-to-Disk: 디스크에 실제 파일 생성
    pub fn save_app(
        id: &str,
        name: &str,
        description: &str,
        app_type: &str,
        source_code: &str,
        bundle_html: &str,
    ) -> Result<GeneratedAppMeta, String> {
        let apps_dir = Self::get_apps_dir();
        let app_dir = apps_dir.join(id);
        fs::create_dir_all(&app_dir).map_err(|e| format!("폴더 생성 실패: {:?}", e))?;

        // 1) 원본 소스코드 파일 저장 (예: App.svelte 또는 index.html)
        let main_filename = if app_type == "svelte" { "App.svelte" } else { "index.html" };
        let source_path = app_dir.join(main_filename);
        fs::write(&source_path, source_code).map_err(|e| format!("소스 저장 실패: {:?}", e))?;

        // 2) 샌드박스 실행용 단독 실행 HTML 번들 저장
        let bundle_path = app_dir.join("bundle.html");
        let final_bundle = if bundle_html.is_empty() {
            Self::wrap_in_standalone_html(name, source_code)
        } else {
            bundle_html.to_string()
        };
        fs::write(&bundle_path, &final_bundle).map_err(|e| format!("번들 저장 실패: {:?}", e))?;

        let meta = GeneratedAppMeta {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            app_type: app_type.to_string(),
            main_file: main_filename.to_string(),
            created_at: chrono_local_now(),
            bundle_html: final_bundle,
        };

        // 3) 메타데이터 JSON 저장
        let meta_json = serde_json::to_string_pretty(&meta).unwrap_or_default();
        let _ = fs::write(app_dir.join("meta.json"), meta_json);

        Ok(meta)
    }

    /// 2. 생성된 앱 목록 조회
    pub fn list_apps() -> Vec<GeneratedAppMeta> {
        let apps_dir = Self::get_apps_dir();
        let mut list = Vec::new();

        if let Ok(entries) = fs::read_dir(apps_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let meta_file = p.join("meta.json");
                    if let Ok(content) = fs::read_to_string(meta_file) {
                        if let Ok(meta) = serde_json::from_str::<GeneratedAppMeta>(&content) {
                            list.push(meta);
                        }
                    }
                }
            }
        }
        list.reverse();
        list
    }

    /// 3. 단독 포터블 HTML 앱으로 패키징 (Glassmorphism & Tailwind 탑재)
    pub fn wrap_in_standalone_html(title: &str, body_or_script: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="ko" class="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{}</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link href="https://fonts.googleapis.com/css2?family=Pretendard:wght@400;600;700;800&family=JetBrains+Mono:wght@400;600&display=swap" rel="stylesheet">
  <style>
    body {{
      font-family: 'Pretendard', sans-serif;
      background: radial-gradient(circle at top left, #0f172a, #020617);
      min-height: 100vh;
      color: #f8fafc;
      margin: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 1.5rem;
    }}
    .glass-panel {{
      background: rgba(15, 23, 42, 0.65);
      backdrop-filter: blur(24px);
      -webkit-backdrop-filter: blur(24px);
      border: 1px solid rgba(255, 255, 255, 0.12);
      box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5), inset 0 1px 1px rgba(255, 255, 255, 0.1);
    }}
  </style>
</head>
<body class="selection:bg-cyan-500 selection:text-black">
  <div id="app" class="w-full max-w-md">
    {}
  </div>
</body>
</html>"#,
            title, body_or_script
        )
    }

    /// 4. 독립 파일(HTML) 내보내기
    pub fn export_to_desktop(id: &str, custom_dest_dir: Option<String>) -> Result<String, String> {
        let apps_dir = Self::get_apps_dir();
        let app_dir = apps_dir.join(id);
        let bundle_path = app_dir.join("bundle.html");

        if !bundle_path.exists() {
            return Err("해당 앱 번들을 찾을 수 없습니다.".to_string());
        }

        let target_dir = match custom_dest_dir {
            Some(d) => PathBuf::from(d),
            None => dirs_next_desktop().unwrap_or_else(|| std::env::temp_dir()),
        };

        let target_file = target_dir.join(format!("{}.html", id));
        fs::copy(&bundle_path, &target_file)
            .map_err(|e| format!("내보내기 복사 실패: {:?}", e))?;

        Ok(target_file.to_string_lossy().to_string())
    }
}

fn chrono_local_now() -> String {
    // 간이 타임스탬프
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{}s", since_the_epoch.as_secs())
}

fn dirs_next_desktop() -> Option<PathBuf> {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let desktop = PathBuf::from(profile).join("Desktop");
        if desktop.exists() {
            return Some(desktop);
        }
    }
    None
}
