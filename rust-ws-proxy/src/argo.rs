//! Cloudflare Argo Tunnel process management.

use std::{path::Path, sync::Arc, time::Duration};

use anyhow::{Context, Result, bail};
use serde_json::Value;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::RwLock,
};
use tracing::{error, info, warn};

use crate::config::Config;

pub type SharedArgoDomain = Arc<RwLock<Option<String>>>;

/// Start and supervise a cloudflared tunnel.
pub async fn start_tunnel(config: Config, argo_domain: SharedArgoDomain) -> Result<()> {
    if !config.argo_domain.is_empty() {
        set_argo_domain(&argo_domain, config.argo_domain.clone()).await;
    }

    loop {
        if let Err(err) = run_once(&config, argo_domain.clone()).await {
            error!("Argo tunnel error: {err:#}");
        }

        warn!("Argo tunnel exited; restarting in 5 seconds");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn run_once(config: &Config, argo_domain: SharedArgoDomain) -> Result<()> {
    let args = tunnel_args(config).await?;
    info!("Starting Argo tunnel with {}", config.cloudflared_path);

    let mut child = Command::new(&config.cloudflared_path)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| {
            format!(
                "failed to start cloudflared at '{}'; set CLOUDFLARED_PATH or install cloudflared",
                config.cloudflared_path
            )
        })?;

    if let Some(stdout) = child.stdout.take() {
        let domain = argo_domain.clone();
        tokio::spawn(async move {
            read_cloudflared_output("stdout", stdout, domain).await;
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let domain = argo_domain.clone();
        tokio::spawn(async move {
            read_cloudflared_output("stderr", stderr, domain).await;
        });
    }

    let status = child.wait().await?;
    if status.success() {
        info!("Argo tunnel process exited successfully");
    } else {
        warn!("Argo tunnel process exited with status: {status}");
    }

    Ok(())
}

async fn tunnel_args(config: &Config) -> Result<Vec<String>> {
    if config.argo_auth.is_empty() {
        return Ok(vec![
            "tunnel".to_string(),
            "--edge-ip-version".to_string(),
            "auto".to_string(),
            "--no-autoupdate".to_string(),
            "--protocol".to_string(),
            "http2".to_string(),
            "--url".to_string(),
            format!("http://127.0.0.1:{}", config.port),
        ]);
    }

    if config.argo_auth.contains("TunnelSecret") {
        if config.argo_domain.is_empty() {
            bail!("ARGO_DOMAIN is required when ARGO_AUTH contains Cloudflare tunnel JSON");
        }

        let config_path = write_tunnel_config(config).await?;
        return Ok(vec![
            "tunnel".to_string(),
            "--edge-ip-version".to_string(),
            "auto".to_string(),
            "--config".to_string(),
            config_path,
            "run".to_string(),
        ]);
    }

    Ok(vec![
        "tunnel".to_string(),
        "--edge-ip-version".to_string(),
        "auto".to_string(),
        "--no-autoupdate".to_string(),
        "--protocol".to_string(),
        "http2".to_string(),
        "run".to_string(),
        "--token".to_string(),
        config.argo_auth.clone(),
    ])
}

async fn write_tunnel_config(config: &Config) -> Result<String> {
    tokio::fs::create_dir_all(&config.file_path).await?;

    let credentials_path = Path::new(&config.file_path).join("tunnel.json");
    let config_path = Path::new(&config.file_path).join("tunnel.yml");

    tokio::fs::write(&credentials_path, &config.argo_auth).await?;

    let value: Value = serde_json::from_str(&config.argo_auth)
        .context("ARGO_AUTH is not valid Cloudflare tunnel JSON")?;
    let tunnel_id = value
        .get("TunnelID")
        .and_then(Value::as_str)
        .context("ARGO_AUTH JSON missing TunnelID")?;

    let tunnel_yml = format!(
        r#"tunnel: {tunnel_id}
credentials-file: {}
protocol: http2

ingress:
  - hostname: {}
    service: http://127.0.0.1:{}
    originRequest:
      noTLSVerify: true
  - service: http_status:404
"#,
        credentials_path.display(),
        config.argo_domain,
        config.port
    );

    tokio::fs::write(&config_path, tunnel_yml).await?;
    Ok(config_path.display().to_string())
}

async fn read_cloudflared_output<R>(name: &'static str, reader: R, argo_domain: SharedArgoDomain)
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();

    loop {
        match lines.next_line().await {
            Ok(Some(line)) => {
                if let Some(domain) = extract_trycloudflare_domain(&line) {
                    set_argo_domain(&argo_domain, domain).await;
                }
                info!("cloudflared {name}: {line}");
            }
            Ok(None) => break,
            Err(err) => {
                warn!("failed reading cloudflared {name}: {err}");
                break;
            }
        }
    }
}

async fn set_argo_domain(argo_domain: &SharedArgoDomain, domain: String) {
    let mut current = argo_domain.write().await;
    if current.as_deref() != Some(domain.as_str()) {
        info!("Argo domain available: {domain}");
        *current = Some(domain);
    }
}

fn extract_trycloudflare_domain(line: &str) -> Option<String> {
    let marker = "trycloudflare.com";
    let mut search_from = 0;

    while let Some(relative_end) = line[search_from..].find(marker) {
        let end = search_from + relative_end + marker.len();
        let prefix = &line[..end];

        if let Some(start) = prefix
            .rfind("https://")
            .map(|idx| idx + "https://".len())
            .or_else(|| prefix.rfind("http://").map(|idx| idx + "http://".len()))
        {
            return Some(prefix[start..end].trim_matches('/').to_string());
        }

        search_from = end;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            uuid: "7bd180e8-1142-4387-93f5-03e8d750a896".to_string(),
            port: 3000,
            domain: String::new(),
            sub_path: "sub".to_string(),
            name: "test".to_string(),
            ws_path: "7bd180e8".to_string(),
            auto_access: false,
            debug: false,
            allow_shadowsocks: false,
            argo_enabled: true,
            argo_domain: String::new(),
            argo_auth: String::new(),
            cloudflared_path: "cloudflared".to_string(),
            file_path: ".tmp".to_string(),
        }
    }

    #[tokio::test]
    async fn quick_tunnel_args_point_to_local_service() {
        let config = test_config();
        let args = tunnel_args(&config).await.unwrap();

        assert_eq!(
            args,
            vec![
                "tunnel",
                "--edge-ip-version",
                "auto",
                "--no-autoupdate",
                "--protocol",
                "http2",
                "--url",
                "http://127.0.0.1:3000"
            ]
        );
    }

    #[tokio::test]
    async fn token_tunnel_args_use_cloudflare_token() {
        let mut config = test_config();
        config.argo_auth = "test-token".to_string();

        let args = tunnel_args(&config).await.unwrap();

        assert_eq!(
            args,
            vec![
                "tunnel",
                "--edge-ip-version",
                "auto",
                "--no-autoupdate",
                "--protocol",
                "http2",
                "run",
                "--token",
                "test-token"
            ]
        );
    }

    #[tokio::test]
    async fn json_tunnel_args_write_cloudflared_config() {
        let mut config = test_config();
        config.argo_domain = "example.com".to_string();
        config.argo_auth =
            r#"{"TunnelID":"12345678-1234-1234-1234-123456789abc","TunnelSecret":"secret"}"#
                .to_string();
        config.file_path = std::env::temp_dir()
            .join(format!("rust-ws-proxy-argo-test-{}", std::process::id()))
            .display()
            .to_string();

        let args = tunnel_args(&config).await.unwrap();
        let config_path = Path::new(&config.file_path).join("tunnel.yml");
        let credentials_path = Path::new(&config.file_path).join("tunnel.json");

        assert_eq!(
            args,
            vec![
                "tunnel".to_string(),
                "--edge-ip-version".to_string(),
                "auto".to_string(),
                "--config".to_string(),
                config_path.display().to_string(),
                "run".to_string(),
            ]
        );

        let tunnel_yml = tokio::fs::read_to_string(&config_path).await.unwrap();
        assert!(tunnel_yml.contains("tunnel: 12345678-1234-1234-1234-123456789abc"));
        assert!(tunnel_yml.contains("hostname: example.com"));
        assert!(tunnel_yml.contains("service: http://127.0.0.1:3000"));

        let credentials = tokio::fs::read_to_string(&credentials_path).await.unwrap();
        assert_eq!(credentials, config.argo_auth);

        tokio::fs::remove_dir_all(&config.file_path).await.unwrap();
    }

    #[test]
    fn extracts_trycloudflare_domain_from_output() {
        let line = "INF Requesting new quick Tunnel on trycloudflare.com... https://abc-def.trycloudflare.com";

        assert_eq!(
            extract_trycloudflare_domain(line).as_deref(),
            Some("abc-def.trycloudflare.com")
        );
    }
}
