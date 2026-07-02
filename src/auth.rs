use rand::RngExt;

/// One shared session token per container run, created only when
/// DASHBOARD_PASSWORD is set. Restarting the container logs everyone out.
pub struct Auth {
    password: Option<String>,
    session_token: String,
}

impl Auth {
    pub fn new(password: Option<String>) -> Self {
        let bytes: [u8; 32] = rand::rng().random();
        let session_token = bytes.iter().map(|b| format!("{b:02x}")).collect();
        Self {
            password,
            session_token,
        }
    }

    pub fn required(&self) -> bool {
        self.password.is_some()
    }

    /// Validates a login attempt; returns the session token to set as cookie.
    pub fn login(&self, attempt: &str) -> Option<&str> {
        match &self.password {
            Some(p) if p == attempt => Some(&self.session_token),
            _ => None,
        }
    }

    pub fn is_authorized(&self, cookie_header: Option<&str>) -> bool {
        if !self.required() {
            return true;
        }
        let Some(cookies) = cookie_header else {
            return false;
        };
        cookies
            .split(';')
            .filter_map(|c| c.trim().split_once('='))
            .any(|(name, value)| name == "session" && value == self.session_token)
    }

    pub fn cookie(&self) -> String {
        format!(
            "session={}; HttpOnly; SameSite=Strict; Path=/; Max-Age=2592000",
            self.session_token
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_password_means_everything_is_authorized() {
        let auth = Auth::new(None);
        assert!(!auth.required());
        assert!(auth.is_authorized(None));
        assert!(auth.is_authorized(Some("session=whatever")));
    }

    #[test]
    fn login_returns_token_only_for_correct_password() {
        let auth = Auth::new(Some("hunter2".into()));
        assert!(auth.login("wrong").is_none());
        assert!(auth.login("").is_none());
        let token = auth.login("hunter2").unwrap();
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn authorization_requires_matching_session_cookie() {
        let auth = Auth::new(Some("hunter2".into()));
        let token = auth.login("hunter2").unwrap().to_string();

        assert!(!auth.is_authorized(None));
        assert!(!auth.is_authorized(Some("session=forged")));
        assert!(!auth.is_authorized(Some("other=x")));
        assert!(auth.is_authorized(Some(&format!("session={token}"))));
        assert!(auth.is_authorized(Some(&format!("theme=dark; session={token}"))));
    }

    #[test]
    fn tokens_differ_between_instances() {
        let a = Auth::new(Some("x".into()));
        let b = Auth::new(Some("x".into()));
        assert_ne!(a.login("x").unwrap(), b.login("x").unwrap());
    }
}
