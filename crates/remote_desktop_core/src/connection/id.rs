use rand::Rng;
use std::fmt;

/// 9-digit session ID formatted as XXX-XXX-XXX
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(u32);

impl SessionId {
    pub fn random() -> Self {
        let mut rng = rand::rng();
        Self(rng.random_range(0..1_000_000_000))
    }

    pub fn parse(s: &str) -> Option<Self> {
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() != 9 {
            return None;
        }
        digits.parse::<u32>().ok().map(Self)
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.0;
        write!(f, "{:03}-{:03}-{:03}", id / 1_000_000, (id / 1_000) % 1_000, id % 1_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_format() {
        let id = SessionId(123456789);
        assert_eq!(id.to_string(), "123-456-789");
    }

    #[test]
    fn session_id_parse() {
        let id = SessionId::parse("123-456-789").unwrap();
        assert_eq!(id, SessionId(123456789));
    }

    #[test]
    fn session_id_parse_invalid() {
        assert!(SessionId::parse("12345").is_none());
        assert!(SessionId::parse("abc-def-ghi").is_none());
    }

    #[test]
    fn session_id_random() {
        let id = SessionId::random();
        let s = id.to_string();
        assert_eq!(s.len(), 11); // XXX-XXX-XXX
        assert!(SessionId::parse(&s).is_some());
    }
}
