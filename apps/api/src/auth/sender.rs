//! Delivery of one-time codes. The SMS provider is an open decision (PRD), so delivery sits
//! behind [`CodeSender`] and only the local console sender exists for now.

use std::{future::Future, pin::Pin, sync::Mutex};

use crate::auth::phone::Phone;

pub type SendFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

/// Sends a one-time code to a phone number.
pub trait CodeSender: Send + Sync {
    fn send<'a>(&'a self, phone: &'a Phone, code: &'a str) -> SendFuture<'a>;
}

/// Prints codes to standard error, for local development. It is the only place a code is ever
/// written out; api refuses to start with it in release builds (see `main.rs`).
pub struct ConsoleSender;

impl CodeSender for ConsoleSender {
    fn send<'a>(&'a self, phone: &'a Phone, code: &'a str) -> SendFuture<'a> {
        Box::pin(async move {
            eprintln!(
                "[dev code sender] sign-in code for {}: {code}",
                phone.masked()
            );
            Ok(())
        })
    }
}

/// Keeps sent codes in memory, so tests can read them.
#[derive(Default)]
pub struct MemorySender {
    sent: Mutex<Vec<(String, String)>>,
}

impl MemorySender {
    /// The last code sent to `phone` (E.164), if any.
    pub fn last_code(&self, phone: &str) -> Option<String> {
        let sent = self.sent.lock().unwrap_or_else(|e| e.into_inner());
        sent.iter()
            .rev()
            .find(|(p, _)| p == phone)
            .map(|(_, c)| c.clone())
    }

    pub fn count(&self) -> usize {
        self.sent.lock().unwrap_or_else(|e| e.into_inner()).len()
    }
}

impl CodeSender for MemorySender {
    fn send<'a>(&'a self, phone: &'a Phone, code: &'a str) -> SendFuture<'a> {
        Box::pin(async move {
            self.sent
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push((phone.as_str().to_owned(), code.to_owned()));
            Ok(())
        })
    }
}
