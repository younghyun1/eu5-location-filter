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
///
/// # Errors
///
/// Returns `AppError::Ui` when the opener cannot be started or the browser refuses the popup.
pub(crate) fn open(link: Link) -> Result<(), AppError> {
    open_url(link.url())
}

/// Browser build: the host bridge owns `window.open` and refuses non-HTTPS targets.
#[cfg(all(feature = "web", target_family = "wasm"))]
fn open_url(url: &str) -> Result<(), AppError> {
    crate::app::host::open_url(url)
}

/// Desktop build: starts the platform opener with no inherited streams and reaps it on a short
/// lived thread, so the update loop never blocks and the child never lingers as a zombie.
#[cfg(feature = "desktop")]
fn open_url(url: &str) -> Result<(), AppError> {
    use std::process::{Command, Stdio};

    if !url.starts_with("https://") {
        return Err(AppError::Ui(format!("refusing to open a non-HTTPS link: {url}")));
    }
    let program = opener_program();
    let mut child = Command::new(program)
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| AppError::Ui(format!("starting {program} for {url} failed: {error}")))?;
    let reaper = std::thread::Builder::new()
        .name(String::from("link-opener"))
        .spawn(move || {
            // The opener's exit status carries nothing the interface could act on.
            let _status = child.wait();
        });
    match reaper {
        Ok(_handle) => Ok(()),
        Err(error) => Err(AppError::Ui(format!(
            "could not wait for {program}: {error}"
        ))),
    }
}

/// Program that opens a URL with the default browser.
#[cfg(feature = "desktop")]
const fn opener_program() -> &'static str {
    if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    }
}

#[cfg(test)]
mod tests {
    use super::Link;
    use crate::app::i18n::Text;

    #[test]
    fn every_link_is_an_https_url_with_its_own_label() {
        for link in Link::ALL {
            assert!(link.url().starts_with("https://"), "{link:?}");
        }
        let titles: Vec<Text> = Link::ALL.iter().map(|link| link.title()).collect();
        assert_eq!(
            titles,
            [Text::AboutLinkGithub, Text::AboutLinkWebsite, Text::AboutLinkLinkedin]
        );
    }

    #[test]
    fn urls_are_distinct() {
        for (index, link) in Link::ALL.iter().enumerate() {
            for other in &Link::ALL[index + 1..] {
                assert_ne!(link.url(), other.url());
            }
        }
    }

    #[cfg(feature = "desktop")]
    #[test]
    fn desktop_opener_is_a_known_program() {
        assert!(matches!(super::opener_program(), "open" | "xdg-open"));
    }
}
