pub(crate) mod query;

pub(super) mod scaler {
    use serde::{Deserialize, Serialize};

    #[allow(clippy::upper_case_acronyms)]
    pub type URI = String;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DateTime(String);

    impl From<&chrono::DateTime<chrono::Utc>> for DateTime {
        fn from(value: &chrono::DateTime<chrono::Utc>) -> Self {
            DateTime(value.to_rfc3339())
        }
    }

    impl TryFrom<DateTime> for chrono::DateTime<chrono::Utc> {
        type Error = chrono::ParseError;

        fn try_from(value: DateTime) -> Result<Self, Self::Error> {
            chrono::DateTime::parse_from_rfc3339(value.0.as_str()).map(Into::into)
        }
    }
}
