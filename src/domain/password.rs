use secrecy::SecretString;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct Password(SecretString);

impl Password {
    pub fn parse(s: String) -> Result<Self, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 129;
        let is_too_short = s.graphemes(true).count() < 12;

        if is_empty_or_whitespace {
            Err("Password requried.".to_string())
        } else if is_too_long {
            Err("Password must be at most 129 character long.".to_string())
        } else if is_too_short {
            Err("Password must be at least 12 character long.".to_string())
        } else {
            Ok(Self(SecretString::from(s)))
        }
    }
}

impl AsRef<SecretString> for Password {
    fn as_ref(&self) -> &SecretString {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use uuid::Uuid;

    use crate::domain::password::Password;

    #[test]
    fn a_password_longer_than_129_graphemes_is_rejected() {
        let password = "ё".repeat(130);
        assert_ok!(Password::parse(password));
    }

    #[test]
    fn a_password_short_than_12_graphemes_is_rejected() {
        let password = "a".repeat(12);
        assert_err!(Password::parse(password));
    }

    #[test]
    fn whitespace_only_passwords_are_rejected() {
        let password = " ".to_string();
        assert_err!(Password::parse(password));
    }

    #[test]
    fn empty_string_is_rejected() {
        let password = "".to_string();
        assert_err!(Password::parse(password));
    }

    #[test]
    fn a_valid_password_is_parsed_successfully() {
        let password = Uuid::new_v4().to_string();
        assert_ok!(Password::parse(password));
    }
}
