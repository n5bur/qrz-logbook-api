use crate::{
    adif::AdifParser,
    error::{QrzLogbookError, QrzLogbookResult},
    models::{
        DeleteResponse, FetchOptions, FetchResponse, InsertResponse, QsoRecord, StatusResponse,
    },
};
use reqwest::Client;
use std::collections::HashMap;

const API_ENDPOINT: &str = "https://logbook.qrz.com/api";

/// QRZ Logbook API client
pub struct QrzLogbookClient {
    client: Client,
    api_key: String,
    #[allow(dead_code)] // User agent is used for requests, but not needed in all methods
    user_agent: String,
}

impl QrzLogbookClient {
    /// Create a new QRZ Logbook client
    ///
    /// # Arguments
    /// * `api_key` - Your QRZ API access key
    /// * `user_agent` - Identifiable user agent (max 128 chars, should include callsign)
    ///
    /// # Example
    /// ```rust,no_run
    /// use qrz_logbook_api::QrzLogbookClient;
    ///
    /// let client = QrzLogbookClient::new("YOUR-API-KEY", "MyApp/1.0.0 (YOURCALL)").unwrap();
    /// ```
    pub fn new(
        api_key: impl Into<String>,
        user_agent: impl Into<String>,
    ) -> QrzLogbookResult<Self> {
        let api_key = api_key.into();
        let user_agent = user_agent.into();

        // Validate API key format (basic validation)
        if api_key.is_empty() || api_key.len() < 10 {
            return Err(QrzLogbookError::InvalidKey);
        }

        // Validate user agent
        if user_agent.is_empty() || user_agent.len() > 128 {
            return Err(QrzLogbookError::InvalidUserAgent);
        }

        // Check for generic user agents
        let lower_ua = user_agent.to_lowercase();
        if lower_ua.contains("python-requests")
            || lower_ua.contains("node-fetch")
            || lower_ua == "curl"
            || lower_ua == "wget"
        {
            return Err(QrzLogbookError::InvalidUserAgent);
        }

        let client = Client::builder().user_agent(&user_agent).build()?;

        Ok(Self {
            client,
            api_key,
            user_agent,
        })
    }

    /// Insert a single QSO record into the logbook
    ///
    /// # Arguments
    /// * `qso` - The QSO record to insert
    /// * `replace` - Whether to replace existing duplicate QSOs
    ///
    /// # Example
    /// ```rust,no_run
    /// use qrz_logbook_api::{QrzLogbookClient, QsoRecord};
    /// use chrono::{NaiveDate, NaiveTime};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = QrzLogbookClient::new("YOUR-API-KEY", "MyApp/1.0.0 (YOURCALL)")?;
    ///
    /// let qso = QsoRecord::builder()
    ///     .call("W1AW")
    ///     .station_callsign("K1ABC")
    ///     .date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
    ///     .time_on(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
    ///     .band("20m")
    ///     .mode("SSB")
    ///     .build();
    ///
    /// let result = client.insert_qso(&qso, false).await?;
    /// println!("Inserted QSO with ID: {}", result.logid);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn insert_qso(
        &self,
        qso: &QsoRecord,
        replace: bool,
    ) -> QrzLogbookResult<InsertResponse> {
        let adif = AdifParser::to_adif(qso);

        let mut params = vec![
            ("KEY", self.api_key.as_str()),
            ("ACTION", "INSERT"),
            ("ADIF", &adif),
        ];

        if replace {
            params.push(("OPTION", "REPLACE"));
        }

        let response = self.make_request(params).await?;
        self.parse_insert_response(response)
    }

    /// Delete one or more QSO records from the logbook
    ///
    /// # Arguments
    /// * `logids` - Vector of logids to delete
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = qrz_logbook_api::QrzLogbookClient::new("key", "agent")?;
    /// let result = client.delete_qsos(vec![12345, 12346]).await?;
    /// println!("Deleted {} QSOs", result.deleted_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_qsos(&self, logids: Vec<u64>) -> QrzLogbookResult<DeleteResponse> {
        if logids.is_empty() {
            return Err(QrzLogbookError::invalid_params("No logids provided"));
        }

        let logids_str = logids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let params = vec![
            ("KEY", self.api_key.as_str()),
            ("ACTION", "DELETE"),
            ("LOGIDS", &logids_str),
        ];

        let response = self.make_request(params).await?;
        self.parse_delete_response(response)
    }

    /// Get status information about the logbook
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = qrz_logbook_api::QrzLogbookClient::new("key", "agent")?;
    /// let status = client.get_status().await?;
    /// for (key, value) in &status.data {
    ///     println!("{}: {}", key, value);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_status(&self) -> QrzLogbookResult<StatusResponse> {
        let params = vec![("KEY", self.api_key.as_str()), ("ACTION", "STATUS")];

        let response = self.make_request(params).await?;
        self.parse_status_response(response)
    }

    /// Fetch QSO records from the logbook with optional filtering
    ///
    /// # Arguments
    /// * `options` - Fetch options for filtering
    ///
    /// # Example
    /// ```rust,no_run
    /// use qrz_logbook_api::FetchOptions;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = qrz_logbook_api::QrzLogbookClient::new("key", "agent")?;
    /// // Fetch all QSOs
    /// let all_qsos = client.fetch_qsos(&FetchOptions::all()).await?;
    ///
    /// // Fetch QSOs on 20m band
    /// let twenty_meter_qsos = client.fetch_qsos(
    ///     &FetchOptions::new().band("20m").max(100)
    /// ).await?;
    ///
    /// println!("Found {} QSOs", twenty_meter_qsos.count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_qsos(&self, options: &FetchOptions) -> QrzLogbookResult<FetchResponse> {
        let option_string = options.to_option_string();

        let mut params = vec![("KEY", self.api_key.as_str()), ("ACTION", "FETCH")];

        if !option_string.is_empty() {
            params.push(("OPTION", &option_string));
        }

        let response = self.make_request(params).await?;
        self.parse_fetch_response(response)
    }

    /// Fetch QSOs with automatic paging
    ///
    /// This method automatically handles paging to retrieve all QSOs matching the criteria.
    /// It fetches QSOs in batches of 250 to avoid timeouts.
    ///
    /// # Arguments
    /// * `options` - Fetch options for filtering (max will be overridden for paging)
    ///
    /// # Example
    /// ```rust,no_run
    /// use qrz_logbook_api::FetchOptions;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = qrz_logbook_api::QrzLogbookClient::new("key", "agent")?;
    /// let all_qsos = client.fetch_all_qsos(&FetchOptions::new().band("20m")).await?;
    /// println!("Retrieved {} QSOs total", all_qsos.len());
    /// # Ok(())
    /// # }
    /// ```
    /// This method will continue fetching QSOs until no more records are available or the API returns an empty page.
    pub async fn fetch_all_qsos(&self, options: &FetchOptions) -> QrzLogbookResult<Vec<QsoRecord>> {
        let mut all_qsos = Vec::new();
        let mut after_logid = 0u64;
        let page_size = 250u32;

        loop {
            let mut page_options = options.clone();
            page_options.max = Some(page_size);
            page_options.after_logid = if after_logid > 0 {
                Some(after_logid)
            } else {
                None
            };

            let response = self.fetch_qsos(&page_options).await?;

            if response.qsos.is_empty() {
                break;
            }

            // Find the highest logid for next page
            if let Some(max_logid) = response.logids.iter().max() {
                after_logid = max_logid + 1;
            }

            all_qsos.extend(response.qsos.clone());

            // If we got fewer records than requested, we're done
            if response.qsos.len() < page_size as usize {
                break;
            }
        }

        Ok(all_qsos)
    }

    async fn make_request(&self, params: Vec<(&str, &str)>) -> QrzLogbookResult<String> {
        let response = self.client.post(API_ENDPOINT).form(&params).send().await?;

        if !response.status().is_success() {
            return Err(QrzLogbookError::Http(
                response.error_for_status().unwrap_err(),
            ));
        }

        Ok(response.text().await?)
    }

    /// Parse the response from an INSERT action
    pub fn parse_insert_response(&self, response: String) -> QrzLogbookResult<InsertResponse> {
        let params = self.parse_response_params(&response)?;

        match params.get("RESULT").map(|s| s.as_str()) {
            Some("OK") => {
                let logid = params
                    .get("LOGID")
                    .ok_or_else(|| QrzLogbookError::api_error("Missing LOGID in response"))?
                    .parse()
                    .map_err(|_| QrzLogbookError::api_error("Invalid LOGID format"))?;

                let count = params
                    .get("COUNT")
                    .unwrap_or(&"1".to_string())
                    .parse()
                    .map_err(|_| QrzLogbookError::api_error("Invalid COUNT format"))?;

                Ok(InsertResponse { logid, count })
            }
            Some("FAIL") => {
                let reason = params
                    .get("REASON")
                    .map(|s| s.as_str())
                    .unwrap_or("Unknown error");
                Err(QrzLogbookError::api_error(reason))
            }
            Some("AUTH") => Err(QrzLogbookError::Auth),
            _ => Err(QrzLogbookError::api_error("Unexpected response format")),
        }
    }

    /// Parse the response from a DELETE action
    pub fn parse_delete_response(&self, response: String) -> QrzLogbookResult<DeleteResponse> {
        let params = self.parse_response_params(&response)?;

        match params.get("RESULT").map(|s| s.as_str()) {
            Some("OK") | Some("PARTIAL") => {
                let deleted_count = params
                    .get("COUNT")
                    .unwrap_or(&"0".to_string())
                    .parse()
                    .map_err(|_| QrzLogbookError::api_error("Invalid COUNT format"))?;

                let not_found_logids = if let Some(logids_str) = params.get("LOGIDS") {
                    logids_str
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect()
                } else {
                    Vec::new()
                };

                Ok(DeleteResponse {
                    deleted_count,
                    not_found_logids,
                })
            }
            Some("FAIL") => {
                let reason = params
                    .get("REASON")
                    .map(|s| s.as_str())
                    .unwrap_or("Unknown error");
                Err(QrzLogbookError::api_error(reason))
            }
            Some("AUTH") => Err(QrzLogbookError::Auth),
            _ => Err(QrzLogbookError::api_error("Unexpected response format")),
        }
    }

    /// Parse the response from a STATUS action
    pub fn parse_status_response(&self, response: String) -> QrzLogbookResult<StatusResponse> {
        let params = self.parse_response_params(&response)?;

        match params.get("RESULT").map(|s| s.as_str()) {
            Some("OK") => {
                let data = if let Some(data_str) = params.get("DATA") {
                    self.parse_data_params(data_str)?
                } else {
                    HashMap::new()
                };

                Ok(StatusResponse { data })
            }
            Some("FAIL") => {
                let reason = params
                    .get("REASON")
                    .map(|s| s.as_str())
                    .unwrap_or("Unknown error");
                Err(QrzLogbookError::api_error(reason))
            }
            Some("AUTH") => Err(QrzLogbookError::Auth),
            _ => Err(QrzLogbookError::api_error("Unexpected response format")),
        }
    }
    /// Parse the response from a FETCH action
    pub fn parse_fetch_response(&self, response: String) -> QrzLogbookResult<FetchResponse> {
        let params = self.parse_response_params(&response)?;

        match params.get("RESULT").map(|s| s.as_str()) {
            Some("OK") => {
                let count = params
                    .get("COUNT")
                    .unwrap_or(&"0".to_string())
                    .parse()
                    .map_err(|_| QrzLogbookError::api_error("Invalid COUNT format"))?;

                let logids = if let Some(logids_str) = params.get("LOGIDS") {
                    logids_str
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect()
                } else {
                    Vec::new()
                };

                let qsos = if let Some(adif_str) = params.get("ADIF") {
                    AdifParser::parse_adif(adif_str)?
                } else {
                    Vec::new()
                };

                Ok(FetchResponse {
                    count,
                    logids,
                    qsos,
                })
            }
            Some("FAIL") => {
                let reason = params
                    .get("REASON")
                    .map(|s| s.as_str())
                    .unwrap_or("Unknown error");
                Err(QrzLogbookError::api_error(reason))
            }
            Some("AUTH") => Err(QrzLogbookError::Auth),
            _ => Err(QrzLogbookError::api_error("Unexpected response format")),
        }
    }

    fn parse_response_params(&self, response: &str) -> QrzLogbookResult<HashMap<String, String>> {
        let mut params = HashMap::new();

        for pair in response.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                params.insert(
                    urlencoding::decode(key)
                        .map_err(|_| {
                            QrzLogbookError::api_error("Invalid URL encoding in response")
                        })?
                        .to_string(),
                    urlencoding::decode(value)
                        .map_err(|_| {
                            QrzLogbookError::api_error("Invalid URL encoding in response")
                        })?
                        .to_string(),
                );
            }
        }

        Ok(params)
    }

    fn parse_data_params(&self, data: &str) -> QrzLogbookResult<HashMap<String, String>> {
        let mut params = HashMap::new();

        for pair in data.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                params.insert(key.to_string(), value.to_string());
            }
        }

        Ok(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = QrzLogbookClient::new("test-api-key-12345", "TestApp/1.0.0 (N0CALL)");
        assert!(client.is_ok());
    }

    #[test]
    fn test_invalid_api_key() {
        let client = QrzLogbookClient::new("", "TestApp/1.0.0 (N0CALL)");
        assert!(matches!(client, Err(QrzLogbookError::InvalidKey)));
    }

    #[test]
    fn test_invalid_user_agent() {
        let client = QrzLogbookClient::new("test-api-key-12345", "python-requests");
        assert!(matches!(client, Err(QrzLogbookError::InvalidUserAgent)));
    }

    #[test]
    fn test_parse_response_params() {
        let client = QrzLogbookClient::new("test-api-key-12345", "TestApp/1.0.0 (N0CALL)").unwrap();
        let response = "RESULT=OK&LOGID=12345&COUNT=1";
        let params = client.parse_response_params(response).unwrap();

        assert_eq!(params.get("RESULT"), Some(&"OK".to_string()));
        assert_eq!(params.get("LOGID"), Some(&"12345".to_string()));
        assert_eq!(params.get("COUNT"), Some(&"1".to_string()));
    }
}
