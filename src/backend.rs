use super::board::IssueLabel;
use crate::Issue;
use anyhow::{anyhow, Result};
use string_template::Template;

const GITHUB_GRAPHQL_ENDPOINT: &str = "https://api.github.com/graphql";

#[derive(Debug, Clone, Copy)]
pub struct Repository<'a> {
    pub owner: &'a str,
    pub name: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct Query<'a> {
    pub repo: Repository<'a>,
    pub page: &'a GithubPage,
}

pub struct QueryResult {
    pub issues: Vec<Issue>,
    pub next_page: GithubPage,
}

pub trait Backend {
    fn query(&self, query: Query) -> Result<QueryResult>;
    fn labels(&self, repo: Repository) -> Result<Vec<IssueLabel>>;
}

pub struct Github {
    search: Template,
    labels: Template,
}

#[derive(Debug, Clone, Default)]
pub struct GithubPage {
    end_cursor: Option<String>,
}

impl Github {
    pub fn new() -> Self {
        let search = Template::new(include_str!("search.graphql"));
        let labels = Template::new(include_str!("labels.graphql"));
        Github { search, labels }
    }
}

impl Backend for Github {
    fn query(&self, query: Query) -> Result<QueryResult> {
        let search = format!(
            "user:{} repo:{} is:issue state:open",
            query.repo.owner, query.repo.name
        );
        let mut args = std::collections::HashMap::new();
        args.insert("search", search.as_str());
        let page = match &query.page.end_cursor {
            Some(cursor) => format!("\"{}\"", cursor),
            None => "null".to_owned(),
        };
        args.insert("page", &page);
        let query = self.search.render(&args);

        let response = ureq::post(GITHUB_GRAPHQL_ENDPOINT)
            .auth_kind("bearer", include_str!("../github_token"))
            .send_json(serde_json::json!({ "query": query }))
            .into_json()?;

        let issues_json: &serde_json::Value =
            response.pointer("/data/search/nodes").ok_or_else(|| {
                anyhow!(
                    "Response did not contain issues:\n{}",
                    serde_json::to_string(&response).unwrap_or("Invalid JSON response".into())
                )
            })?;

        let mut issues = Vec::new();
        for issue in issues_json.as_array().unwrap() {
            let author = issue["author"]["name"]
                .as_str()
                .or_else(|| issue["author"]["login"].as_str())
                .ok_or(anyhow!("An issue had no author"))?
                .into();
            let title = issue["title"]
                .as_str()
                .ok_or(anyhow!("An issue had no title"))?
                .into();
            issues.push(Issue { title, author });
        }

        let page_info = response.pointer("/data/search/pageInfo").ok_or_else(|| {
            anyhow!(
                "Response did not contain page info:\n{}",
                serde_json::to_string(&response).unwrap_or("Invalid JSON response".into())
            )
        })?;

        let next_page = GithubPage {
            end_cursor: Some(
                page_info["endCursor"]
                    .as_str()
                    .ok_or(anyhow!("Page info had no endCursor"))?
                    .into(),
            ),
        };

        Ok(QueryResult { issues, next_page })
    }

    fn labels(&self, repo: Repository) -> Result<Vec<IssueLabel>> {
        let mut args = std::collections::HashMap::new();
        args.insert("repo", repo.name);
        args.insert("owner", repo.owner);
        let query = self.labels.render(&args);

        let response = ureq::post(GITHUB_GRAPHQL_ENDPOINT)
            .auth_kind("bearer", include_str!("../github_token"))
            .send_json(serde_json::json!({ "query": query }))
            .into_json()?;

        let labels_json: &serde_json::Value =
            response.pointer("/data/labels/nodes").ok_or_else(|| {
                anyhow!(
                    "Response did not contain labels:\n{}",
                    serde_json::to_string(&response).unwrap_or("Invalid JSON response".into())
                )
            })?;

        let mut labels = Vec::new();
        for label in labels_json.as_array().unwrap() {
            let name = label["name"]
                .as_str()
                .ok_or(anyhow!("A label had no name"))?
                .into();
            let description = label["description"]
                .as_str()
                .ok_or(anyhow!("A label had no description"))?
                .into();
            let color = label["color"]
                .as_str()
                .ok_or(anyhow!("A label had no color"))?
                .into();
            labels.push(IssueLabel { name, description, color });
        }
        
        Ok(labels)
    }
}
