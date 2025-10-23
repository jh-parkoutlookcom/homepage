use reqwest;
use chrono::{DateTime, Duration, Utc};

#[derive(serde::Deserialize)]
struct TokenResponse {
    Token: String,
}
use serde_json;
use serde::Deserialize;

const AUTH_HEADER: &'static str = "Basic eXVpb3BcYWRtaW5pc3RyYXRvcjpoYWNrZXIjMQ==";


pub struct CvadRestApi {
    client: reqwest::Client,
    cvad_url: String,
    site_name: String, 
    instance_id: Option<String>, 
    token: Option<String>,
}

impl CvadRestApi {
    pub async fn new(cvad_url: &str, site_name: &str) -> Self{
        let client = reqwest::Client::new();
        let mut res = client
            .post(format!("{}/Tokens", cvad_url))
            .header("Authorization", AUTH_HEADER)
            .body(" ")
            .send()
            .await;

        let token = match res {
            Ok(response) => {
                let body = response.text().await.unwrap();

                // Token 값만 추출
                serde_json::from_str::<TokenResponse>(&body)
                    .map(|t| t.Token)
                    .ok()
            }
            Err(e) => {
                eprintln!("Error fetching token: {}", e);
                None
            }
        };

        res = client
            .get(format!("{}/Sites/{}", cvad_url, site_name))
            .header("Citrix-CustomerId", "CitrixOnPremises")
            .header("Authorization", format!("CWSAuth Bearer={}", token.as_ref().unwrap()))
            .header("Accept", "application/json")
            .send()
            .await;

        let instance_id = match res {
            Ok(response) => {
                if response.status().is_success() {
                    let body = response.text().await.unwrap();
                    let site_info: serde_json::Value = serde_json::from_str(&body).unwrap();
                    site_info.get("Id").and_then(|id| id.as_str().map(|s| s.to_string()))
                } else {
                    eprintln!("Error fetching site info: {}", response.status());
                    None
                }
            }
            Err(e) => {
                eprintln!("Error sending request: {}", e);
                None
            }
        };

        CvadRestApi { client, cvad_url: cvad_url.to_string(), site_name: site_name.to_string(), instance_id: instance_id, token: token }
    }

   pub async fn get_site_id(&self) -> Option<String> {
        let site_info = self.get_site(&self.site_name.as_str()).await?;
        site_info.get("Id")?.as_str().map(|s| s.to_string())
    }

    async fn get_site(&self, site: &str) -> Option<serde_json::Value> {
        let response = self.client
            .get(format!("{}/Sites/{}", self.cvad_url, site))
            .header("Citrix-CustomerId", "CitrixOnPremises")
            .header("Authorization", format!("CWSAuth Bearer={}", self.token.as_ref().unwrap()))
            .header("Accept", "application/json")
            .send()
            .await;

        match response {
            Ok(res) => {
                if res.status().is_success() {
                    let body = res.text().await.unwrap();
                    serde_json::from_str(&body).ok()
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
        let dt = DateTime::parse_from_rfc3339(&first_log_date?).unwrap().with_timezone(&Utc);

        let plus_30 = dt + Duration::days(30);

        // Local time으로 변환
        let local_dt = plus_30.with_timezone(&chrono::Local);

        Some(local_dt.to_rfc3339())
    }
    
    async fn get_first_log_date(&self) -> Option<String> {
        let response = self.client
            .get(format!("{}/ConfigLog/GetFirstLogDate", self.cvad_url))
            .header("Citrix-CustomerId","CitrixOnPremises")
            .header("Citrix-InstanceId", self.instance_id.as_ref().unwrap())
            .header("Authorization", format!("CWSAuth Bearer={}", self.token.as_ref().unwrap()))
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