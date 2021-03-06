use error::Error;
use hexx::FromHex;
use {HexColor, TryFrom};

impl HexColor {
    fn new<S: Into<String>>(s: S) -> HexColor {
        HexColor(s.into())
    }
}

impl Default for HexColor {
    fn default() -> HexColor {
        HexColor::new("#000")
    }
}

impl ::std::fmt::Display for HexColor {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Default slack colors built-in to the API
/// See: https://api.slack.com/docs/attachments
#[derive(Copy, Clone, Debug)]
pub enum SlackColor {
    /// green
    Good,
    /// orange
    Warning,
    /// red
    Danger,
}

// can't seem to convert enum to slice despite trait being implemented
// need this to support passing in the string directly
const SLACK_COLORS: [&'static str; 3] = [// SlackColor::Good.as_slice(),
                                         "good",
                                         "warning",
                                         "danger"];

impl ::std::fmt::Display for SlackColor {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl AsRef<str> for SlackColor {
    fn as_ref(&self) -> &str {
        match *self {
            SlackColor::Good => "good",
            SlackColor::Warning => "warning",
            SlackColor::Danger => "danger",
        }
    }
}

impl From<SlackColor> for HexColor {
    fn from(color: SlackColor) -> HexColor {
        HexColor::new(color.to_string())
    }
}

impl<S> TryFrom<S> for HexColor
    where S: Into<String>
{
    type Err = Error;
    fn try_from(s: S) -> ::std::result::Result<Self, Self::Err> {
        let s: String = s.into();
        if SLACK_COLORS.contains(&&s[..]) {
            return Ok(HexColor(s));
        }

        let num_chars = s.chars().count();
        if num_chars != 7 && num_chars != 4 {
            return Err(Error::HexColor(format!("Must be 4 or 7 characters long (including #): \
                                                found `{}`",
                                               s)));
        }
        if s.chars().next().unwrap() != '#' {
            return Err(Error::HexColor(format!("No leading #: found `{}`", s)));
        }

        // #d18 -> #dd1188
        let hex = if num_chars == 4 {
            s.chars().skip(1).fold(String::from("#"), |mut s, c| {
                s.push(c);
                s.push(c);
                s
            })
        } else {
            s.clone()
        };

        // see if the remaining part of the string is actually hex
        match Vec::from_hex(&hex[1..]) {
            Ok(_) => Ok(HexColor::new(s)),
            Err(e) => Err(e.into()),
        }
    }
}

// even though this will always succeed, it simplifies the trait bound in the builder
impl TryFrom<SlackColor> for HexColor {
    type Err = Error;
    fn try_from(color: SlackColor) -> ::std::result::Result<Self, Self::Err> {
        Ok(color.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use {HexColor, TryFrom};

    #[test]
    fn test_hex_color_too_short() {
        let err = HexColor::try_from("abc").unwrap_err();
        assert_eq!(format!("{}", err),
                   "hex color parsing error: Must be 4 or 7 characters long (including #): found \
                    `abc`"
                       .to_owned());
    }

    #[test]
    fn test_hex_color_missing_hash() {
        let err = HexColor::try_from("1234567").unwrap_err();
        assert_eq!(format!("{}", err),
                   "hex color parsing error: No leading #: found `1234567`".to_owned());
    }

    #[test]
    fn test_hex_color_invalid_hex_fmt() {
        let err = HexColor::try_from("#abc12z").unwrap_err();
        assert_eq!(format!("{}", err),
                   "hexx::FromHexError: Invalid character 'z' at position 5".to_owned());
    }

    #[test]
    fn test_hex_color_good() {
        let h: HexColor = HexColor::try_from(SlackColor::Good).unwrap();
        assert_eq!(format!("{}", h), "good".to_owned());
    }

    #[test]
    fn test_hex_color_danger_str() {
        let ok = HexColor::try_from("danger").unwrap();
        assert_eq!(format!("{}", ok), "danger".to_owned());
    }

    #[test]
    fn test_hex_color_3_char_hex() {
        let ok = HexColor::try_from("#d18").unwrap();
        assert_eq!(format!("{}", ok), "#d18".to_owned());
    }

    #[test]
    fn test_hex_color_valid_upper_hex() {
        let ok = HexColor::try_from("#103D18").unwrap();
        assert_eq!(format!("{}", ok), "#103D18".to_owned());
    }

    #[test]
    fn test_hex_color_valid_lower_hex() {
        let ok = HexColor::try_from("#103d18").unwrap();
        assert_eq!(format!("{}", ok), "#103d18".to_owned());
    }
}
