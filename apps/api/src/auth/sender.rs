//! Delivery of one-time codes and invitations by SMS or email. The SMS and email providers are
//! open decisions (PRD), so delivery sits behind [`MessageSender`] and only the local console
//! sender exists for now.

use std::{future::Future, pin::Pin, sync::Mutex};

use crate::auth::{destination::Destination, roles::Role};

pub type SendFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

/// What is sent.
#[derive(Debug, Clone, Copy)]
pub enum Message<'a> {
    SignInCode(&'a str),
    /// An invitation to join as staff (FT-072): sign in with this phone number or email.
    StaffInvitation(Role),
}

impl Message<'_> {
    /// The text a person receives.
    pub fn text(&self) -> String {
        match self {
            Self::SignInCode(code) => {
                format!("Your RaveConnect sign-in code is {code}. Don't share it with anyone.")
            }
            Self::StaffInvitation(role) => format!(
                "You've been invited to RaveConnect as {}. Sign in to the admin app with this number or email to accept.",
                role.label()
            ),
        }
    }
}

/// Sends a message to a phone number (SMS) or an email address.
pub trait MessageSender: Send + Sync {
    fn send<'a>(&'a self, to: &'a Destination, message: Message<'a>) -> SendFuture<'a>;
}

/// Prints messages to standard error, for local development. It is the only place a code is
/// ever written out; api refuses to start with it in release builds (see `main.rs`).
pub struct ConsoleSender;

impl MessageSender for ConsoleSender {
    fn send<'a>(&'a self, to: &'a Destination, message: Message<'a>) -> SendFuture<'a> {
        Box::pin(async move {
            eprintln!(
                "[dev message sender] {} to {}: {}",
                to.channel(),
                to.masked(),
                message.text()
            );
            Ok(())
        })
    }
}

/// A message kept by [`MemorySender`].
#[derive(Debug, Clone)]
pub struct Sent {
    pub to: String,
    pub code: Option<String>,
    pub invitation: Option<Role>,
}

/// Keeps sent messages in memory, so tests can read them.
#[derive(Default)]
pub struct MemorySender {
    sent: Mutex<Vec<Sent>>,
}

impl MemorySender {
    fn sent(&self) -> std::sync::MutexGuard<'_, Vec<Sent>> {
        self.sent.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The last code sent to `to` (E.164 phone number or lower-case email), if any.
    pub fn last_code(&self, to: &str) -> Option<String> {
        self.sent()
            .iter()
            .rev()
            .filter(|s| s.to == to)
            .find_map(|s| s.code.clone())
    }

    /// The last invitation sent to `to`, if any.
    pub fn last_invitation(&self, to: &str) -> Option<Role> {
        self.sent()
            .iter()
            .rev()
            .filter(|s| s.to == to)
            .find_map(|s| s.invitation)
    }

    pub fn count(&self) -> usize {
        self.sent().len()
    }
}

impl MessageSender for MemorySender {
    fn send<'a>(&'a self, to: &'a Destination, message: Message<'a>) -> SendFuture<'a> {
        Box::pin(async move {
            let (code, invitation) = match message {
                Message::SignInCode(code) => (Some(code.to_owned()), None),
                Message::StaffInvitation(role) => (None, Some(role)),
            };
            self.sent().push(Sent {
                to: to.as_str().to_owned(),
                code,
                invitation,
            });
            Ok(())
        })
    }
}
