//! Cryptography and cycle model for Ouroboros.
//!
//! The relay may verify signatures. It must not open sealed epoch keys.
//! Decrypted partner views never include cycle history.

#![forbid(unsafe_code)]

mod account;
mod device;
mod export;
mod model;
mod notify;
mod qr;
mod seal;
mod view;

pub use account::{hash_account_code, verify_account_code, AccountCode};
pub use device::{verify_detached, DeviceIdentity, DevicePublic};
pub use export::{export_refuses_partner_dir, export_tracker, PeriodRecord, TrackerExport};
pub use model::{CycleModel, Feedback, WELLNESS_DISCLAIMER};
pub use notify::{partner_phase_text, set_partner_sharing, EmptyWake, ReminderPlan};
pub use qr::{decode_invite, short_code, QrInvite};
pub use seal::SealedEpochKey;
pub use view::{
    decrypt_view, encrypt_view, open_partner_view, sign_view, PartnerView, PhaseWindow, SignedView,
};

/// Errors returned by identity, seal, and view operations.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CryptoError {
    /// The account code hash does not match.
    #[error("account code rejected")]
    AccountCodeRejected,
    /// A signature did not verify.
    #[error("signature rejected")]
    BadSignature,
    /// The sealed epoch key could not be opened.
    #[error("seal rejected")]
    SealRejected,
    /// The partner view ciphertext could not be opened.
    #[error("view rejected")]
    ViewRejected,
    /// The payload included fields the partner must not receive.
    #[error("partner view rejected")]
    ViewSchemaRejected,
    /// The QR payload included a forbidden field or was malformed.
    #[error("invite rejected")]
    InviteRejected,
    /// Export was pointed at the partner sandbox.
    #[error("partner directory cannot be exported")]
    PartnerDirectory,
}
