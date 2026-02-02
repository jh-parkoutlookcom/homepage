use chrono::{DateTime, Duration, Utc};
use reqwest;

#[derive(serde::Deserialize)]
struct TokenResponse {
    #[serde(rename = "Token")]
    token: String,
}

use serde_json;

const AUTH_HEADER: &'static str = "Basic eXVpb3BcYWRtaW5pc3RyYXRvcjpoYWNrZXIjMQ==";

pub struct CvadRestApi {
    client: reqwest::Client,
    cvad_url: String,
    site_name: String,
    instance_id: Option<String>,
    token: Option<String>,
}

impl CvadRestApi {
    pub async fn new(v_cvad_url: &str, site_name: &str) -> Self {
        let cvad_url = format!("{}/cvad/manage", v_cvad_url);
        let client = reqwest::Client::new();

        // Try to fetch token and error handling
        let mut token = None;
        match client
            .post(format!("{}/Tokens", cvad_url))
            .header("Authorization", AUTH_HEADER)
            .body(" ")
            .send()
            .await
        {
            Ok(response) => match response.text().await {
                Ok(body) => {
                    token = serde_json::from_str::<TokenResponse>(&body)
                        .map(|t| t.token)
                        .ok();
                }
                Err(e) => {
                    eprintln!("Error reading token response body: {}", e);
                }
            },
            Err(e) => {
                eprintln!("Error fetching token: {}", e);
            }
        }

        // Try to fetch instance_id with error handling
        let mut instance_id = None;
        if let Some(ref t) = token {
            match client
                .get(format!("{}/Sites/{}", cvad_url, site_name))
                .header("Citrix-CustomerId", "CitrixOnPremises")
                .header("Authorization", format!("CWSAuth Bearer={}", t))
                .header("Accept", "application/json")
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(body) => {
                                if let Ok(site_info) =
                                    serde_json::from_str::<serde_json::Value>(&body)
                                {
                                    instance_id = site_info
                                        .get("Id")
                                        .and_then(|id| id.as_str().map(|s| s.to_string()));
                                }
                            }
                            Err(e) => {
                                eprintln!("Error reading site info response: {}", e);
                            }
                        }
                    } else {
                        eprintln!("Error fetching site info: {}", response.status());
                    }
                }
                Err(e) => {
                    eprintln!("Error sending request: {}", e);
                }
            }
        }

        CvadRestApi {
            client,
            cvad_url: cvad_url.to_string(),
            site_name: site_name.to_string(),
            instance_id: instance_id,
            token: token,
        }
    }

    pub async fn get_site_id(&self) -> Option<String> {
        let site_info = self.get_site(&self.site_name.as_str()).await?;
        site_info.get("Id")?.as_str().map(|s| s.to_string())
    }

    async fn get_site(&self, site: &str) -> Option<serde_json::Value> {
        let response = self
            .client
            .get(format!("{}/Sites/{}", self.cvad_url, site))
            .header("Citrix-CustomerId", "CitrixOnPremises")
            .header(
                "Authorization",
                format!("CWSAuth Bearer={}", self.token.as_ref().unwrap()),
            )
            .header("Accept", "application/json")
            .send()
            .await;

        match response {
            Ok(res) => {
                if res.status().is_success() {
                    match res.text().await {
                        Ok(body) => serde_json::from_str(&body).ok(),
                        Err(e) => {
                            eprintln!("Error reading site info response: {}", e);
                            None
                        }

                    }
                } else {
                    eprintln!("Error fetching site info: {}", res.status());
                    None
                }
            }
            Err(e) => {
                eprintln!("Error sending request: {}", e);
                None
            }
        }
    }

    pub async fn get_expire_date(&self) -> Option<String> {
        let first_log_date = self.get_first_log_date().await;
        let dt = DateTime::parse_from_rfc3339(&first_log_date?)
            .unwrap()
            .with_timezone(&Utc);

        let plus_30 = dt + Duration::days(30);

        // Local time으로 변환
        let local_dt = plus_30.with_timezone(&chrono::Local);

        Some(local_dt.to_rfc3339())
    }

    async fn get_first_log_date(&self) -> Option<String> {
        let response = self
            .client
            .get(format!("{}/ConfigLog/GetFirstLogDate", self.cvad_url))
            .header("Citrix-CustomerId", "CitrixOnPremises")
            .header("Citrix-InstanceId", self.instance_id.as_ref().unwrap())
            .header(
                "Authorization",
                format!("CWSAuth Bearer={}", self.token.as_ref().unwrap()),
            )
            .send()
            .await;

        match response {
            Ok(res) => {
                if res.status().is_success() {
                    let body = res.text().await.unwrap();
                    serde_json::from_str(&body).ok()
                } else {
                    eprintln!("Error fetching first log date: {}", res.status());
                    None
                }
            }
            Err(e) => {
                eprintln!("Error sending request: {}", e);
                None
            }
        }
    }
}
