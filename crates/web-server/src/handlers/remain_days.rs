use axum::{http::StatusCode, response::Json};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use shared::cvad_restapi;
use shared::ssl_checker;

const CVAD_LISTS: [[&str; 2]; 2] = [
    ["https://nuc-citrix.yuiop.org", "nuc-citrix"],
    ["https://cvad.yuiop.org", "yuiop"],
];

const SSL_LISTS: [&str; 5] = [
    "https://nuc-citrix.yuiop.org",
    "https://nuc-gateway.yuiop.org",
    "https://cvad.yuiop.org",
    "https://gateway.yuiop.org:40443",
    "https://n8n.yuiop.org",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceStatus {
    name: String,
    service_type: String, // "cvad" or "ssl"
    expire_date: String,
    remaining: String,
    status: String,
    status_code: String, // "ok", "warning", "error", "unknown"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    timestamp: String,
    total_services: usize,
    services: Vec<ServiceStatus>,
    summary: StatusSummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusSummary {
    ok: usize,
    warning: usize,
    error: usize,
    unknown: usize,
}

fn truncate(s: &str, max: usize) -> String {
    let single_line = s.replace('\n', " ");
    let mut acc = String::new();
    for ch in single_line.chars().take(max) {
        acc.push(ch);
    }
    if single_line.chars().count() <= max {
        acc
    } else {
        format!("{}…", acc)
    }
}

fn calculate_remaining(expire_rfc3339: &str) -> (String, String, String) {
    if let Ok(expire_dt) = DateTime::parse_from_rfc3339(expire_rfc3339) {
        let now = Local::now();
        let expire_local = expire_dt.with_timezone(&Local);
        let duration = expire_local.signed_duration_since(now);
        let total_secs = duration.num_seconds();

        if total_secs < 0 {
            return (
                "Expired".to_string(),
                "error".to_string(),
                "expired".to_string(),
            );
        }

        let days = total_secs / 86_400;
        let hours = (total_secs % 86_400) / 3600;
        let minutes = (total_secs % 3600) / 60;

        let formatted = if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes.max(0))
        };

        let (status_code, status) = if days < 0 || (days == 0 && hours < 1) {
            ("error", "expired")
        } else if days == 0 || days <= 7 {
            ("warning", "expiring_soon")
        } else {
            ("ok", "valid")
        };

        (formatted, status_code.to_string(), status.to_string())
    } else {
        (
            "N/A".to_string(),
            "unknown".to_string(),
            "parse_error".to_string(),
        )
    }
}

async fn check_all_services() -> Vec<ServiceStatus> {
    let mut services = Vec::new();

    // CVAD 라이센스 체크
    for row in CVAD_LISTS.iter() {
        let cvad_url = row[0];
        let site_name = row[1];
        let api = cvad_restapi::CvadRestApi::new(cvad_url, site_name).await;
        let expire_date = api.get_expire_date().await;
        let name = cvad_url.replace("https://", "").replace("/cvad/manage", "");

        let (remaining, status_code, status) = if let Some(ref expire) = expire_date {
            if let Ok(expire_dt) = DateTime::parse_from_rfc3339(expire) {
                let (remain, code, stat) = calculate_remaining(expire);
                (remain, code, stat)
            } else {
                (
                    "N/A".to_string(),
                    "unknown".to_string(),
                    "parse_error".to_string(),
                )
            }
        } else {
            (
                "N/A".to_string(),
                "error".to_string(),
                "no_data".to_string(),
            )
        };

        let formatted_date = if let Some(ref expire) = expire_date {
            if let Ok(expire_dt) = DateTime::parse_from_rfc3339(expire) {
                expire_dt.format("%Y-%m-%d %H:%M:%S").to_string()
            } else {
                "N/A".to_string()
            }
        } else {
            "N/A".to_string()
        };

        services.push(ServiceStatus {
            name,
            service_type: "cvad".to_string(),
            expire_date: formatted_date,
            remaining,
            status,
            status_code,
        });
    }

    // SSL 인증서 체크
    for ssl_url in SSL_LISTS.iter() {
        let status = ssl_checker::check_ssl_expiry(ssl_url).await;
        let name = ssl_url.replace("https://", "").replace("http://", "");

        let (expire_display, remaining_display, status_text, status_code) = match status {
            ssl_checker::SslCheckStatus::Ok { expire_rfc3339 } => {
                if let Ok(expire_dt) = DateTime::parse_from_rfc3339(&expire_rfc3339) {
                    let (remain, code, stat) = calculate_remaining(&expire_rfc3339);
                    (
                        expire_dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                        remain,
                        stat,
                        code,
                    )
                } else {
                    (
                        "N/A".to_string(),
                        "N/A".to_string(),
                        "parse_error".to_string(),
                        "unknown".to_string(),
                    )
                }
            }
            ssl_checker::SslCheckStatus::ChainUntrusted {
                expire_rfc3339,
                verify_desc: _,
            } => {
                if let Ok(expire_dt) = DateTime::parse_from_rfc3339(&expire_rfc3339) {
                    let (remain, _, _) = calculate_remaining(&expire_rfc3339);
                    (
                        expire_dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                        remain,
                        "untrusted".to_string(),
                        "warning".to_string(),
                    )
                } else {
                    (
                        "N/A".to_string(),
                        "N/A".to_string(),
                        "untrusted".to_string(),
                        "warning".to_string(),
                    )
                }
            }
            ssl_checker::SslCheckStatus::TlsBypassed {
                expire_rfc3339,
                reason,
            } => {
                if let Ok(expire_dt) = DateTime::parse_from_rfc3339(&expire_rfc3339) {
                    let (remain, _, _) = calculate_remaining(&expire_rfc3339);
                    (
                        expire_dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                        remain,
                        format!("bypassed: {}", truncate(&reason, 40)),
                        "warning".to_string(),
                    )
                } else {
                    (
                        "N/A".to_string(),
                        "N/A".to_string(),
                        "bypassed".to_string(),
                        "warning".to_string(),
                    )
                }
            }
            ssl_checker::SslCheckStatus::TcpFailed(e) => (
                "N/A".to_string(),
                "N/A".to_string(),
                format!("tcp_failed: {}", truncate(&e, 40)),
                "error".to_string(),
            ),
            ssl_checker::SslCheckStatus::TlsFailed(e) => (
                "N/A".to_string(),
                "N/A".to_string(),
                format!("tls_failed: {}", truncate(&e, 40)),
                "error".to_string(),
            ),
            ssl_checker::SslCheckStatus::NoCertificate => (
                "N/A".to_string(),
                "N/A".to_string(),
                "no_certificate".to_string(),
                "error".to_string(),
            ),
            ssl_checker::SslCheckStatus::ParseFailed(src) => (
                "N/A".to_string(),
                "N/A".to_string(),
                format!("parse_failed: {}", truncate(&src, 40)),
                "unknown".to_string(),
            ),
            ssl_checker::SslCheckStatus::DnsFailed => (
                "N/A".to_string(),
                "N/A".to_string(),
                "dns_failed".to_string(),
                "error".to_string(),
            ),
        };

        services.push(ServiceStatus {
            name,
            service_type: "ssl".to_string(),
            expire_date: expire_display,
            remaining: remaining_display,
            status: status_text,
            status_code,
        });
    }

    services
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": Local::now().to_rfc3339()
    }))
}

pub async fn get_remain_days() -> Result<Json<ApiResponse>, StatusCode> {
    let services = check_all_services().await;

    let mut summary = StatusSummary {
        ok: 0,
        warning: 0,
        error: 0,
        unknown: 0,
    };

    for service in &services {
        match service.status_code.as_str() {
            "ok" => summary.ok += 1,
            "warning" => summary.warning += 1,
            "error" => summary.error += 1,
            _ => summary.unknown += 1,
        }
    }

    let response = ApiResponse {
        timestamp: Local::now().to_rfc3339(),
        total_services: services.len(),
        services,
        summary,
    };

    Ok(Json(response))
}
