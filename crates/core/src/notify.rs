use serde::Serialize;

/// Body of a partner wake. It has no phase label.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct EmptyWake {
    /// Always `sync`.
    pub wake: &'static str,
}

impl EmptyWake {
    /// Build the only wake body the relay sends.
    pub fn sync() -> Self {
        Self { wake: "sync" }
    }

    /// JSON used as the push body.
    pub fn json(&self) -> String {
        serde_json::to_string(self).expect("wake body is static")
    }
}

/// Phase text for a partner notification. `None` means the notification is suppressed.
pub fn partner_phase_text(sealed_key_present: bool, phase_label: &str) -> Option<String> {
    if sealed_key_present {
        Some(phase_label.to_string())
    } else {
        None
    }
}

/// Local tracker reminders and the separate partner-facing schedule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderPlan {
    /// Reminders that stay on the tracker phone.
    pub tracker_local: Vec<String>,
    /// Wakes shared with a partner.
    pub partner_facing: Vec<String>,
}

/// Clear partner wakes when sharing is turned off. Tracker reminders stay.
pub fn set_partner_sharing(plan: &mut ReminderPlan, enabled: bool) {
    if !enabled {
        plan.partner_facing.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wake_body_has_no_phase_label() {
        let json = EmptyWake::sync().json();
        assert_eq!(json, r#"{"wake":"sync"}"#);
        assert!(!json.contains("phase"));
        assert!(!json.contains("period"));
    }

    #[test]
    fn revoked_partnership_suppresses_phase_text() {
        assert_eq!(partner_phase_text(false, "period"), None);
        assert_eq!(
            partner_phase_text(true, "period").as_deref(),
            Some("period")
        );
    }

    #[test]
    fn sharing_off_keeps_tracker_reminders() {
        let mut plan = ReminderPlan {
            tracker_local: vec!["period window".into()],
            partner_facing: vec!["partner wake".into()],
        };
        set_partner_sharing(&mut plan, false);
        assert_eq!(plan.tracker_local, vec!["period window".to_string()]);
        assert!(plan.partner_facing.is_empty());
    }
}
