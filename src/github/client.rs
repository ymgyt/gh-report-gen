use std::{fmt::Debug, time::Duration};

use chrono::{DateTime, Utc};
use graphql_client::{GraphQLQuery, Response};
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Serialize};
use tracing::error;

use crate::github::{gql::query, Pat};

pub struct GithubClient {
    inner: reqwest::Client,
}

impl GithubClient {
    const ENDPOINT: &str = "https://api.github.com/graphql";

    /// Construct GithubClient.
    pub fn new(pat: Option<Pat>) -> anyhow::Result<Self> {
        let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

        let mut headers = HeaderMap::new();

        if let Some(pat) = pat {
            let mut token = HeaderValue::try_from(format!("bearer {}", pat.into_inner()))?;
            token.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, token);
        }

        let inner = reqwest::ClientBuilder::new()
            .user_agent(user_agent)
            .default_headers(headers)
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self { inner })
    }

    /// Authenticated by current pat token, return github user name.
    pub async fn authenticate(&self) -> anyhow::Result<String> {
        let variables = query::authenticate::Variables {};
        let req = query::Authenticate::build_query(variables);
        let res: query::authenticate::ResponseData = self.request(&req).await?;

        Ok(res.viewer.login)
    }

    pub async fn query_issues(
        &self,
        gh_user: impl Into<String>,
        since: &DateTime<Utc>,
        cursor: Option<String>,
    ) -> anyhow::Result<query::issues::ResponseData> {
        let variables = query::issues::Variables {
            user: gh_user.into(),
            since: since.into(),
            after: cursor,
            first: Some(100),
        };
        let req = query::Issues::build_query(variables);
        let res: query::issues::ResponseData = self.request(&req).await?;

        Ok(res)
    }

    pub async fn query_issues_all(
        &self,
        gh_user: impl Into<String>,
        since: &DateTime<Utc>,
    ) -> anyhow::Result<Vec<query::issues::IssuesViewerIssuesNodes>> {
        let gh_user = gh_user.into();
        let since = since;
        let mut cursor = None;
        let mut issues = Vec::new();

        loop {
            let res = self.query_issues(&gh_user, since, cursor).await?;
            if let Some(new_issues) = res.viewer.issues.nodes {
                issues.extend(new_issues.into_iter().flatten());
            }

            if !res.viewer.issues.page_info.has_next_page {
                break;
            }
            cursor = res.viewer.issues.page_info.end_cursor;
        }

        Ok(issues)
    }

    #[tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        ret,
    )]
    async fn request<Body, ResponseData>(&self, body: &Body) -> anyhow::Result<ResponseData>
    where
        Body: Serialize + ?Sized + Debug,
        ResponseData: DeserializeOwned + Debug,
    {
        let res: Response<ResponseData> = self
            .inner
            .post(Self::ENDPOINT)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        match (res.data, res.errors) {
            (_, Some(errs)) if !errs.is_empty() => {
                for err in errs {
                    error!("{err:?}");
                }
                Err(anyhow::anyhow!("failed to request github api"))
            }
            (Some(data), _) => Ok(data),
            _ => Err(anyhow::anyhow!("unexpected response",)),
        }
    }
}
