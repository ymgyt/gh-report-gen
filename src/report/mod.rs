use std::io::Write;

use camino::Utf8Path;
use chrono::{DateTime, Days, Utc};
use serde::Serialize;
use tera::Tera;

use crate::{
    github::{client::GithubClient, gql::query},
    prelude::*,
};

pub struct Reporter<'a> {
    pub client: &'a GithubClient,
    pub since: &'a Option<DateTime<Utc>>,
    pub filter: IssueFilter<'a>,
    pub template: Option<&'a Utf8Path>,
}

impl<'a> Reporter<'a> {
    const TEMPLATE_NAME: &str = "report";

    /// Fetch issues then, render report to writer with given template.
    /// If no template given, the default is used.
    pub async fn report(self, writer: impl Write) -> anyhow::Result<()> {
        let gh_user = {
            let gh_user = self.client.authenticate().await?;
            debug!(?gh_user, "Successfully authenticated by github api");
            gh_user
        };

        let issues: Vec<Issue> = {
            let since = self
                .since
                .or_else(|| Utc::now().checked_sub_days(Days::new(7)))
                .unwrap();

            let mut issues = self
                .client
                .query_issues_all(&gh_user, &since)
                .await?
                .into_iter()
                .map(Issue::try_from)
                .collect::<Result<Vec<_>, _>>()?;

            issues.retain(|issue| self.filter.should_reported(issue));
            issues
        };

        let (tmpl, cx) = {
            let mut tmpl = Tera::default();
            match self.template {
                Some(template) => {
                    tmpl.add_template_file(template.as_std_path(), Some(Self::TEMPLATE_NAME))?
                }
                None => tmpl.add_raw_template(
                    Self::TEMPLATE_NAME,
                    include_str!("templates/default.tmpl"),
                )?,
            }

            let mut cx = tera::Context::new();
            cx.insert("user", &gh_user);
            cx.insert("issues", &issues);

            (tmpl, cx)
        };

        tmpl.render_to(Self::TEMPLATE_NAME, &cx, writer)
            .map_err(anyhow::Error::from)
    }
}

/// Responsible for determining if an issue should be reported.
pub struct IssueFilter<'a> {
    pub include: &'a [glob::Pattern],
    pub exclude: &'a [glob::Pattern],
}

impl<'a> IssueFilter<'a> {
    fn should_reported(&self, issue: &Issue) -> bool {
        let repo = issue.repository_identifier();
        self.include.iter().any(|pattern| pattern.matches(&repo))
            && self.exclude.iter().all(|pattern| !pattern.matches(&repo))
    }
}

// OPEN | CLOSED
type IssueState = query::issues::IssueState;

#[derive(Debug, Serialize)]
struct Issue {
    number: i64,
    title: String,
    closed: bool,
    closed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    state: IssueState,
    url: String,

    repository_name: String,
    repository_owner: String,

    assignees: Vec<String>,
    labels: Vec<String>,

    tracked_issues_count: i64,
    tracked_closed_issues_count: i64,
}

impl Issue {
    fn repository_identifier(&self) -> String {
        format!("{}/{}", self.repository_owner, self.repository_name)
    }
}

impl TryFrom<query::issues::IssuesViewerIssuesNodes> for Issue {
    type Error = anyhow::Error;

    fn try_from(value: query::issues::IssuesViewerIssuesNodes) -> Result<Self, Self::Error> {
        let assignees = value
            .assignees
            .nodes
            .map(|nodes| nodes.into_iter().flatten().map(|node| node.login).collect())
            .unwrap_or_else(Vec::new);
        let labels = value
            .labels
            .and_then(|labels| labels.nodes)
            .map(|nodes| nodes.into_iter().flatten().map(|node| node.name).collect())
            .unwrap_or_else(Vec::new);

        Ok(Issue {
            number: value.number,
            title: value.title,
            closed: value.closed,
            closed_at: value.closed_at.map(TryInto::try_into).transpose()?,
            created_at: value.created_at.try_into()?,
            updated_at: value.updated_at.try_into()?,
            state: value.state,
            url: value.url,

            repository_name: value.repository.name,
            repository_owner: value.repository.owner.login,
            assignees,
            labels,

            tracked_issues_count: value.tracked_issues_count,
            tracked_closed_issues_count: value.tracked_closed_issues_count,
        })
    }
}
