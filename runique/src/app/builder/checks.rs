//! The checks before anything is built: each staging, then what they need
//! of one another.
use super::RuniqueAppBuilder;
use crate::{
    config::server::secret_key_is_weak, errors::build::BuildError, mailer::MailerConfig,
    translation::t,
};

impl<P> RuniqueAppBuilder<P> {
    pub(super) fn validate(&self) -> Result<(), BuildError> {
        self.core.validate()?;
        self.middleware.validate()?;
        self.statics.validate()?;
        #[cfg(feature = "admin")]
        self.admin.validate()?;
        self.cross_validate()
    }

    fn cross_validate(&self) -> Result<(), BuildError> {
        if self.config.dev {
            return Ok(());
        }

        use crate::errors::build::{CheckError, CheckReport};

        let mut report = CheckReport::new();
        #[cfg(feature = "acme")]
        let sec = &self.config.security;
        let srv = &self.config.server;

        if secret_key_is_weak(&srv.secret_key) {
            report.add(
                CheckError::new(
                    "Security",
                    t(runique_i18n::keys::build::error::SECRET_KEY_WEAK),
                )
                .with_suggestion(t(runique_i18n::keys::build::hint::SECRET_KEY_WEAK)),
            );
        }

        // Without a public URL, a reset link would be built from the request's
        // `Host` header: someone asking a reset for a victim's address with
        // `Host: evil.com` would get the victim's token mailed in a link to
        // their own site.
        #[cfg(feature = "admin")]
        let admin_enabled = self.admin.enabled;
        #[cfg(not(feature = "admin"))]
        let admin_enabled = false;
        if (self.password_reset.is_some() || admin_enabled) && srv.public_url.is_none() {
            report.add(
                CheckError::new(
                    "PublicUrl",
                    t(runique_i18n::keys::build::error::PUBLIC_URL_MISSING),
                )
                .with_suggestion(t(runique_i18n::keys::build::hint::PUBLIC_URL_MISSING)),
            );
        }

        // No mailer, no reset email: the page could never send its link.
        // The mailer lives in the `.env` alone. Asked of the environment
        // itself, not of the process-wide mailer another app may have set.
        let mailer_ready = MailerConfig::from_env().is_some();
        if self.password_reset.is_some() && !mailer_ready {
            report.add(
                CheckError::new(
                    "Mailer",
                    t(runique_i18n::keys::build::error::MAILER_MISSING),
                )
                .with_suggestion(t(runique_i18n::keys::build::hint::MAILER_MISSING)),
            );
        }

        #[cfg(feature = "acme")]
        if sec.acme_enabled {
            if sec.acme_domain.is_none() {
                report.add(
                    CheckError::new(
                        "ACME",
                        t(runique_i18n::keys::build::error::ACME_DOMAIN_MISSING),
                    )
                    .with_suggestion(t(runique_i18n::keys::build::hint::ACME_DOMAIN_MISSING)),
                );
            }
            if sec.acme_email.is_none() {
                report.add(
                    CheckError::new(
                        "ACME",
                        t(runique_i18n::keys::build::error::ACME_EMAIL_MISSING),
                    )
                    .with_suggestion(t(runique_i18n::keys::build::hint::ACME_EMAIL_MISSING)),
                );
            }
        }

        if report.has_errors() {
            return Err(BuildError::check(report));
        }

        Ok(())
    }

    #[cfg(feature = "admin")]
    fn admin_is_ready(&self) -> bool {
        self.admin.is_ready()
    }

    #[cfg(not(feature = "admin"))]
    fn admin_is_ready(&self) -> bool {
        true
    }

    pub(super) fn all_ready(&self) -> bool {
        self.core.is_ready()
            && self.middleware.is_ready()
            && self.statics.is_ready()
            && self.admin_is_ready()
    }
}
