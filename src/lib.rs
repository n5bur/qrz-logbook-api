//! # QRZ Logbook API Client
//!
//! A Rust client library for interacting with the QRZ Logbook API.
//!
//! ## Features
//!
//! - Insert QSO records
//! - Delete QSO records  
//! - Fetch QSO records with filtering
//! - Get logbook status
//! - Full ADIF support
//! - Type-safe API with comprehensive error handling
//!
//! ## Example
//!
//! ```rust,no_run
//! use qrz_logbook_api::{QrzLogbookClient, QsoRecord};
//! use chrono::{NaiveDate, NaiveTime};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = QrzLogbookClient::new("YOUR-API-KEY", "MyApp/1.0.0 (YOURCALL)")?;
//!     
//!     let qso = QsoRecord::builder()
//!         .call("W1AW")
//!         .station_callsign("K1ABC")
//!         .date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
//!         .time_on(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
//!         .band("20m")
//!         .mode("SSB")
//!         .build();
//!     
//!     let result = client.insert_qso(&qso, false).await?;
//!     println!("Inserted QSO with ID: {}", result.logid);
//!     
//!     Ok(())
//! }
//! ```

pub mod adif;
pub mod client;
pub mod error;
pub mod models;

pub use client::QrzLogbookClient;
pub use error::{QrzLogbookError, QrzLogbookResult};
pub use models::*;
