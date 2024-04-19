use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::{
    spawn,
    sync::RwLock,
    time::{sleep, Instant},
};

static DEFAULT_SET: &str = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
static DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);
static INTERVAL: Duration = Duration::from_secs(5);

pub enum TokenError {
    NoMoreTokens,
    TokenNotFound,
}

struct Token {
    created_at: Instant,
}

impl Token {
    fn new() -> Self {
        Self {
            created_at: Instant::now(),
        }
    }
}

pub struct TokenManager {
    engine: Arc<TokenManagerEngine>,
}

impl TokenManager {
    pub fn new(length: u8, timeout: Option<Duration>, chars: Option<&str>) -> Self {
        let engine = Arc::new(TokenManagerEngine::new(length, timeout, chars));
        let manager = Self { engine };
        let engine = Arc::clone(&manager.engine);
        spawn(async move { engine.start().await });
        manager
    }
    pub async fn create(&self) -> Result<String, TokenError> {
        self.engine.create().await
    }
    pub async fn connect(&self, token: &str) -> Result<(), TokenError> {
        self.engine.connect(token).await
    }
    pub async fn list(&self) -> Vec<String> {
        self.engine.list().await
    }
}

impl Drop for TokenManager {
    fn drop(&mut self) {
        let engine = Arc::clone(&self.engine);
        spawn(async move { engine.stop().await });
    }
}

struct TokenManagerEngine {
    length: u8,
    chars: Vec<char>,
    tokens: RwLock<HashMap<String, Token>>,
    timeout: Duration,
    running: AtomicBool,
}

impl TokenManagerEngine {
    fn new(length: u8, timeout: Option<Duration>, chars: Option<&str>) -> Self {
        let chars = chars.unwrap_or(DEFAULT_SET).chars().collect();
        let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT);
        let tokens = RwLock::new(HashMap::new());
        Self {
            length,
            chars,
            tokens,
            timeout,
            running: AtomicBool::new(true),
        }
    }
    async fn start(&self) {
        while self.running.load(Ordering::Relaxed) {
            // println!("Checking tokens");
            let mut expired = Vec::new();
            {
                let tokens = self.tokens.read().await;
                for (token, data) in tokens.iter() {
                    if data.created_at.elapsed() > self.timeout {
                        expired.push(token.clone());
                    }
                }
            }
            // println!("Expired tokens: {:?}", expired);
            if !expired.is_empty() {
                let mut tokens = self.tokens.write().await;
                for token in expired {
                    tokens.remove(&token);
                }
            }
            // println!("Sleeping");
            sleep(INTERVAL).await;
        }
    }

    async fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    async fn create(&self) -> Result<String, TokenError> {
        // println!("Creating token");
        let mut token = String::new();
        let mut unique = false;
        let mut count = 0;
        {
            let tokens = self.tokens.read().await;
            while !unique {
                for _ in 0..self.length {
                    let idx = rand::random::<usize>() % self.chars.len();
                    token.push(self.chars[idx]);
                }
                unique = !tokens.contains_key(&token);
                if count > 2 {
                    return Err(TokenError::NoMoreTokens);
                }
                count += 1;
            }
        }
        // println!("Token created: {}", token);
        {
            let mut tokens = self.tokens.write().await;
            // println!("got lock");
            tokens.insert(token.clone(), Token::new());
        }
        // println!("Token inserted: {}", token);
        Ok(token)
    }
    async fn connect(&self, token: &str) -> Result<(), TokenError> {
        {
            let tokens = self.tokens.read().await;
            if !tokens.contains_key(token) {
                return Err(TokenError::TokenNotFound);
            }
        }
        {
            let mut tokens = self.tokens.write().await;
            tokens.remove(token);
        }
        Ok(())
    }
    async fn list(&self) -> Vec<String> {
        let tokens = self.tokens.read().await;
        tokens.keys().cloned().collect()
    }
}
