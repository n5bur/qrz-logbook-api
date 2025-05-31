# qrz-logbook-api

[![CI](https://github.com/n5bur/qrz-logbook-api/actions/workflows/ci.yml/badge.svg)](https://github.com/n5bur/qrz-logbook-api/actions/workflows/ci.yml)

[![License](https://img.shields.io/crates/l/qrz-logbook-api.svg)](https://github.com/n5bur/qrz-logbook-api)

A comprehensive, type-safe Rust client library for the [QRZ Logbook API](https://www.qrz.com/docs/logbook/QRZLogbookAPI.html). This library provides an easy-to-use interface for amateur radio operators to interact with their QRZ logbooks programmatically.

## Features

- **Complete API Coverage**: All QRZ Logbook API endpoints supported
  - Insert QSO records
  - Delete QSO records
  - Fetch QSO records with advanced filtering
  - Get logbook status and statistics
- **Type Safety**: Comprehensive error handling and type-safe operations
- **ADIF Support**: Full ADIF (Amateur Data Interchange Format) parsing and generation
- **Async/Await**: Built with modern async Rust using `tokio` and `reqwest`
- **Automatic Paging**: Built-in support for fetching large logbooks efficiently
- **Builder Pattern**: Intuitive QSO record creation with builder pattern
- **Comprehensive Documentation**: Extensive docs with examples

## Requirements

- A valid QRZ.com account
- QRZ Logbook API access key (available from your QRZ account page)
- Note: Some operations require a QRZ subscription

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qrz-logbook-api = "0.1"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
chrono = "0.4"
```

## Quick Start

```rust
use qrz_logbook_api::{QrzLogbookClient, QsoRecord, FetchOptions};
use chrono::{NaiveDate, NaiveTime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with your API key and a descriptive user agent
    let client = QrzLogbookClient::new(
        "YOUR-API-KEY", 
        "MyLogApp/1.0.0 (YOURCALL)"
    )?;

    // Create a new QSO record
    let qso = QsoRecord::builder()
        .call("W1AW")
        .station_callsign("K1ABC")
        .date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        .time_on(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
        .band("20m")
        .mode("SSB")
        .freq(14.205)
        .rst_sent("59")
        .rst_rcvd("59")
        .name("John")
        .qth("Newington, CT")
        .comment("Great signal from ARRL HQ!")
        .build();

    // Insert the QSO
    let result = client.insert_qso(&qso, false).await?;
    println!("Inserted QSO with ID: {}", result.logid);

    // Fetch QSOs from the last 30 days on 20m
    let recent_qsos = client.fetch_qsos(&FetchOptions::new()
        .band("20m")
        .max(50)
    ).await?;
    
    println!("Found {} recent QSOs on 20m", recent_qsos.count);

    Ok(())
}
```

## API Operations

### Insert QSO Records

```rust
use qrz_logbook_api::{QrzLogbookClient, QsoRecord};
use chrono::{NaiveDate, NaiveTime};

let qso = QsoRecord::builder()
    .call("JA1XYZ")
    .station_callsign("W1ABC")
    .date(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap())
    .time_on(NaiveTime::from_hms_opt(12, 30, 0).unwrap())
    .time_off(NaiveTime::from_hms_opt(12, 35, 0).unwrap()) // Optional
    .band("40m")
    .mode("FT8")
    .freq(7.074)
    .rst_sent("73")
    .rst_rcvd("73")
    .additional_field("gridsquare", "PM95")  // Custom ADIF fields
    .build();

// Insert without replacing duplicates
let result = client.insert_qso(&qso, false).await?;

// Insert and replace any existing duplicates
let result = client.insert_qso(&qso, true).await?;
```

### Fetch QSO Records

```rust
use qrz_logbook_api::FetchOptions;
use chrono::NaiveDate;

// Fetch all QSOs (with automatic paging)
let all_qsos = client.fetch_all_qsos(&FetchOptions::all()).await?;

// Fetch specific QSOs with filtering
let filtered_qsos = client.fetch_qsos(&FetchOptions::new()
    .band("20m")                    // Only 20m contacts
    .mode("CW")                     // Only CW contacts
    .max(100)                       // Limit to 100 results
    .date_range(                    // Date range filter
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()
    )
).await?;

// Paging through large result sets
let mut after_logid = 0;
loop {
    let page = client.fetch_qsos(&FetchOptions::new()
        .max(250)
        .after_logid(after_logid)
    ).await?;
    
    if page.qsos.is_empty() {
        break;
    }
    
    // Process the page...
    for qso in &page.qsos {
        println!("QSO with {}: {} on {}", qso.call, qso.band, qso.qso_date);
    }
    
    // Set up for next page
    after_logid = page.logids.iter().max().unwrap() + 1;
}
```

### Delete QSO Records

```rust
// Delete specific QSOs by logid
let result = client.delete_qsos(vec![12345, 12346, 12347]).await?;
println!("Deleted {} QSOs", result.deleted_count);

if !result.not_found_logids.is_empty() {
    println!("Could not find logids: {:?}", result.not_found_logids);
}
```

### Get Logbook Status

```rust
let status = client.get_status().await?;
for (key, value) in &status.data {
    println!("{}: {}", key, value);
}
// Example output:
// total_qsos: 1234
// confirmed: 567  
// dxcc_total: 89
// start_date: 2020-01-01
// end_date: 2030-12-31
```

## Advanced Usage

### Working with ADIF Data

```rust
use qrz_logbook_api::adif::AdifParser;

// Parse ADIF from string
let adif_string = "<call:4>W1AW<band:3>20m<mode:3>SSB<qso_date:8>20240115<time_on:4>1430<station_callsign:5>K1ABC<eor>";
let qsos = AdifParser::parse_adif(&adif_string)?;

// Generate ADIF from QSO record
let qso = QsoRecord::builder()
    .call("W1AW")
    .station_callsign("K1ABC")
    // ... other fields
    .build();
let adif = AdifParser::to_adif(&qso);
```

### Error Handling

```rust
use qrz_logbook_api::{QrzLogbookError, QrzResult};

match client.insert_qso(&qso, false).await {
    Ok(result) => println!("Success: {}", result.logid),
    Err(QrzLogbookError::Auth) => eprintln!("Authentication failed - check your API key"),
    Err(QrzLogbookError::Api { reason }) => eprintln!("API error: {}", reason),
    Err(QrzLogbookError::Http(e)) => eprintln!("Network error: {}", e),
    Err(QrzLogbookError::AdifParse(msg)) => eprintln!("ADIF parsing error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Custom ADIF Fields

```rust
let qso = QsoRecord::builder()
    .call("VK2ABC")
    .station_callsign("W1XYZ")
    // ... required fields ...
    .additional_field("gridsquare", "QF56")
    .additional_field("iota", "OC-001")
    .additional_field("sota_ref", "VK2/SM-001")
    .additional_field("my_gridsquare", "FN42")
    .build();
```

## Configuration

### User Agent Requirements

The QRZ API requires an identifiable user agent. For personal scripts, include your callsign:

```rust
// Good examples:
let client = QrzLogbookClient::new(api_key, "LogUploader/1.0.0 (W1ABC)")?;
let client = QrzLogbookClient::new(api_key, "ContestLogger/2.1.0 (VE3XYZ)")?;

// Bad examples (will be rejected):
let client = QrzLogbookClient::new(api_key, "python-requests")?;  // ❌
let client = QrzLogbookClient::new(api_key, "curl")?;             // ❌
```

### TLS Configuration

By default, the library uses `rustls`. To use the system's native TLS:

```toml
[dependencies]
qrz-logbook-api = { version = "0.1", default-features = false, features = ["native-tls"] }
```

## Error Types

- **`QrzLogbookError::Http`**: Network and HTTP errors
- **`QrzLogbookError::Api`**: API-specific errors (invalid data, etc.)
- **`QrzLogbookError::Auth`**: Authentication failures
- **`QrzLogbookError::InvalidKey`**: Invalid API key format
- **`QrzLogbookError::InvalidUserAgent`**: Invalid user agent string
- **`QrzLogbookError::AdifParse`**: ADIF parsing errors
- **`QrzLogbookError::InvalidParams`**: Invalid parameter combinations

## Testing

Run the test suite:

```bash
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Add tests for your changes
4. Ensure all tests pass (`cargo test`)
5. Commit your changes (`git commit -am 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- QRZ.com for providing the logbook API
- The amateur radio community for ADIF standardization
- All contributors to this project

## Disclaimer

This library is not officially associated with QRZ.com. QRZ is a trademark of QRZ LLC. Please respect QRZ's terms of service and API usage guidelines.

For support with the QRZ Logbook API itself, please contact QRZ.com directly.