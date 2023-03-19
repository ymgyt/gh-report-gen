# gh-report-gen

`gh-report-gen` is a CLI tool written in Rust that fetches Github issues and generate report based on a specified template.  
It helps developers easily crate summaries and overviews of the issue in their repositories.


## Installation

```sh
cargo install gh-report-gen
```

## Prerequisties

### Create Github Personal access tokens(PAT)

`gh-report-gen` uses the Github graphql api.
Therefore, a PAT with repo scope which grants read access to your repositories is required.

[Settings](https://github.com/settings/tokens) > Generate new tokens

**The GraphQL API does not support authentication with fine-grained personal access tokens.** 


## Usage

It is assumed that the environment variable `GH_PAT` is exported.

```sh
export GH_PAT=ghp_your_personal_access_token_with_repo_scope
```

Fetch issues and output in default format

```
gh-report-gen
```

### Filter issues

Fetch issues updated/created after the specified time.  
The time is specified in RFC3339 format.

```sh
gh-report-gen --since "2023-03-01T00:00:00+09:00"
```

Specify repositories from which issues will be fetched.
The repository is specified in `owner/name` format.  
Since glob pattern is supported, you can write `--include myorg/*` to specify all repositories in myorg organization.  
Specify repositories you want to exclude with `--exclude`.  
By default, `--include *` is specified, so all repositories are included.  

You may specify include/exclude as many times as you wish.  
In order for an issue to be eligible for inclusion, it must fall under one of the includes, but not all of the excludes.

For example, to target all repositories of the ymgyt user except the repository corresponding to the "-handson" suffix and  
the foo repository of the myorg organization, execute the following.

```sh
gh-report-gen \
  --include ymgyt/* \
  --exlucde ymgyt/*-handson \
  --include myorg/foo
```

### Customize output

You can specify how to output the retrieved issues.  
The format is based on Rust's template library, [tera](https://github.com/Keats/tera).  
By default, [the following template](https://github.com/ymgyt/gh-report-gen/blob/main/src/report/templates/default.tmpl) is specified.  

For example, to output only the title of the issue

```sh
cat << EOF > ./issues.tmpl
# Issues
user: {{ user }}
{% for issue in issues %}
- {{ issue.title }}
{% endfor %}
EOF

gh-report-gen --template ./issues.tmpl
```

#### Variables available in template

The following variables can be referenced in template.  

| name | description |
| --- | --- |
| `user` | Github username(login) who created the PAT |
| `issues` | Array of `issue` |

#### `issue`

The `issue` has the following fields  
To reference the title of an issue, use something like `issue.title`.  
All times are in RFC3339 format.

| name | description |
| -- | --- |
| `number` | issue ID |
| `title` | title of the issue |
| `closed` | boolean whether the issue is closed or not |
| `closed_at` | time the issue was closed, or None if the issue was not closed |
| `created_at` | time the issue was created |
| `updated_at` | time the issue was updated |
| `state` | current issue state. `OPEN` | `CLOSED` |
| `url` | link to issue |
| `repository_name` | name of the repository to which the issue belongs |
| `repository_owner` | owner of the repository to which the issue belogns |
| `assignees` | assignees for issue excluding the user |
| `labels` | labels of the issue |
| `tracked_issues_count` | number of issues the issue is tracking |
| `tracked_closed_issues_count` | number of closed issues the issue is tracking |

## Development

### Update Github graphql schema

```sh
cargo install graphql_client_cli --force
graphql-client introspect-schema https://api.github.com/graphql \
    --header "Authorization: bearer ${GH_PAT}" \
    --header "user-agent: rust-graphql-client"
```

[Github Docs](https://docs.github.com/en/graphql/guides/introduction-to-graphql#discovering-the-graphql-api)

## License

This project is licensed under the [MIT license.](./LICENSE)

