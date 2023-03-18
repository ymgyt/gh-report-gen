use std::borrow::Cow;

use camino::Utf8PathBuf;
use chrono::{DateTime, Utc};
use clap::{ArgAction, Args, Parser};

use crate::{
    github::{client::GithubClient, Pat},
    prelude::*,
    report::{IssueFilter, Reporter},
};

#[derive(Parser, Debug)]
#[command(version, about = "xxx")]
pub struct App {
    /// Specify logging verbosity, like -vvv
    #[arg(
        long,
        short = 'v',
        action = ArgAction::Count)]
    verbose: u8,

    #[command(flatten)]
    github: GithubOptions,

    #[command(flatten)]
    report: ReportOptions,
}

#[derive(Args, Debug)]
struct GithubOptions {
    /// Personal access token with which authenticate to github api
    #[arg(long, visible_alias = "token", env = "GH_PAT", hide_env_values = true)]
    pat: Pat,

    /// List issues that have been updated at or after the given date
    // https://docs.github.com/en/graphql/reference/input-objects#issuefilters
    #[arg(long)]
    since: Option<DateTime<Utc>>,

    /// Repositories containing issues to be reported. glob pattern supported(like 'myorg/*')
    #[arg(
        long = "include",
        short = 'I',
        default_value = "*",
        value_name = "owner/name"
    )]
    include_repositories: Vec<glob::Pattern>,

    /// Repositories in which issues are excluded. glob pattern supported(like 'myorg/*')
    #[arg(long = "exclude", short = 'E', value_name = "owner/name")]
    exclude_repositories: Vec<glob::Pattern>,
}

#[derive(Args, Debug)]
struct ReportOptions {
    /// Template path for rendering issues
    #[arg(long, short = 't')]
    template: Option<Utf8PathBuf>,
}

pub fn parse() -> App {
    App::parse()
}

fn init_subscriber(verbose: u8) {
    use tracing_subscriber::fmt::time::LocalTime;

    let (target, src, filter) = match verbose {
        0 => (false, false, "info"),
        1 => (true, false, "gh_report_gen=debug"),
        2 => (false, true, "gh_report_gen=trace"),
        _ => (false, true, "gh_report_gen=trace,debug"),
    };

    // Respect log directive given by user if exists.
    let filter = std::env::var("LOG")
        .ok()
        .map(|env| Cow::Owned(env))
        .unwrap_or(Cow::Borrowed(filter));

    tracing_subscriber::fmt()
        .with_timer(LocalTime::rfc_3339())
        .with_target(target)
        .with_file(src)
        .with_line_number(src)
        .with_ansi(true)
        .with_env_filter(filter)
        .init();
}

impl App {
    /// Application entry point.
    /// return exit code.
    pub async fn run(self) -> i32 {
        init_subscriber(self.verbose);

        debug!(args=?self, "Debug...");

        match self.run_inner().await {
            Ok(_) => 0,
            Err(err) => {
                error!("{err:?}");
                2
            }
        }
    }

    async fn run_inner(self) -> anyhow::Result<()> {
        let App {
            github:
                GithubOptions {
                    pat,
                    since,
                    include_repositories,
                    exclude_repositories,
                },
            report: ReportOptions { template },
            ..
        } = self;

        let client = GithubClient::new(Some(pat))?;

        let reporter = Reporter {
            client: &client,
            since: &since,
            filter: IssueFilter {
                include: include_repositories.as_slice(),
                exclude: exclude_repositories.as_slice(),
            },
            template: template.as_deref(),
        };

        reporter.report(std::io::stdout()).await
    }
}
