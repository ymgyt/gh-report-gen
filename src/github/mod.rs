pub(crate) mod client;
pub(crate) mod gql;

use std::{fmt, str::FromStr};

/// Represents personal access token.
#[derive(Clone)]
pub struct Pat(String);

impl Pat {
    fn into_inner(self) -> String {
        self.0
    }
}

impl FromStr for Pat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("ghp_") {
            anyhow::bail!("personal access token should start with ghp_")
        }
        Ok(Pat(s.to_owned()))
    }
}

impl fmt::Debug for Pat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // steal from https://qiita.com/jwhaco/items/924dc8c488c98fd412f6
        let visible = 3;
        let masked = {
            let s = self.0.chars().rev().take(visible).collect::<String>();
            let s = s.chars().rev().collect::<String>();
            format!("{s:*>30}")
        };
        f.debug_tuple("Pat").field(&masked).finish()
    }
}
