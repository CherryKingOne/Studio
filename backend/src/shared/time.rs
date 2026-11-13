use chrono::NaiveDateTime;

pub fn iso(dt: NaiveDateTime) -> String {
    dt.and_utc().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub fn iso_opt(dt: Option<NaiveDateTime>) -> Option<String> {
    dt.map(iso)
}
