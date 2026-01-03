use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rsa::{Oaep, RsaPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};

const DEFAULT_API_BASE_URL: &str = "https://api-test.ksef.mf.gov.pl/v2";

// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallengeResponse {
    pub challenge: String,
    pub timestamp: String,
    #[serde(rename = "timestampMs")]
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextIdentifier {
    #[serde(rename = "type")]
    pub identifier_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitTokenRequest {
    pub challenge: String,
    #[serde(rename = "contextIdentifier")]
    pub context_identifier: ContextIdentifier,
    #[serde(rename = "encryptedToken")]
    pub encrypted_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub token: String,
    #[serde(rename = "validUntil")]
    pub valid_until: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInitResponse {
    #[serde(rename = "referenceNumber")]
    pub reference_number: String,
    #[serde(rename = "authenticationToken")]
    pub authentication_token: TokenInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatusResponse {
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "authenticationMethod")]
    pub authentication_method: String,
    pub status: StatusInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusInfo {
    pub code: i32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokensResponse {
    #[serde(rename = "accessToken")]
    pub access_token: TokenInfo,
    #[serde(rename = "refreshToken")]
    pub refresh_token: TokenInfo,
}

// Session state
#[derive(Debug, Clone)]
struct SessionState {
    access_token: String,
    refresh_token: String,
    #[allow(dead_code)]
    ksef_token: String, // Original KSeF token for re-authentication
    nip: String, // NIP for re-authentication
}

pub struct KsefClient {
    client: reqwest::Client,
    base_url: String,
    session_state: Arc<Mutex<Option<SessionState>>>,
    disable_encryption: bool,
}

impl KsefClient {
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_API_BASE_URL.to_string())
    }

    pub fn with_base_url(base_url: String) -> Self {
        // Check if encryption should be disabled (for test environment)
        let disable_encryption = std::env::var("KSEF_DISABLE_ENCRYPTION")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);

        if disable_encryption {
            eprintln!("⚠️  Token encryption DISABLED (test environment)");
        }

        Self {
            client: reqwest::Client::new(),
            base_url,
            session_state: Arc::new(Mutex::new(None)),
            disable_encryption,
        }
    }

    fn get_access_token(&self) -> Option<String> {
        self.session_state
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| s.access_token.clone())
    }

    fn build_headers(
        &self,
        extra_headers: Option<reqwest::header::HeaderMap>,
    ) -> reqwest::header::HeaderMap {
        let mut headers = extra_headers.unwrap_or_default();
        if let Some(token) = self.get_access_token() {
            if let Ok(value) = format!("Bearer {}", token).parse() {
                headers.insert("Authorization", value);
            }
        }
        headers
    }

    // Authentication Methods

    /// Step 1: Get authentication challenge
    pub async fn get_auth_challenge(&self) -> Result<AuthChallengeResponse> {
        let url = format!("{}/auth/challenge", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let challenge: AuthChallengeResponse = response.json().await?;
            Ok(challenge)
        } else {
            let body = response.text().await?;
            Err(anyhow!("Failed to get challenge ({}): {}", status, body))
        }
    }

    /// Helper: Encrypt KSeF token with RSA-OAEP
    fn encrypt_token(ksef_token: &str, timestamp_ms: i64, cert_base64: &str) -> Result<String> {
        use rsa::pkcs8::DecodePublicKey;
        use sha2::Sha256;

        // Format: token|timestampMs
        let payload = format!("{}|{}", ksef_token, timestamp_ms);

        // Decode base64 certificate
        let cert_der = BASE64
            .decode(cert_base64.as_bytes())
            .map_err(|e| anyhow!("Failed to decode certificate base64: {}", e))?;

        // Parse X.509 certificate
        let (_, cert) = x509_parser::parse_x509_certificate(&cert_der)
            .map_err(|e| anyhow!("Failed to parse X.509 certificate: {}", e))?;

        // Extract public key from certificate (SubjectPublicKeyInfo format)
        let public_key_der = cert.public_key().raw;

        // Parse RSA public key from DER (SubjectPublicKeyInfo / PKCS#8)
        let public_key = RsaPublicKey::from_public_key_der(public_key_der)
            .map_err(|e| anyhow!("Failed to parse RSA public key from certificate: {}", e))?;

        // Encrypt using RSA-OAEP with SHA-256
        let padding = Oaep::new::<Sha256>();
        let encrypted = public_key
            .encrypt(&mut rand::thread_rng(), padding, payload.as_bytes())
            .map_err(|e| anyhow!("Failed to encrypt token: {}", e))?;

        // Encode to Base64
        Ok(BASE64.encode(&encrypted))
    }

    /// Step 2: Authenticate with KSeF token
    pub async fn authenticate_with_ksef_token(
        &self,
        nip: &str,
        ksef_token: &str,
        cert_base64: &str,
    ) -> Result<AuthInitResponse> {
        // Get challenge
        let challenge = self.get_auth_challenge().await?;

        // Encrypt token (or send base64-encoded plain token in test environment)
        let encrypted_token = if self.disable_encryption {
            eprintln!("Test mode: encoding token as base64 (no encryption)");
            // Format: token|timestamp (as per API spec)
            let payload = format!("{}|{}", ksef_token, challenge.timestamp_ms);
            BASE64.encode(payload.as_bytes())
        } else {
            eprintln!("Production mode: encrypting token with RSA-OAEP");
            Self::encrypt_token(ksef_token, challenge.timestamp_ms, cert_base64)?
        };

        // Prepare request
        let request = InitTokenRequest {
            challenge: challenge.challenge.clone(),
            context_identifier: ContextIdentifier {
                identifier_type: "nip".to_string(), // lowercase 'nip' as shown in your curl
                value: nip.to_string(),
            },
            encrypted_token,
        };

        // Send authentication request
        let url = format!("{}/auth/ksef-token", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let auth_response: AuthInitResponse = response.json().await?;
            Ok(auth_response)
        } else {
            let body = response.text().await?;
            Err(anyhow!("Authentication failed ({}): {}", status, body))
        }
    }

    /// Step 3: Check authentication status
    pub async fn check_auth_status(
        &self,
        reference_number: &str,
        auth_token: &str,
    ) -> Result<AuthStatusResponse> {
        let url = format!("{}/auth/{}", self.base_url, reference_number);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let status_response: AuthStatusResponse = response.json().await?;
            Ok(status_response)
        } else {
            let body = response.text().await?;
            Err(anyhow!(
                "Failed to check auth status ({}): {}",
                status,
                body
            ))
        }
    }

    /// Step 4: Redeem tokens
    pub async fn redeem_tokens(&self, auth_token: &str) -> Result<TokensResponse> {
        let url = format!("{}/auth/token/redeem", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let tokens: TokensResponse = response.json().await?;
            Ok(tokens)
        } else {
            let body = response.text().await?;
            Err(anyhow!("Failed to redeem tokens ({}): {}", status, body))
        }
    }

    /// Get public key certificate for token encryption
    async fn get_encryption_certificate(&self) -> Result<String> {
        #[derive(Deserialize)]
        struct PublicKeyCertificate {
            certificate: String,
            usage: Vec<String>,
        }

        let url = format!("{}/security/public-key-certificates", self.base_url);
        let response = self.client.get(&url).send().await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await?;
            return Err(anyhow!("Failed to get certificates ({}): {}", status, body));
        }

        let certificates: Vec<PublicKeyCertificate> = response.json().await?;

        // Find certificate for KsefTokenEncryption
        let cert = certificates
            .iter()
            .find(|c| c.usage.contains(&"KsefTokenEncryption".to_string()))
            .ok_or_else(|| anyhow!("No certificate found for KsefTokenEncryption"))?;

        // Return the base64-encoded certificate (will be parsed by encrypt_token)
        Ok(cert.certificate.clone())
    }

    /// Complete authentication flow (automatically fetches public key if needed)
    pub async fn authenticate(&self, nip: &str, ksef_token: &str) -> Result<String> {
        // Step 0: Get public key certificate (only if encryption is enabled)
        let cert_base64 = if self.disable_encryption {
            String::new() // Not needed in test mode
        } else {
            eprintln!("Fetching public key certificate...");
            self.get_encryption_certificate().await?
        };

        // Step 1 & 2: Initiate authentication
        eprintln!("Initiating authentication...");
        let auth_init = self
            .authenticate_with_ksef_token(nip, ksef_token, &cert_base64)
            .await?;
        let auth_token = auth_init.authentication_token.token.clone();
        let reference_number = auth_init.reference_number.clone();

        // Step 3: Poll for status (simplified - you may want to add retries)
        eprintln!("Checking authentication status...");
        let status = self
            .check_auth_status(&reference_number, &auth_token)
            .await?;

        if status.status.code != 100 && status.status.code != 200 {
            return Err(anyhow!(
                "Authentication failed with status {}: {}",
                status.status.code,
                status.status.description
            ));
        }

        // Step 4: Redeem tokens
        eprintln!("Redeeming access tokens...");
        let tokens = self.redeem_tokens(&auth_token).await?;

        // Store session state
        let mut state = self.session_state.lock().unwrap();
        *state = Some(SessionState {
            access_token: tokens.access_token.token.clone(),
            refresh_token: tokens.refresh_token.token.clone(),
            ksef_token: ksef_token.to_string(),
            nip: nip.to_string(),
        });

        Ok(format!(
            "Authentication successful. Access token valid until: {}",
            tokens.access_token.valid_until
        ))
    }

    /// Refresh access token
    pub async fn refresh_access_token(&self) -> Result<String> {
        let refresh_token = {
            let state = self.session_state.lock().unwrap();
            match &*state {
                Some(s) => s.refresh_token.clone(),
                None => return Err(anyhow!("No refresh token available")),
            }
        };

        let url = format!("{}/auth/token/refresh", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", refresh_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            #[derive(Deserialize)]
            struct RefreshResponse {
                #[serde(rename = "accessToken")]
                access_token: TokenInfo,
            }
            let refresh_response: RefreshResponse = response.json().await?;

            // Update access token in state
            let mut state = self.session_state.lock().unwrap();
            if let Some(s) = state.as_mut() {
                s.access_token = refresh_response.access_token.token.clone();
            }

            Ok(format!(
                "Token refreshed. Valid until: {}",
                refresh_response.access_token.valid_until
            ))
        } else {
            let body = response.text().await?;
            Err(anyhow!("Failed to refresh token ({}): {}", status, body))
        }
    }

    /// Get authentication status
    pub fn get_auth_status(&self) -> Result<String> {
        let state = self.session_state.lock().unwrap();
        match &*state {
            None => Ok("Not authenticated".to_string()),
            Some(s) => Ok(format!(
                "Authenticated as NIP: {}\nAccess token available",
                s.nip
            )),
        }
    }

    /// Logout
    pub fn logout(&self) -> Result<String> {
        let mut state = self.session_state.lock().unwrap();
        *state = None;
        Ok("Session cleared successfully".to_string())
    }

    pub async fn get_active_sessions(
        &self,
        page_size: i64,
        continuation_token: Option<&str>,
    ) -> Result<String> {
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

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(query)
            .send()
            .await?;
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

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(export_params)
            .send()
            .await?;
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

        eprintln!("Creating online session with params: {}", session_params);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(session_params)
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;

        eprintln!("Response status: {}, body: {}", status, body);

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn close_online_session(&self, reference_number: &str) -> Result<String> {
        let url = format!(
            "{}/sessions/online/{}/close",
            self.base_url, reference_number
        );
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

    pub async fn submit_invoice(&self, session_ref: &str, invoice_data: &Value) -> Result<String> {
        let url = format!("{}/sessions/online/{}/invoices", self.base_url, session_ref);
        let headers = self.build_headers(None);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(invoice_data)
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn get_sessions(
        &self,
        page_size: i64,
        continuation_token: Option<&str>,
    ) -> Result<String> {
        let url = format!("{}/sessions?pageSize={}", self.base_url, page_size);

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

    pub async fn get_session_status(&self, reference_number: &str) -> Result<String> {
        let url = format!("{}/sessions/{}", self.base_url, reference_number);
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

    pub async fn get_session_invoices(
        &self,
        reference_number: &str,
        continuation_token: Option<&str>,
    ) -> Result<String> {
        let url = format!("{}/sessions/{}/invoices", self.base_url, reference_number);

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

    pub async fn get_invoice_upo_by_ksef(
        &self,
        session_ref: &str,
        ksef_number: &str,
    ) -> Result<String> {
        let url = format!(
            "{}/sessions/{}/invoices/ksef/{}/upo",
            self.base_url, session_ref, ksef_number
        );
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

    pub async fn get_invoice_upo_by_reference(
        &self,
        session_ref: &str,
        invoice_ref: &str,
    ) -> Result<String> {
        let url = format!(
            "{}/sessions/{}/invoices/{}/upo",
            self.base_url, session_ref, invoice_ref
        );
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

    pub async fn get_session_upo(
        &self,
        session_ref: &str,
        upo_ref: &str,
    ) -> Result<String> {
        let url = format!(
            "{}/sessions/{}/upo/{}",
            self.base_url, session_ref, upo_ref
        );
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

    pub async fn create_batch_session(&self, session_params: &Value) -> Result<String> {
        let url = format!("{}/sessions/batch", self.base_url);
        let headers = self.build_headers(None);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(session_params)
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("API error ({}): {}", status, body))
        }
    }

    pub async fn close_batch_session(&self, reference_number: &str) -> Result<String> {
        let url = format!(
            "{}/sessions/batch/{}/close",
            self.base_url, reference_number
        );
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
}

impl Default for KsefClient {
    fn default() -> Self {
        Self::new()
    }
}
