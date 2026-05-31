//! Shared cookie construction so every credential cookie gets the same hardening.

use axum_extra::extract::cookie::{Cookie, SameSite};

use crate::state::AppState;

/// Builds an HttpOnly, SameSite=Lax, Path=/ cookie (Secure when configured).
pub fn make(state: &AppState, name: &'static str, value: String) -> Cookie<'static> {
    let mut c = Cookie::new(name, value);
    c.set_http_only(true);
    c.set_same_site(SameSite::Lax);
    c.set_path("/");
    c.set_secure(state.config.cookie_secure);
    c
}

/// Builds the matching removal cookie (same name + path) for clearing.
pub fn removal(name: &'static str) -> Cookie<'static> {
    let mut c = Cookie::new(name, "");
    c.set_path("/");
    c
}
