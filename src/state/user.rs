#[derive(Debug, Clone)]
pub enum RequestedUser {
    User,
    Bot(Bot),
    UserWithToken(User),
}

#[derive(Debug, Clone)]
pub struct User {
    access_token: String,
}

#[derive(Debug, Clone)]
pub struct Bot {
    token: String,
}

impl User {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
}

impl Bot {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}
