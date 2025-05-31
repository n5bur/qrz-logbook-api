use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// QSO record for the logbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QsoRecord {
    /// Called station's callsign
    pub call: String,
    /// Station callsign (your callsign)
    pub station_callsign: String,
    /// QSO date
    pub qso_date: NaiveDate,
    /// Time on (start time)
    pub time_on: NaiveTime,
    /// Time off (end time, optional)
    pub time_off: Option<NaiveTime>,
    /// Band (e.g., "20m", "40m")
    pub band: String,
    /// Mode (e.g., "SSB", "CW", "FT8")
    pub mode: String,
    /// Frequency in MHz (optional)
    pub freq: Option<f64>,
    /// RST sent (optional)
    pub rst_sent: Option<String>,
    /// RST received (optional)
    pub rst_rcvd: Option<String>,
    /// QTH (location, optional)
    pub qth: Option<String>,
    /// Name (optional)
    pub name: Option<String>,
    /// Comments (optional)
    pub comment: Option<String>,
    /// Additional ADIF fields
    pub additional_fields: HashMap<String, String>,
}

impl QsoRecord {
    /// Create a new QSO record builder
    pub fn builder() -> QsoRecordBuilder {
        QsoRecordBuilder::new()
    }
}

/// Builder for QSO records
#[derive(Debug, Default)]
pub struct QsoRecordBuilder {
    call: Option<String>,
    station_callsign: Option<String>,
    qso_date: Option<NaiveDate>,
    time_on: Option<NaiveTime>,
    time_off: Option<NaiveTime>,
    band: Option<String>,
    mode: Option<String>,
    freq: Option<f64>,
    rst_sent: Option<String>,
    rst_rcvd: Option<String>,
    qth: Option<String>,
    name: Option<String>,
    comment: Option<String>,
    additional_fields: HashMap<String, String>,
}

impl QsoRecordBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn call(mut self, call: impl Into<String>) -> Self {
        self.call = Some(call.into());
        self
    }

    pub fn station_callsign(mut self, callsign: impl Into<String>) -> Self {
        self.station_callsign = Some(callsign.into());
        self
    }

    pub fn date(mut self, date: NaiveDate) -> Self {
        self.qso_date = Some(date);
        self
    }

    pub fn time_on(mut self, time: NaiveTime) -> Self {
        self.time_on = Some(time);
        self
    }

    pub fn time_off(mut self, time: NaiveTime) -> Self {
        self.time_off = Some(time);
        self
    }

    pub fn band(mut self, band: impl Into<String>) -> Self {
        self.band = Some(band.into());
        self
    }

    pub fn mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = Some(mode.into());
        self
    }

    pub fn freq(mut self, freq: f64) -> Self {
        self.freq = Some(freq);
        self
    }

    pub fn rst_sent(mut self, rst: impl Into<String>) -> Self {
        self.rst_sent = Some(rst.into());
        self
    }

    pub fn rst_rcvd(mut self, rst: impl Into<String>) -> Self {
        self.rst_rcvd = Some(rst.into());
        self
    }

    pub fn qth(mut self, qth: impl Into<String>) -> Self {
        self.qth = Some(qth.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    pub fn additional_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional_fields.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> QsoRecord {
        QsoRecord {
            call: self.call.unwrap_or_default(),
            station_callsign: self.station_callsign.unwrap_or_default(),
            qso_date: self
                .qso_date
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()),
            time_on: self
                .time_on
                .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
            time_off: self.time_off,
            band: self.band.unwrap_or_default(),
            mode: self.mode.unwrap_or_default(),
            freq: self.freq,
            rst_sent: self.rst_sent,
            rst_rcvd: self.rst_rcvd,
            qth: self.qth,
            name: self.name,
            comment: self.comment,
            additional_fields: self.additional_fields,
        }
    }
}

/// Response from INSERT action
#[derive(Debug, Clone)]
pub struct InsertResponse {
    pub logid: u64,
    pub count: u32,
}

/// Response from DELETE action
#[derive(Debug, Clone)]
pub struct DeleteResponse {
    pub deleted_count: u32,
    pub not_found_logids: Vec<u64>,
}

/// Response from STATUS action
#[derive(Debug, Clone)]
pub struct StatusResponse {
    pub data: HashMap<String, String>,
}

/// Response from FETCH action
#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub count: u32,
    pub logids: Vec<u64>,
    pub qsos: Vec<QsoRecord>,
}

/// Fetch options for filtering QSOs
#[derive(Debug, Clone, Default)]
pub struct FetchOptions {
    /// Fetch all records
    pub all: bool,
    /// Filter by band
    pub band: Option<String>,
    /// Filter by mode
    pub mode: Option<String>,
    /// Filter by callsign
    pub call: Option<String>,
    /// Maximum number of records to return
    pub max: Option<u32>,
    /// Start after this logid for paging
    pub after_logid: Option<u64>,
    /// Filter by date range (start)
    pub date_from: Option<NaiveDate>,
    /// Filter by date range (end)
    pub date_to: Option<NaiveDate>,
}

impl FetchOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all() -> Self {
        Self {
            all: true,
            ..Default::default()
        }
    }

    pub fn band(mut self, band: impl Into<String>) -> Self {
        self.band = Some(band.into());
        self
    }

    pub fn mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = Some(mode.into());
        self
    }

    pub fn call(mut self, call: impl Into<String>) -> Self {
        self.call = Some(call.into());
        self
    }

    pub fn max(mut self, max: u32) -> Self {
        self.max = Some(max);
        self
    }

    pub fn after_logid(mut self, logid: u64) -> Self {
        self.after_logid = Some(logid);
        self
    }

    pub fn date_range(mut self, from: NaiveDate, to: NaiveDate) -> Self {
        self.date_from = Some(from);
        self.date_to = Some(to);
        self
    }

    /// Convert to API option string
    pub fn to_option_string(&self) -> String {
        let mut options = Vec::new();

        if self.all {
            options.push("ALL".to_string());
        }

        if let Some(ref band) = self.band {
            options.push(format!("BAND:{}", band));
        }

        if let Some(ref mode) = self.mode {
            options.push(format!("MODE:{}", mode));
        }

        if let Some(ref call) = self.call {
            options.push(format!("CALL:{}", call));
        }

        if let Some(max) = self.max {
            options.push(format!("MAX:{}", max));
        }

        if let Some(logid) = self.after_logid {
            options.push(format!("AFTERLOGID:{}", logid));
        }

        if let Some(date) = self.date_from {
            options.push(format!("DATEFROM:{}", date.format("%Y%m%d")));
        }

        if let Some(date) = self.date_to {
            options.push(format!("DATETO:{}", date.format("%Y%m%d")));
        }

        options.join(",")
    }
}
