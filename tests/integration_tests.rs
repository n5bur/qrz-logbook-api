use chrono::{NaiveDate, NaiveTime};
use qrz_logbook_api::{
    adif::AdifParser, FetchOptions, QrzLogbookClient, QsoRecord, QrzLogbookError
};

#[tokio::test]
async fn test_client_creation_valid() {
    let client = QrzLogbookClient::new("valid-api-key-12345", "TestApp/1.0.0 (N0CALL)");
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_client_creation_invalid_key() {
    let result = QrzLogbookClient::new("", "TestApp/1.0.0 (N0CALL)");
    assert!(matches!(result, Err(QrzLogbookError::InvalidKey)));
}

#[tokio::test]
async fn test_client_creation_invalid_user_agent() {
    let result = QrzLogbookClient::new("valid-api-key-12345", "python-requests");
    assert!(matches!(result, Err(QrzLogbookError::InvalidUserAgent)));
    
    let result = QrzLogbookClient::new("valid-api-key-12345", "node-fetch");
    assert!(matches!(result, Err(QrzLogbookError::InvalidUserAgent)));
    
    let result = QrzLogbookClient::new("valid-api-key-12345", "");
    assert!(matches!(result, Err(QrzLogbookError::InvalidUserAgent)));
}

#[test]
fn test_qso_record_builder() {
    let qso = QsoRecord::builder()
        .call("W1AW")
        .station_callsign("K1ABC")
        .date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        .time_on(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
        .band("20m")
        .mode("SSB")
        .freq(14.200)
        .rst_sent("59")
        .rst_rcvd("59")
        .name("John")
        .qth("Boston, MA")
        .comment("Great signal!")
        .additional_field("gridsquare", "FN42aa")
        .build();

    assert_eq!(qso.call, "W1AW");
    assert_eq!(qso.station_callsign, "K1ABC");
    assert_eq!(qso.band, "20m");
    assert_eq!(qso.mode, "SSB");
    assert_eq!(qso.freq, Some(14.200));
    assert_eq!(qso.rst_sent, Some("59".to_string()));
    assert_eq!(qso.rst_rcvd, Some("59".to_string()));
    assert_eq!(qso.name, Some("John".to_string()));
    assert_eq!(qso.qth, Some("Boston, MA".to_string()));
    assert_eq!(qso.comment, Some("Great signal!".to_string()));
    assert_eq!(qso.additional_fields.get("gridsquare"), Some(&"FN42aa".to_string()));
}

#[test]
fn test_fetch_options() {
    let options = FetchOptions::new()
        .band("20m")
        .mode("SSB")
        .max(100)
        .after_logid(12345);

    let option_string = options.to_option_string();
    assert!(option_string.contains("BAND:20m"));
    assert!(option_string.contains("MODE:SSB"));
    assert!(option_string.contains("MAX:100"));
    assert!(option_string.contains("AFTERLOGID:12345"));
}

#[test]
fn test_fetch_options_all() {
    let options = FetchOptions::all();
    let option_string = options.to_option_string();
    assert_eq!(option_string, "ALL");
}

#[test]
fn test_fetch_options_date_range() {
    let options = FetchOptions::new()
        .date_range(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        );

    let option_string = options.to_option_string();
    assert!(option_string.contains("DATEFROM:20240101"));
    assert!(option_string.contains("DATETO:20241231"));
}

#[test]
fn test_adif_generation() {
    let qso = QsoRecord::builder()
        .call("W1AW")
        .station_callsign("K1ABC")
        .date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        .time_on(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
        .band("20m")
        .mode("SSB")
        .freq(14.200)
        .rst_sent("59")
        .rst_rcvd("59")
        .build();

    let adif = AdifParser::to_adif(&qso);
    
    assert!(adif.contains("<call:4>W1AW"));
    assert!(adif.contains("<station_callsign:5>K1ABC"));
    assert!(adif.contains("<qso_date:8>20240115"));
    assert!(adif.contains("<time_on:4>1430"));
    assert!(adif.contains("<band:3>20m"));
    assert!(adif.contains("<mode:3>SSB"));
    assert!(adif.contains("<freq:4>14.2"));
    assert!(adif.contains("<rst_sent:2>59"));
    assert!(adif.contains("<rst_rcvd:2>59"));
    assert!(adif.ends_with("<eor>"));
}

#[test]
fn test_adif_parsing() {
    let adif = "<call:4>W1AW<station_callsign:5>K1ABC<qso_date:8>20240115<time_on:4>1430<band:3>20m<mode:3>SSB<freq:5>14.20<rst_sent:2>59<rst_rcvd:2>59<eor>";
    let qsos = AdifParser::parse_adif(adif).unwrap();
    
    assert_eq!(qsos.len(), 1);
    let qso = &qsos[0];
    assert_eq!(qso.call, "W1AW");
    assert_eq!(qso.station_callsign, "K1ABC");
    assert_eq!(qso.qso_date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
    assert_eq!(qso.time_on, NaiveTime::from_hms_opt(14, 30, 0).unwrap());
    assert_eq!(qso.band, "20m");
    assert_eq!(qso.mode, "SSB");
    assert_eq!(qso.freq, Some(14.20));
    assert_eq!(qso.rst_sent, Some("59".to_string()));
    assert_eq!(qso.rst_rcvd, Some("59".to_string()));
}

#[test]
fn test_adif_parsing_multiple_records() {
    let adif = r#"<call:4>W1AW<station_callsign:5>K1ABC<qso_date:8>20240115<time_on:4>1430<band:3>20m<mode:3>SSB<eor>
<call:6>VE3XYZ<station_callsign:5>K1ABC<qso_date:8>20240115<time_on:4>1445<band:3>40m<mode:2>CW<eor>"#;
    
    let qsos = AdifParser::parse_adif(adif).unwrap();
    assert_eq!(qsos.len(), 2);
    
    assert_eq!(qsos[0].call, "W1AW");
    assert_eq!(qsos[0].band, "20m");
    assert_eq!(qsos[0].mode, "SSB");
    
    assert_eq!(qsos[1].call, "VE3XYZ");
    assert_eq!(qsos[1].band, "40m");
    assert_eq!(qsos[1].mode, "CW");
}

#[test]
fn test_adif_parsing_missing_required_field() {
    let adif = "<call:4>W1AW<qso_date:8>20240115<time_on:4>1430<band:3>20m<mode:3>SSB<eor>";
    let result = AdifParser::parse_adif(adif);
    assert!(result.is_err());
    assert!(matches!(result, Err(QrzLogbookError::AdifParse(_))));
}

#[test]
fn test_adif_parsing_invalid_date() {
    let adif = "<call:4>W1AW<station_callsign:5>K1ABC<qso_date:8>20241301<time_on:4>1430<band:3>20m<mode:3>SSB<eor>";
    let result = AdifParser::parse_adif(adif);
    assert!(result.is_err());
}

#[test]
fn test_adif_parsing_invalid_time() {
    let adif = "<call:4>W1AW<station_callsign:5>K1ABC<qso_date:8>20240115<time_on:4>2561<band:3>20m<mode:3>SSB<eor>";
    let result = AdifParser::parse_adif(adif);
    assert!(result.is_err());
}

#[test]
fn test_adif_roundtrip() {
    let original_qso = QsoRecord::builder()
        .call("W1AW")
        .station_callsign("K1ABC")
        .date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        .time_on(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
        .band("20m")
        .mode("SSB")
        .freq(14.200)
        .rst_sent("59")
        .rst_rcvd("59")
        .name("John")
        .qth("Boston")
        .comment("Test QSO")
        .additional_field("gridsquare", "FN42aa")
        .build();

    // Convert to ADIF and back
    let adif = AdifParser::to_adif(&original_qso);
    let parsed_qsos = AdifParser::parse_adif(&adif).unwrap();
    
    assert_eq!(parsed_qsos.len(), 1);
    let parsed_qso = &parsed_qsos[0];
    
    assert_eq!(parsed_qso.call, original_qso.call);
    assert_eq!(parsed_qso.station_callsign, original_qso.station_callsign);
    assert_eq!(parsed_qso.qso_date, original_qso.qso_date);
    assert_eq!(parsed_qso.time_on, original_qso.time_on);
    assert_eq!(parsed_qso.band, original_qso.band);
    assert_eq!(parsed_qso.mode, original_qso.mode);
    assert_eq!(parsed_qso.freq, original_qso.freq);
    assert_eq!(parsed_qso.rst_sent, original_qso.rst_sent);
    assert_eq!(parsed_qso.rst_rcvd, original_qso.rst_rcvd);
    assert_eq!(parsed_qso.name, original_qso.name);
    assert_eq!(parsed_qso.qth, original_qso.qth);
    assert_eq!(parsed_qso.comment, original_qso.comment);
    assert_eq!(parsed_qso.additional_fields.get("gridsquare"), Some(&"FN42aa".to_string()));
}

// Mock tests would require a mock server setup
// For now, we'll focus on unit tests and provide examples for integration testing

#[cfg(test)]
mod mock_tests {
    use super::*;
    use std::collections::HashMap;

    // Helper function to create a test client
    fn create_test_client() -> QrzLogbookClient {
        QrzLogbookClient::new("test-api-key-12345", "TestSuite/1.0.0 (N0CALL)").unwrap()
    }

    #[test]
    fn test_response_parsing_insert_success() {
        let client = create_test_client();
        let response = "RESULT=OK&LOGID=130877825&COUNT=1".to_string();
        let result = client.parse_insert_response(response).unwrap();
        
        assert_eq!(result.logid, 130877825);
        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_response_parsing_insert_failure() {
        let client = create_test_client();
        let response = "RESULT=FAIL&REASON=Invalid+QSO+data".to_string();
        let result = client.parse_insert_response(response);
        
        assert!(result.is_err());
        if let Err(QrzLogbookError::Api { reason }) = result {
            assert!(reason.contains("Invalid"));
        } else {
            panic!("Expected API error");
        }
    }

    #[test]
    fn test_response_parsing_delete_success() {
        let client = create_test_client();
        let response = "RESULT=OK&COUNT=2".to_string();
        let result = client.parse_delete_response(response).unwrap();
        
        assert_eq!(result.deleted_count, 2);
        assert!(result.not_found_logids.is_empty());
    }

    #[test]
    fn test_response_parsing_delete_partial() {
        let client = create_test_client();
        let response = "RESULT=PARTIAL&COUNT=1&LOGIDS=12346".to_string();
        let result = client.parse_delete_response(response).unwrap();
        
        assert_eq!(result.deleted_count, 1);
        assert_eq!(result.not_found_logids, vec![12346]);
    }

    #[test]
    fn test_response_parsing_status_success() {
        let client = create_test_client();
        let response = "RESULT=OK&DATA=total_qsos%3D1234%26confirmed%3D567%26dxcc_total%3D89".to_string();
        let result = client.parse_status_response(response).unwrap();
        
        assert_eq!(result.data.get("total_qsos"), Some(&"1234".to_string()));
        assert_eq!(result.data.get("confirmed"), Some(&"567".to_string()));
        assert_eq!(result.data.get("dxcc_total"), Some(&"89".to_string()));
    }

    #[test]
    fn test_response_parsing_fetch_success() {
        let client = create_test_client();
        let adif_data = "<call:4>W1AW<station_callsign:5>K1ABC<qso_date:8>20240115<time_on:4>1430<band:3>20m<mode:3>SSB<eor>";
        let response = format!("RESULT=OK&COUNT=1&LOGIDS=12345&ADIF={}", urlencoding::encode(adif_data));
        let result = client.parse_fetch_response(response).unwrap();
        
        assert_eq!(result.count, 1);
        assert_eq!(result.logids, vec![12345]);
        assert_eq!(result.qsos.len(), 1);
        assert_eq!(result.qsos[0].call, "W1AW");
    }

    #[test] 
    fn test_response_parsing_auth_error() {
        let client = create_test_client();
        let response = "RESULT=AUTH".to_string();
        let result = client.parse_insert_response(response);
        
        assert!(matches!(result, Err(QrzLogbookError::Auth)));
    }
}