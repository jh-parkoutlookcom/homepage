use chrono::{DateTime, Local};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use std::io::stdout;
use std::vec;
use tokio;

use shared::cvad_restapi;
use shared::ssl_checker;

const CVAD_LISTS: [[&str; 2]; 2] = [
    ["https://nuc-citrix.yuiop.org/cvad/manage", "nuc-citrix"],
    ["https://cvad.yuiop.org/cvad/manage", "yuiop"],
];

const SSL_LISTS: [&str; 5] = [
    "https://nuc-citrix.yuiop.org",
    "https://nuc-gateway.yuiop.org",
    "https://cvad.yuiop.org",
    "https://gateway.yuiop.org:40443",
    "https://n8n.yuiop.org",
];

fn truncate(s: &str) -> String {
    const MAX: usize = 40;
    let single_line = s.replace('\n', " ");
    let mut acc = String::new();
    for ch in single_line.chars().take(MAX) {
        acc.push(ch);
    }
    if single_line.chars().count() <= MAX {
        acc
    } else {
        format!("{}…", acc)
    }
}

// 기존 bash 기반 SSL 체크 함수 제거됨. ssl_checker::check_ssl_expiry 사용.

fn format_remaining(expire_rfc3339: &str) -> (String, Color, String) {
    if let Ok(expire_dt) = DateTime::parse_from_rfc3339(expire_rfc3339) {
        let now = Local::now();
        let expire_local = expire_dt.with_timezone(&Local);
        let duration = expire_local.signed_duration_since(now);
        let total_secs = duration.num_seconds();
        if total_secs < 0 {
            return ("Expired".to_string(), Color::Red, "Expired".to_string());
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
        let (color, status) = if days < 0 || (days == 0 && hours < 1) {
            (Color::Red, "Expired")
        } else if days == 0 || days <= 7 {
            (Color::Yellow, "Soon")
        } else {
            (Color::Green, "OK")
        };
        return (formatted, color, status.to_string());
    }
    ("N/A".to_string(), Color::Gray, "N/A".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut table_rows = Vec::new();

    // CVAD 라이센스 체크
    for row in CVAD_LISTS.iter() {
        let cvad_url = row[0];
        let instance_id = row[1];
        let api = cvad_restapi::CvadRestApi::new(cvad_url, instance_id).await;
        let expire_date = api.get_expire_date().await;
        let name = format!(
            "CVAD: {}",
            cvad_url.replace("https://", "").replace("/cvad/manage", "")
        );

        // 잔여 시간 및 상태
        let (remain_pretty, status_color, formatted_date, status_text) =
            if let Some(ref expire) = expire_date {
                if let Ok(expire_dt) = DateTime::parse_from_rfc3339(expire) {
                    let (pretty, color, status) = format_remaining(expire);
                    (
                        pretty,
                        color,
                        expire_dt.format("%Y-%m-%d %H:%M").to_string(),
                        status,
                    )
                } else {
                    (
                        "N/A".to_string(),
                        Color::Gray,
                        "N/A".to_string(),
                        "N/A".to_string(),
                    )
                }
            } else {
                (
                    "N/A".to_string(),
                    Color::Gray,
                    "N/A".to_string(),
                    "No Data".to_string(),
                )
            };

        table_rows.push(
            Row::new(vec![
                Cell::from(Span::styled(
                    name,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    formatted_date,
                    Style::default().fg(Color::White),
                )),
                Cell::from(Span::styled(
                    remain_pretty,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(status_text, Style::default().fg(status_color))),
            ])
            .style(Style::default().bg(Color::Black)),
        )
    }

    // SSL 인증서 체크
    for ssl_url in SSL_LISTS.iter() {
        let status = ssl_checker::check_ssl_expiry(ssl_url).await;
        let name = format!(
            "SSL: {}",
            ssl_url.replace("https://", "").replace("http://", "")
        );

        let (expire_display, remaining_display, status_text, color) = match status {
            ssl_checker::SslCheckStatus::Ok { expire_rfc3339 } => {
                if let Ok(expire_dt) = DateTime::parse_from_rfc3339(&expire_rfc3339) {
                    let (pretty, remain_color, status_str) = format_remaining(&expire_rfc3339);
                    (
                        expire_dt.format("%Y-%m-%d %H:%M").to_string(),
                        pretty,
                        status_str,
                        remain_color,
                    )
                } else {
                    (
                        "ParseErr".to_string(),
                        "N/A".to_string(),
                        "ParseErr".to_string(),
                        Color::Gray,
                    )
                }
            }
            ssl_checker::SslCheckStatus::ChainUntrusted {
                expire_rfc3339,
                verify_desc,
            } => {
                if let Ok(expire_dt) = DateTime::parse_from_rfc3339(&expire_rfc3339) {
                    let (pretty, _, _) = format_remaining(&expire_rfc3339);
                    (
                        expire_dt.format("%Y-%m-%d %H:%M").to_string(),
                        pretty,
                        "Untrusted".to_string(),
                        Color::Magenta,
                    )
                } else {
                    (
                        "Untrusted".to_string(),
                        truncate(&verify_desc),
                        "Untrusted".to_string(),
                        Color::Magenta,
                    )
                }
            }
            ssl_checker::SslCheckStatus::TlsBypassed {
                expire_rfc3339,
                reason,
            } => {
                if let Ok(expire_dt) = DateTime::parse_from_rfc3339(&expire_rfc3339) {
                    let (pretty, _, _) = format_remaining(&expire_rfc3339);
                    (
                        expire_dt.format("%Y-%m-%d %H:%M").to_string(),
                        pretty,
                        format!("Bypassed: {}", truncate(&reason)),
                        Color::LightBlue,
                    )
                } else {
                    (
                        "Bypassed".to_string(),
                        "N/A".to_string(),
                        "Bypassed".to_string(),
                        Color::LightBlue,
                    )
                }
            }
            ssl_checker::SslCheckStatus::TcpFailed(e) => (
                "Conn Fail".to_string(),
                "N/A".to_string(),
                truncate(&e),
                Color::Red,
            ),
            ssl_checker::SslCheckStatus::TlsFailed(e) => (
                "TLS Fail".to_string(),
                "N/A".to_string(),
                truncate(&e),
                Color::Red,
            ),
            ssl_checker::SslCheckStatus::NoCertificate => (
                "No Cert".to_string(),
                "N/A".to_string(),
                "No Cert".to_string(),
                Color::Red,
            ),
            ssl_checker::SslCheckStatus::ParseFailed(src) => (
                "ParseFail".to_string(),
                "N/A".to_string(),
                truncate(&src),
                Color::Gray,
            ),
            ssl_checker::SslCheckStatus::DnsFailed => (
                "DNS Fail".to_string(),
                "N/A".to_string(),
                "DNS".to_string(),
                Color::Red,
            ),
        };

        table_rows.push(
            Row::new(vec![
                Cell::from(Span::styled(
                    name,
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    expire_display,
                    Style::default().fg(Color::White),
                )),
                Cell::from(Span::styled(
                    remaining_display,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(status_text, Style::default().fg(color))),
            ])
            .style(Style::default().bg(Color::Black)),
        )
    }

    // ratatui로 출력
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3), // 제목용
                    Constraint::Min(0),    // 테이블용
                    Constraint::Length(3), // 하단 정보용
                ]
                .as_ref(),
            )
            .split(f.area());

        // 상단 제목
        let title = Paragraph::new("🖥️  Citrix CVAD & SSL Certificate Expiration Monitor")
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            );
        f.render_widget(title, chunks[0]);

        // 메인 테이블
        let header = Row::new(vec![
            Cell::from(Span::styled(
                "🏢 Service Name",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "📅 Expire Date",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "⏰ Remaining",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "📌 Status",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
        ])
        .style(Style::default().bg(Color::DarkGray));

        let table = Table::new(
            table_rows,
            [
                Constraint::Percentage(40),
                Constraint::Percentage(25),
                Constraint::Percentage(20),
                Constraint::Percentage(15),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title("📊 CVAD License & SSL Certificate Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .column_spacing(1)
        .row_highlight_style(Style::default().bg(Color::DarkGray));

        f.render_widget(table, chunks[1]);

        // 하단 범례
        let legend = Paragraph::new(vec![Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Cyan)),
            Span::raw("CVAD License  "),
            Span::styled("● ", Style::default().fg(Color::Magenta)),
            Span::raw("SSL Certificate  "),
            Span::styled("● ", Style::default().fg(Color::Green)),
            Span::raw("OK  "),
            Span::styled("● ", Style::default().fg(Color::Yellow)),
            Span::raw("Soon (≤7d)  "),
            Span::styled("● ", Style::default().fg(Color::Magenta)),
            Span::raw("Untrusted  "),
            Span::styled("● ", Style::default().fg(Color::LightBlue)),
            Span::raw("Bypassed (Insecure)  "),
            Span::styled("● ", Style::default().fg(Color::Red)),
            Span::raw("Fail / Expired"),
        ])])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Legend")
                .border_style(Style::default().fg(Color::Gray)),
        );
        f.render_widget(legend, chunks[2]);
    })?;

    std::thread::sleep(std::time::Duration::from_secs(5)); // 5초 후 종료
    Ok(())
}
