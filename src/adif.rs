use crate::{error::QrzLogbookError, models::QsoRecord, QrzLogbookResult};
use chrono::{NaiveDate, NaiveTime};
use std::collections::HashMap;

/// ADIF parser and formatter
pub struct AdifParser;

impl AdifParser {
    /// Convert QSO record to ADIF format
    pub fn to_adif(qso: &QsoRecord) -> String {
        let mut fields = Vec::new();

        // Required fields
        fields.push(format!("<call:{}>{}", qso.call.len(), qso.call));
        fields.push(format!(
            "<station_callsign:{}>{}",
            qso.station_callsign.len(),
            qso.station_callsign
        ));
        fields.push(format!("<qso_date:8>{}", qso.qso_date.format("%Y%m%d")));
        fields.push(format!(
            "<time_on:{}>{}",
            format_time(&qso.time_on).len(),
            format_time(&qso.time_on)
        ));
        fields.push(format!("<band:{}>{}", qso.band.len(), qso.band));
        fields.push(format!("<mode:{}>{}", qso.mode.len(), qso.mode));

        // Optional fields
        if let Some(ref time_off) = qso.time_off {
            let time_str = format_time(time_off);
            fields.push(format!("<time_off:{}>{}", time_str.len(), time_str));
        }

        if let Some(freq) = qso.freq {
            let freq_str = freq.to_string();
            fields.push(format!("<freq:{}>{}", freq_str.len(), freq_str));
        }

        if let Some(ref rst) = qso.rst_sent {
            fields.push(format!("<rst_sent:{}>{}", rst.len(), rst));
        }

        if let Some(ref rst) = qso.rst_rcvd {
            fields.push(format!("<rst_rcvd:{}>{}", rst.len(), rst));
        }

        if let Some(ref qth) = qso.qth {
            fields.push(format!("<qth:{}>{}", qth.len(), qth));
        }

        if let Some(ref name) = qso.name {
            fields.push(format!("<name:{}>{}", name.len(), name));
        }

        if let Some(ref comment) = qso.comment {
            fields.push(format!("<comment:{}>{}", comment.len(), comment));
        }

        // Additional fields
        for (key, value) in &qso.additional_fields {
            fields.push(format!("<{}:{}>{}", key.to_lowercase(), value.len(), value));
        }

        // End of record marker
        fields.push("<eor>".to_string());

        fields.join("")
    }

    /// Parse ADIF string into QSO records
    pub fn parse_adif(adif: &str) -> QrzLogbookResult<Vec<QsoRecord>> {
        let mut qsos = Vec::new();
        let records = adif.split("<eor>");

        for record in records {
            let record = record.trim();
            if record.is_empty() {
                continue;
            }

            let qso = Self::parse_single_record(record)?;
            qsos.push(qso);
        }

        Ok(qsos)
    }

    fn parse_single_record(record: &str) -> QrzLogbookResult<QsoRecord> {
        let mut fields = HashMap::new();
        let mut pos = 0;
        let chars: Vec<char> = record.chars().collect();

        while pos < chars.len() {
            if chars[pos] == '<' {
                // Find field name and length
                let start = pos + 1;
                let mut end = start;
                while end < chars.len() && chars[end] != ':' && chars[end] != '>' {
                    end += 1;
                }

                if end >= chars.len() {
                    break;
                }

                let field_name = chars[start..end].iter().collect::<String>().to_lowercase();

                if chars[end] == '>' {
                    // Field without length (like <eor>)
                    pos = end + 1;
                    continue;
                }

                // Find length
                let length_start = end + 1;
                let mut length_end = length_start;
                while length_end < chars.len() && chars[length_end] != '>' {
                    length_end += 1;
                }

                if length_end >= chars.len() {
                    break;
                }

                let length_str: String = chars[length_start..length_end].iter().collect();
                let length: usize = length_str.parse().map_err(|_| {
                    QrzLogbookError::adif_parse(format!("Invalid length: {}", length_str))
                })?;

                // Extract field value
                let value_start = length_end + 1;
                let value_end = value_start + length;

                if value_end > chars.len() {
                    return Err(QrzLogbookError::adif_parse(
                        "Field value extends beyond record",
                    ));
                }

                let value: String = chars[value_start..value_end].iter().collect();
                fields.insert(field_name, value);

                pos = value_end;
            } else {
                pos += 1;
            }
        }

        Self::fields_to_qso(fields)
    }

    fn fields_to_qso(fields: HashMap<String, String>) -> QrzLogbookResult<QsoRecord> {
        let mut additional_fields = fields.clone();

        // Extract required fields
        let call = additional_fields
            .remove("call")
            .ok_or_else(|| QrzLogbookError::adif_parse("Missing call field"))?;
        let station_callsign = additional_fields
            .remove("station_callsign")
            .ok_or_else(|| QrzLogbookError::adif_parse("Missing station_callsign field"))?;
        let band = additional_fields
            .remove("band")
            .ok_or_else(|| QrzLogbookError::adif_parse("Missing band field"))?;
        let mode = additional_fields
            .remove("mode")
            .ok_or_else(|| QrzLogbookError::adif_parse("Missing mode field"))?;

        // Parse date
        let qso_date_str = additional_fields
            .remove("qso_date")
            .ok_or_else(|| QrzLogbookError::adif_parse("Missing qso_date field"))?;
        let qso_date = parse_date(&qso_date_str)?;

        // Parse time
        let time_on_str = additional_fields
            .remove("time_on")
            .ok_or_else(|| QrzLogbookError::adif_parse("Missing time_on field"))?;
        let time_on = parse_time(&time_on_str)?;

        // Optional fields
        let time_off = additional_fields
            .remove("time_off")
            .map(|s| parse_time(&s))
            .transpose()?;

        let freq = additional_fields
            .remove("freq")
            .map(|s| s.parse::<f64>())
            .transpose()
            .map_err(|_| QrzLogbookError::adif_parse("Invalid frequency format"))?;

        let rst_sent = additional_fields.remove("rst_sent");
        let rst_rcvd = additional_fields.remove("rst_rcvd");
        let qth = additional_fields.remove("qth");
        let name = additional_fields.remove("name");
        let comment = additional_fields.remove("comment");

        Ok(QsoRecord {
            call,
            station_callsign,
            qso_date,
            time_on,
            time_off,
            band,
            mode,
            freq,
            rst_sent,
            rst_rcvd,
            qth,
            name,
            comment,
            additional_fields,
        })
    }
}

fn format_time(time: &NaiveTime) -> String {
    time.format("%H%M").to_string()
}

fn parse_date(date_str: &str) -> QrzLogbookResult<NaiveDate> {
    if date_str.len() != 8 {
        return Err(QrzLogbookError::adif_parse(
            "Date must be 8 characters (YYYYMMDD)",
        ));
    }

    let year: i32 = date_str[0..4]
        .parse()
        .map_err(|_| QrzLogbookError::adif_parse("Invalid year in date"))?;
    let month: u32 = date_str[4..6]
        .parse()
        .map_err(|_| QrzLogbookError::adif_parse("Invalid month in date"))?;
    let day: u32 = date_str[6..8]
        .parse()
        .map_err(|_| QrzLogbookError::adif_parse("Invalid day in date"))?;

    NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| QrzLogbookError::adif_parse("Invalid date"))
}

fn parse_time(time_str: &str) -> QrzLogbookResult<NaiveTime> {
    let time_str = if time_str.len() == 4 {
        format!("{}00", time_str) // Add seconds if not present
    } else {
        time_str.to_string()
    };

    if time_str.len() != 6 {
        return Err(QrzLogbookError::adif_parse(
            "Time must be 4 or 6 characters (HHMM or HHMMSS)",
        ));
    }

    let hour: u32 = time_str[0..2]
        .parse()
        .map_err(|_| QrzLogbookError::adif_parse("Invalid hour in time"))?;
    let minute: u32 = time_str[2..4]
        .parse()
        .map_err(|_| QrzLogbookError::adif_parse("Invalid minute in time"))?;
    let second: u32 = time_str[4..6]
        .parse()
        .map_err(|_| QrzLogbookError::adif_parse("Invalid second in time"))?;

    NaiveTime::from_hms_opt(hour, minute, second)
        .ok_or_else(|| QrzLogbookError::adif_parse("Invalid time"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_to_adif() {
        let qso = QsoRecord {
            call: "W1AW".to_string(),
            station_callsign: "K1ABC".to_string(),
            qso_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            time_on: NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
            time_off: None,
            band: "20m".to_string(),
            mode: "SSB".to_string(),
            freq: Some(14.200),
            rst_sent: Some("59".to_string()),
            rst_rcvd: Some("59".to_string()),
            qth: None,
            name: None,
            comment: None,
            additional_fields: HashMap::new(),
        };

        let adif = AdifParser::to_adif(&qso);
        assert!(adif.contains("<call:4>W1AW"));
        assert!(adif.contains("<station_callsign:5>K1ABC"));
        assert!(adif.contains("<qso_date:8>20240115"));
        assert!(adif.contains("<time_on:4>1430"));
        assert!(adif.contains("<eor>"));
    }

    #[test]
    fn test_parse_adif() {
        let adif = "<call:4>W1AW<station_callsign:5>K1ABC<qso_date:8>20240115<time_on:4>1430<band:3>20m<mode:3>SSB<eor>";
        let qsos = AdifParser::parse_adif(adif).unwrap();

        assert_eq!(qsos.len(), 1);
        let qso = &qsos[0];
        assert_eq!(qso.call, "W1AW");
        assert_eq!(qso.station_callsign, "K1ABC");
        assert_eq!(qso.band, "20m");
        assert_eq!(qso.mode, "SSB");
    }
}
