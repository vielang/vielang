use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tracing::{info, error};
use uuid::Uuid;
use base64::Engine as _;

pub struct ArduinoCompilerService {
    cli_path: String,
    timeout: Duration,
    sketch_base_dir: PathBuf,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileResult {
    pub success: bool,
    pub hex: Option<String>,
    pub errors: String,
    pub output: String,
    pub compile_time_ms: u64,
}

impl ArduinoCompilerService {
    pub fn new(cli_path: &str, timeout_secs: u64, sketch_dir: &str) -> Self {
        Self {
            cli_path: cli_path.to_string(),
            timeout: Duration::from_secs(timeout_secs),
            sketch_base_dir: PathBuf::from(sketch_dir),
        }
    }

    pub async fn compile(&self, code: &str, board_fqbn: &str) -> CompileResult {
        let start = Instant::now();
        let build_id = Uuid::new_v4().to_string();
        let sketch_dir = self.sketch_base_dir.join(&build_id).join("sketch");
        let build_dir = self.sketch_base_dir.join(&build_id).join("build");

        // Create directories
        if let Err(e) = tokio::fs::create_dir_all(&sketch_dir).await {
            return CompileResult {
                success: false,
                hex: None,
                errors: format!("Failed to create sketch dir: {}", e),
                output: String::new(),
                compile_time_ms: start.elapsed().as_millis() as u64,
            };
        }
        let _ = tokio::fs::create_dir_all(&build_dir).await;

        // Write sketch file
        let ino_path = sketch_dir.join("sketch.ino");
        if let Err(e) = tokio::fs::write(&ino_path, code).await {
            let _ = tokio::fs::remove_dir_all(self.sketch_base_dir.join(&build_id)).await;
            return CompileResult {
                success: false,
                hex: None,
                errors: format!("Failed to write sketch: {}", e),
                output: String::new(),
                compile_time_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Run arduino-cli compile
        let result = tokio::time::timeout(self.timeout, async {
            Command::new(&self.cli_path)
                .args([
                    "compile",
                    "--fqbn", board_fqbn,
                    "--output-dir", build_dir.to_str().unwrap_or("/tmp"),
                    sketch_dir.to_str().unwrap_or("/tmp"),
                ])
                .output()
                .await
        })
        .await;

        let compile_time_ms = start.elapsed().as_millis() as u64;

        let compile_result = match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    // Find the .hex file
                    let hex_content = self.find_and_read_hex(&build_dir).await;
                    match hex_content {
                        Some(hex) => {
                            let hex_b64 = base64::engine::general_purpose::STANDARD.encode(&hex);
                            info!("Arduino compile success: {} bytes hex, {}ms", hex.len(), compile_time_ms);
                            CompileResult {
                                success: true,
                                hex: Some(hex_b64),
                                errors: String::new(),
                                output: stdout,
                                compile_time_ms,
                            }
                        }
                        None => CompileResult {
                            success: false,
                            hex: None,
                            errors: "Compilation succeeded but .hex file not found".into(),
                            output: stdout,
                            compile_time_ms,
                        },
                    }
                } else {
                    CompileResult {
                        success: false,
                        hex: None,
                        errors: stderr,
                        output: stdout,
                        compile_time_ms,
                    }
                }
            }
            Ok(Err(e)) => {
                error!("arduino-cli execution failed: {}", e);
                CompileResult {
                    success: false,
                    hex: None,
                    errors: format!("Failed to run arduino-cli: {}. Is it installed?", e),
                    output: String::new(),
                    compile_time_ms,
                }
            }
            Err(_) => CompileResult {
                success: false,
                hex: None,
                errors: format!("Compilation timed out after {}s", self.timeout.as_secs()),
                output: String::new(),
                compile_time_ms,
            },
        };

        // Cleanup temp directory
        let cleanup_dir = self.sketch_base_dir.join(&build_id);
        tokio::spawn(async move {
            let _ = tokio::fs::remove_dir_all(cleanup_dir).await;
        });

        compile_result
    }

    async fn find_and_read_hex(&self, build_dir: &PathBuf) -> Option<String> {
        let mut entries = tokio::fs::read_dir(build_dir).await.ok()?;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().map(|e| e == "hex").unwrap_or(false) {
                return tokio::fs::read_to_string(&path).await.ok();
            }
        }
        None
    }
}
