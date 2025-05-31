//! Basic usage example for the QRZ Logbook API
//!
//! This example demonstrates how to:
//! - Create a client
//! - Insert a QSO
//! - Fetch QSOs with filtering
//! - Get logbook status
//!
//! To run this example:
//! ```
//! QRZ_API_KEY=your-api-key cargo run --example basic_usage
//! ```

use chrono::{NaiveDate, NaiveTime};
use qrz_logbook_api::{FetchOptions, QrzLogbookClient, QsoRecord};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("QRZ_API_KEY").expect("Please set QRZ_API_KEY environment variable");

    // Create client with descriptive user agent
    let client = QrzLogbookClient::new(&api_key, "BasicExample/1.0.0 (YOURCALL)")?;

    println!("QRZ Logbook API Basic Usage Example");
    println!("===================================\n");

    // Get logbook status
    println!("📊 Getting logbook status...");
    match client.get_status().await {
        Ok(status) => {
            for (key, value) in &status.data {
                println!("  {}: {}", key, value);
            }
        }
        Err(e) => println!("❌ Error getting status: {}", e),
    }

    // Create a sample QSO
    let qso = QsoRecord::builder()
        .call("W1AW")
        .station_callsign("K1ABC") // Replace with your callsign
        .date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        .time_on(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
        .time_off(NaiveTime::from_hms_opt(14, 35, 0).unwrap())
        .band("20m")
        .mode("SSB")
        .freq(14.205)
        .rst_sent("59")
        .rst_rcvd("59")
        .name("ARRL HQ")
        .qth("Newington, CT")
        .comment("Example QSO for API testing")
        .additional_field("gridsquare", "FN31")
        .build();

    // Insert the QSO (commented out to avoid adding test data)
    // Uncomment the following block to actually insert:
    /*
    println!("\n📝 Inserting sample QSO...");
    match client.insert_qso(&qso, false).await {
        Ok(result) => {
            println!("✅ QSO inserted successfully!");
            println!("  Log ID: {}", result.logid);
            println!("  Count: {}", result.count);
        }
        Err(e) => println!("❌ Error inserting QSO: {}", e),
    }
    */

    // Fetch recent QSOs
    println!("\n🔍 Fetching recent QSOs...");
    match client.fetch_qsos(&FetchOptions::new().max(10)).await {
        Ok(response) => {
            println!("✅ Found {} QSOs (showing up to 10):", response.count);
            for qso in &response.qsos {
                println!(
                    "  {} | {} | {} {} | {} | {}",
                    qso.qso_date.format("%Y-%m-%d"),
                    qso.time_on.format("%H:%M"),
                    qso.call,
                    qso.mode,
                    qso.band,
                    qso.rst_sent.as_deref().unwrap_or("--")
                );
            }
        }
        Err(e) => println!("❌ Error fetching QSOs: {}", e),
    }

    // Fetch QSOs on specific band
    println!("\n🔍 Fetching QSOs on 20m band...");
    match client
        .fetch_qsos(&FetchOptions::new().band("20m").max(5))
        .await
    {
        Ok(response) => {
            println!("✅ Found {} QSOs on 20m (showing up to 5):", response.count);
            for qso in &response.qsos {
                println!(
                    "  {} {} - {} ({})",
                    qso.qso_date.format("%Y-%m-%d"),
                    qso.call,
                    qso.mode,
                    qso.freq
                        .map(|f| format!("{:.3} MHz", f))
                        .unwrap_or_else(|| "No freq".to_string())
                );
            }
        }
        Err(e) => println!("❌ Error fetching 20m QSOs: {}", e),
    }

    println!("\n✨ Example completed!");
    Ok(())
}
