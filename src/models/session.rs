use std::error::Error;
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use std::collections::HashSet;
use rand::prelude::ThreadRng;
use crate::token::Token;
use crate::proto::Status;
use super::user;

const TOKEN_LEN: usize = 8;
const EPH_TOKEN_TIMEOUT: Duration = Duration::from_secs(20);

const COOKIE_LEN: usize = 32;
const COOKIE_SEPARATOR: &str = "=";
const ERR_DEADLINE_EXCEEDED: &str = "Deadline exceeded";
const ERR_NO_TID: &str = "The provided cookie has no token ID";
const ERR_SESSION_ALREADY_EXISTS: &str = "A session already exists for client";
const ERR_BROKEN_COOKIE: &str = "No session has been found for cookie";
const ERR_NO_LOGED_EMAIL: &str = "No session has been logged with email";
const ERR_SESSION_BUILD: &str = "Something has failed while building session for";

static mut INSTANCE: Option<Box<dyn Factory>> = None;

pub trait Ctrl {
    fn get_cookie(&self) -> &str;
    fn get_created_at(&self) -> SystemTime;
    fn get_touch_at(&self) -> SystemTime;
    fn get_deadline(&self) -> SystemTime;
    fn get_status(&self) -> Status;
    fn get_email(&self) -> &str;
    fn get_token(&mut self) -> Result<String, Box<dyn Error>>;
}

pub trait Factory {
    fn new_session(&mut self, client: Box<dyn user::Ctrl>) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>>;
    fn get_session_by_cookie(&mut self, cookie: &str) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>>;
    fn get_session_by_email(&mut self, addr: &str) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>>;
    fn destroy_session(&mut self, cookie: &str) -> Result<(), Box<dyn Error>>;
}

pub fn get_instance<'a>() -> &'a mut Box<dyn Factory> {
    let provider: &mut Option<Box<dyn Factory>>;
    unsafe {
        provider = &mut INSTANCE
    }

    match provider {
        Some(ctrl) => {
            ctrl
        },
        None => {
            let timeout = Duration::new(3600, 0);
            let instance = Provider::new(timeout);
            
            unsafe {
                INSTANCE = Some(Box::new(instance));
            }
            
            get_instance()
        },
    }
}

struct Provider {
    timeout: Duration,
    bytoken: HashMap<Token, Box<dyn Ctrl>>,
    byemail: HashMap<String, String>,
    rand_gen: ThreadRng,
}

impl Provider {
    fn new(timeout: Duration) -> impl Factory {
        Provider{
            timeout: timeout,
            bytoken: HashMap::new(),
            byemail: HashMap::new(),
            rand_gen: rand::thread_rng(),
        }
    }

    fn cookie_gen(&mut self) -> Token {
        let deadline = SystemTime::now() + self.timeout;
        Token::new(&mut self.rand_gen, deadline, COOKIE_LEN)
    }

    fn split_cookie(cookie: &str) -> Vec<&str> {
        let split = cookie.split(COOKIE_SEPARATOR);
        split.collect()
    }

    //fn split_email(cookie: &str) -> Result<&str, Box<dyn Error>> {
    //    let split = Provider::split_cookie(cookie);
    //    if split.len() < 2 {
    //        Err(ERR_NO_EMAIL.into())
    //    } else {
    //        Ok(split[1])
    //    }
    //}

    fn split_token(cookie: &str) -> Result<&str, Box<dyn Error>> {
        let split = Provider::split_cookie(cookie);
        if split.len() < 1 {
            Err(ERR_NO_TID.into())
        } else {
            Ok(split[0])
        }
    }

    fn is_alive(&mut self, token: &Token) -> Result<(), Box<dyn Error>> {
        if let Some(pair) = self.bytoken.get_key_value(token) {
            if pair.0.is_alive() {
                self.destroy_session_by_token(token)?;
                let msg = format!("{}", ERR_DEADLINE_EXCEEDED);
                Err(msg.into())
            } else {
                Ok(())
            }

        } else {
            let msg = format!("{}", ERR_BROKEN_COOKIE);
            Err(msg.into())
        }
    }

    fn get_session_by_token(&mut self, token: &Token) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        self.is_alive(token)?;
        if let Some(sess) = self.bytoken.get_mut(token) {
            Ok(sess)
        } else {
            let msg = format!("{} {}", ERR_BROKEN_COOKIE, token);
            Err(msg.into())
        }
    }

    fn destroy_session_by_token(&mut self, token: &Token) -> Result<(), Box<dyn Error>> {
        if let Some(sess) = self.bytoken.remove(&token) {
            let email = sess.get_email();
            self.byemail.remove(email);
        } else {
            let msg = format!("{} {}", ERR_BROKEN_COOKIE, token);
            return Err(msg.into());
        }

        Ok(())
    }
}

impl Factory for Provider {
    fn new_session(&mut self, client: Box<dyn user::Ctrl>) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        let timeout = self.timeout;
        let email = client.get_email().to_string();

        if let None = self.byemail.get(&email) {
            let token = self.cookie_gen();
            //let cookie = format!("{}{}{}", token, COOKIE_SEPARATOR, email);
            let sess = Session::new(client, token.to_string(), timeout);
            
            self.byemail.insert(email.to_string(), token.to_string());
            self.bytoken.insert(token.clone(), Box::new(sess));

            if let Some(sess) = self.bytoken.get_mut(&token) {
                Ok(sess)
            } else {
                let msg = format!("{} {}", ERR_SESSION_BUILD, email);
                Err(msg.into())
            }

        } else {
            // checking if there is already a session for the provided email
            let msg = format!("{} {}", ERR_SESSION_ALREADY_EXISTS, email);
            Err(msg.into())
        }
    }

    fn get_session_by_cookie(&mut self, cookie: &str) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        let tid = Provider::split_token(cookie)?;
        let token = Token::from_string(tid);
        self.get_session_by_token(&token)
    }

    fn get_session_by_email(&mut self, email: &str) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        if let Some(tid) = self.byemail.get(email) {
            let token = Token::from_string(tid);
            self.get_session_by_token(&token)
        } else {
            let msg = format!("{} {}", ERR_NO_LOGED_EMAIL, email);
            Err(msg.into())
        }
    }

    fn destroy_session(&mut self, cookie: &str) -> Result<(), Box<dyn Error>> {
        let tid = Provider::split_token(cookie)?;
        let token = Token::from_string(tid);
        self.destroy_session_by_token(&token)
    }
}

pub struct Session {
    pub cookie: String,
    pub created_at: SystemTime,
    pub touch_at: SystemTime,
    pub timeout: Duration,
    pub status: Status,
    rand_gen: ThreadRng,
    user: Box<dyn user::Ctrl>,
    tokens: HashSet<Token>,
}

impl Session {
    pub fn new(user: Box<dyn user::Ctrl>, cookie: String, timeout: Duration) -> Self {
        Session{
            cookie: cookie,
            created_at: SystemTime::now(),
            touch_at: SystemTime::now(),
            timeout: timeout,
            status: Status::New,
            rand_gen: rand::thread_rng(),
            user: user,
            tokens: HashSet::new(),
        }
    }
}

impl Ctrl for Session {
    fn get_cookie(&self) -> &str {
        &self.cookie
    }

    fn get_email(&self) -> &str {
        self.user.get_email()
    }

    fn get_created_at(&self) -> SystemTime {
        self.created_at
    }

    fn get_touch_at(&self) -> SystemTime {
        self.touch_at
    }

    fn get_deadline(&self) -> SystemTime {
        self.created_at + self.timeout
    }

    fn get_status(&self) -> Status {
        self.status
    }

    fn get_token(&mut self) -> Result<String, Box<dyn Error>> {
        let deadline = SystemTime::now() + EPH_TOKEN_TIMEOUT;
        let token = Token::new(&mut self.rand_gen, deadline, TOKEN_LEN);
        let tid = token.to_string();
        self.tokens.insert(token);
        Ok(tid)
    }
}