use anyhow::{anyhow, Result};
use serde_json::Value;

const DEFAULT_API_BASE_URL: &str = "https://api-test.ksef.mf.gov.pl/v2";

pub struct KsefClient {
    client: reqwest::Client,
    base_url: String,
    session_token: Option<String>,
}

impl KsefClient {
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_API_BASE_URL.to_string())
    }

    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            session_token: None,
        }
    }

    pub fn set_session_token(&mut self, token: String) {
        self.session_token = Some(token);
    }

    pub fn clear_session_token(&mut self) {
        self.session_token = None;
    }

    fn build_headers(&self, extra_headers: Option<reqwest::header::HeaderMap>) -> reqwest::header::HeaderMap {
        let mut headers = extra_headers.unwrap_or_default();
        if let Some(token) = &self.session_token {
            if let Ok(value) = token.parse() {
                headers.insert("SessionToken", value);
            }
        }
        headers
    }

    pub async fn get_active_sessions(&self, page_size: i64, continuation_token: Option<&str>) -> Result<String> {
        let url = format!("{}/auth/sessions?pageSize={}", self.base_url, page_size);

        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(token) = continuation_token {
            headers.insert("x-continuation-token", token.parse()?);
        }
        let headers = self.build_headers(Some(headers));

        let response = self.client.get(&url).headers(headers).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn get_current_session(&self) -> Result<String> {
        let url = format!("{}/auth/sessions/current", self.base_url);
        let headers = self.build_headers(None);

        let response = self.client.get(&url).headers(headers).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn terminate_session(&self, reference_number: &str) -> Result<String> {
        let url = format!("{}/auth/sessions/{}", self.base_url, reference_number);
        let headers = self.build_headers(None);

        let response = self.client.delete(&url).headers(headers).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn get_invoice(&self, ksef_number: &str) -> Result<String> {
        let url = format!("{}/invoices/ksef/{}", self.base_url, ksef_number);
        let headers = self.build_headers(None);

        let response = self.client.get(&url).headers(headers).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn query_invoice_metadata(&self, query: &Value) -> Result<String> {
        let url = format!("{}/invoices/query/metadata", self.base_url);
        let headers = self.build_headers(None);

        let response = self.client.post(&url).headers(headers).json(query).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn create_invoice_export(&self, export_params: &Value) -> Result<String> {
        let url = format!("{}/invoices/exports", self.base_url);
        let headers = self.build_headers(None);

        let response = self.client.post(&url).headers(headers).json(export_params).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn get_export_status(&self, reference_number: &str) -> Result<String> {
        let url = format!("{}/invoices/exports/{}", self.base_url, reference_number);
        let headers = self.build_headers(None);

        let response = self.client.get(&url).headers(headers).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn get_public_key_certificates(&self) -> Result<String> {
        let url = format!("{}/security/public-key-certificates", self.base_url);

        let response = self.client.get(&url).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn get_rate_limits(&self) -> Result<String> {
        let url = format!("{}/rate-limits", self.base_url);
        let headers = self.build_headers(None);

        let response = self.client.get(&url).headers(headers).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn create_online_session(&self, session_params: &Value) -> Result<String> {
        let url = format!("{}/sessions/online", self.base_url);
        let headers = self.build_headers(None);

        let response = self.client.post(&url).headers(headers).json(session_params).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn close_online_session(&self, reference_number: &str) -> Result<String> {
        let url = format!("{}/sessions/online/{}/close", self.base_url, reference_number);
        let headers = self.build_headers(None);

        let response = self.client.post(&url).headers(headers).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn submit_invoice(&self, session_ref: &str, invoice_data: &str) -> Result<String> {
        let url = format!("{}/sessions/{}/invoices", self.base_url, session_ref);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/xml".parse()?);
        let headers = self.build_headers(Some(headers));

        let response = self.client.post(&url).headers(headers).body(invoice_data.to_string()).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }
}

impl Default for KsefClient {
    fn default() -> Self {
        Self::new()
    }
}
