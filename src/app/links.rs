//! About dialog links and opening them outside the application.

use crate::AppError;
use crate::app::i18n::Text;

/// Links in the About dialog.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Link {
    /// Author's GitHub profile.
    Github,
    /// cyhdev.com.
    Website,
    /// Author's LinkedIn profile.
    Linkedin,
}

impl Link {
    /// Every link in dialog order.
    pub(crate) const ALL: [Self; 3] = [Self::Github, Self::Website, Self::Linkedin];

    /// Target URL, unchanged from the previous interface.
    #[must_use]
    pub(crate) const fn url(self) -> &'static str {
        match self {
            Self::Github => "https://github.com/younghyun1",
            Self::Website => "https://cyhdev.com",
            Self::Linkedin => "https://www.linkedin.com/in/young-hyun-chi-553431376/",
        }
    }

    /// Button label.
    #[must_use]
    pub(crate) const fn title(self) -> Text {
        match self {
            Self::Github => Text::AboutLinkGithub,
            Self::Website => Text::AboutLinkWebsite,
            Self::Linkedin => Text::AboutLinkLinkedin,
        }
    }
}

/// Opens a link: on desktop with the platform opener (`xdg-open` on Linux, `open` on macOS),
/// in the browser with `window.open(url, "_blank")`, which the frame sandbox allows as a popup.
pub(crate) fn open(link: Link) -> Result<(), AppError> {
    Err(AppError::Ui(format!(
        "opening {} is not implemented yet",
        link.url()
    )))
}
