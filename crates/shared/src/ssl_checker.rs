use chrono::{DateTime, Utc};
use openssl::ssl::{SslConnector, SslMethod, SslStream};
use openssl::x509::X509VerifyResult;
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants (e.g., DnsFailed) reserved for future use
pub enum SslCheckStatus {
    Ok { expire_rfc3339: String },
    DnsFailed,
    TcpFailed(String),
    TlsFailed(String),
    ChainUntrusted { expire_rfc3339: String, verify_desc: String },
    TlsBypassed { expire_rfc3339: String, reason: String },
    NoCertificate,
    ParseFailed(String),
}

fn parse_host_port(input: &str) -> (String, u16) {
    let s = input.trim_start_matches("https://").trim_start_matches("http://");
    let host_part = s.split('/').next().unwrap_or(s);
    if let Some((h,p)) = host_part.rsplit_once(':') { if let Ok(port)=p.parse::<u16>() { return (h.to_string(), port); } }
    (host_part.to_string(), 443)
}

pub async fn check_ssl_expiry(host: &str) -> SslCheckStatus {
    let (domain, port) = parse_host_port(host);
    if cfg!(debug_assertions) { println!("[SSL] Checking {}:{}", domain, port); }

    let domain_clone = domain.clone();
    tokio::task::spawn_blocking(move || {
        let addr = format!("{}:{}", domain_clone, port);
        let tcp = match TcpStream::connect(addr) { Ok(s) => s, Err(e) => return SslCheckStatus::TcpFailed(e.to_string()) };
        tcp.set_read_timeout(Some(Duration::from_secs(8))).ok();
        tcp.set_write_timeout(Some(Duration::from_secs(8))).ok();

        let builder = match SslConnector::builder(SslMethod::tls()) { Ok(b) => b, Err(e) => return SslCheckStatus::TlsFailed(e.to_string()) };
        let connector = builder.build();
        let ssl_stream = match connector.connect(&domain_clone, tcp) { Ok(s) => s, Err(e) => {
            // 1차 핸드셰이크 실패 시: 검증 비활성화(insecure) 재시도로 만료일만 추출
            if cfg!(debug_assertions) { eprintln!("[SSL][Retry] normal TLS failed: {} -- retry with no verification", e); }
            // 새로 TCP 연결 시도
            let addr2 = format!("{}:{}", domain_clone, port);
            let tcp2 = match TcpStream::connect(addr2) { Ok(s) => s, Err(e2) => return SslCheckStatus::TlsFailed(format!("{} | retry_tcp: {}", e, e2)) };
            tcp2.set_read_timeout(Some(Duration::from_secs(8))).ok();
            tcp2.set_write_timeout(Some(Duration::from_secs(8))).ok();
            let mut insecure_builder = match SslConnector::builder(SslMethod::tls()) { Ok(b) => b, Err(e2) => return SslCheckStatus::TlsFailed(format!("{} | retry_builder: {}", e, e2)) };
            // Insecure: trust all certs (dangerous, only for expiry retrieval)
            // Safety: set_verify_callback is safe with NONE; no need for unsafe block (API is safe)
            insecure_builder.set_verify_callback(openssl::ssl::SslVerifyMode::NONE, |_, _| true);
            let insecure = insecure_builder.build();
            return match insecure.connect(&domain_clone, tcp2) { Ok(s2) => {
                // 추출만 하고 상태는 Bypassed 처리 (인증서 만료정보만 수집)
                extract_status_internal(s2, true, format!("normal_fail:{}", e))
            }, Err(e2) => SslCheckStatus::TlsFailed(format!("{} | retry_handshake: {}", e, e2)) };
        }};
        extract_status_internal(ssl_stream, false, "".to_string())
    }).await.unwrap_or(SslCheckStatus::TlsFailed("JoinError".to_string()))
}

fn extract_status_internal(ssl_stream: SslStream<TcpStream>, bypassed: bool, reason: String) -> SslCheckStatus {
    // 체인 검증 결과 확인
    let verify_result = ssl_stream.ssl().verify_result();
    let verify_code_desc = format!("{:?}", verify_result);

    let cert = match ssl_stream.ssl().peer_certificate() { Some(c) => c, None => return SslCheckStatus::NoCertificate };
    let asn1_time = cert.not_after();
    let text = asn1_time.to_string();
    let fmts = ["%b %e %H:%M:%S %Y %Z", "%b %d %H:%M:%S %Y %Z"]; 
    let mut parsed_dt: Option<DateTime<Utc>> = None;
    for f in fmts.iter() {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(&text, f) {
            let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
            parsed_dt = Some(dt);
            break;
        }
    }
    let dt = match parsed_dt { Some(d) => d, None => return SslCheckStatus::ParseFailed(text) };
    let rfc3339 = dt.to_rfc3339();

    if bypassed {
        return SslCheckStatus::TlsBypassed { expire_rfc3339: rfc3339, reason };
    }

    if verify_result != X509VerifyResult::OK {
        SslCheckStatus::ChainUntrusted { expire_rfc3339: rfc3339, verify_desc: verify_code_desc }
    } else {
        SslCheckStatus::Ok { expire_rfc3339: rfc3339 }
    }
}
