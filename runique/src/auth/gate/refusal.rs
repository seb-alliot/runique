//! One answer for every refusal. "Doesn't exist", "out of the parent", "no
//! right", "protected account" look the same from outside, so ids can't be
//! enumerated; the reason only goes to the logs.
use crate::utils::trad::t;

#[derive(Debug)]
pub struct Refusal {
    /// Logged when the refusal is made; kept on the value for the tests only.
    #[cfg(test)]
    reason: RefusalReason,
}

/// Never shown to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RefusalReason {
    NoRight,
    NotOwner,
    /// The user's rank manages no account.
    RankTooLow,
    /// A lookup the decision needed failed: refused, never let through.
    LookupFailed,
    /// A guarded write touched no row: it changed between the gate and the
    /// write (owner, rank), it is out of the user's rank, or it no longer exists.
    StaleRow,
    /// A client-sent id (foreign key, many-to-many target) the user can't read.
    UnreadableTarget,
    /// The database refused the write.
    WriteFailed,
    /// No row read: missing, out of the user's read scope, or a malformed id.
    /// The scoped read can't tell which, and doesn't need to.
    NotFound,
    /// The table right without any readable column beyond the key.
    NoReadableColumn,
}

impl Refusal {
    pub(crate) fn new(reason: RefusalReason) -> Self {
        tracing::info!(?reason, "admin action refused");
        Self {
            #[cfg(test)]
            reason,
        }
    }

    #[cfg(test)]
    pub(crate) fn reason(&self) -> &RefusalReason {
        &self.reason
    }

    /// The message shown, whatever the reason.
    pub fn message(&self) -> String {
        t("admin.access.action_not_allowed").into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_reason_shows_the_same_message() {
        let messages: Vec<String> = [
            RefusalReason::NoRight,
            RefusalReason::NotOwner,
            RefusalReason::RankTooLow,
            RefusalReason::LookupFailed,
            RefusalReason::StaleRow,
            RefusalReason::UnreadableTarget,
            RefusalReason::WriteFailed,
            RefusalReason::NotFound,
            RefusalReason::NoReadableColumn,
        ]
        .into_iter()
        .map(|r| Refusal::new(r).message())
        .collect();
        assert!(messages.iter().all(|m| m == &messages[0]), "{messages:?}");
        assert_ne!(
            messages[0], "admin.access.action_not_allowed",
            "key resolved"
        );
        assert!(!messages[0].is_empty());
        assert_eq!(messages[0], t("admin.access.action_not_allowed"));
    }
}
