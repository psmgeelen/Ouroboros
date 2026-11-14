use std::path::Path;

use serde::Serialize;

use crate::CryptoError;

/// One logged period included in a tracker export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PeriodRecord {
    /// `YYYY-MM-DD`.
    pub start: String,
    /// `YYYY-MM-DD`.
    pub end: String,
    /// Flow description chosen by the tracker.
    pub flow: String,
    /// Optional note.
    pub notes: String,
}

/// File the tracker writes after biometric confirm. Partner ciphertext is not a field.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TrackerExport {
    /// The owner's account code, so they can restore the same identity.
    pub account_code: String,
    /// Logged periods.
    pub periods: Vec<PeriodRecord>,
    /// Short description of the on-device model.
    pub model_summary: String,
}

/// Serialize a tracker export. The function does not accept partner bytes.
pub fn export_tracker(export: &TrackerExport) -> String {
    serde_json::to_string(export).expect("export struct is always serializable")
}

/// Refuse to export a path inside the partner sandbox.
pub fn export_refuses_partner_dir(path: &Path) -> Result<(), CryptoError> {
    let inside_partner = path
        .components()
        .any(|component| component.as_os_str() == "partner");
    if inside_partner {
        Err(CryptoError::PartnerDirectory)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_contains_period_and_account_code_only() {
        let export = TrackerExport {
            account_code: "1234567890123456".into(),
            periods: vec![PeriodRecord {
                start: "2026-01-01".into(),
                end: "2026-01-05".into(),
                flow: "medium".into(),
                notes: "cramps".into(),
            }],
            model_summary: "cycle mean 28".into(),
        };
        let planted = "partner-ciphertext-blob";
        let encoded = export_tracker(&export);
        assert!(encoded.contains("1234567890123456"));
        assert!(encoded.contains("2026-01-01"));
        assert!(!encoded.contains(planted));
        assert!(export_refuses_partner_dir(Path::new("/data/partner/cache")).is_err());
        assert!(export_refuses_partner_dir(Path::new("/data/tracker/log")).is_ok());
    }
}
