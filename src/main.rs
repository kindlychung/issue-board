use anyhow::{anyhow, Result};
use druid::{
    widget::{CrossAxisAlignment, Flex, Label, List, Scroll},
    AppLauncher, Data, Lens, LocalizedString, UnitPoint, Widget, WidgetExt, WindowDesc,
};
use std::sync::Arc;
use string_template::Template;

const GITHUB_GRAPHQL_ENDPOINT: &str = "https://api.github.com/graphql";

#[derive(Clone, Data, Lens)]
struct IssueBoard {
    columns: Arc<Vec<IssueColumn>>,
}

#[derive(Clone, Data, Lens)]
struct IssueColumn {
    issues: Arc<Vec<Issue>>,
}

#[derive(Clone, Data, Lens)]
struct Issue {
    title: Arc<str>,
    author: Arc<str>,
}

struct QueryResult {
    pub issues: Vec<Issue>,
    pub next_page: GithubPage,
}

trait Backend {
    fn query(&self, query: Query) -> Result<QueryResult>;
}

struct Github {
    the_query: Template,
}

#[derive(Debug, Clone, Default)]
struct GithubPage {
    end_cursor: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct Query<'a> {
    owner: &'a str,
    repo: &'a str,
    page: &'a GithubPage,
}

impl Github {
    pub fn new() -> Self {
        let the_query = Template::new(include_str!("query.graphql"));
        Github { the_query }
    }
}

impl Backend for Github {
    fn query(&self, query: Query) -> Result<QueryResult> {
        let mut args = std::collections::HashMap::new();
        args.insert("owner", query.owner);
        args.insert("repo", query.repo);
        let page = match &query.page.end_cursor {
            Some(cursor) => format!("\"{}\"", cursor),
            None => "null".to_owned(),
        };
        args.insert("page", &page);
        let query = self.the_query.render(&args);

        let response = ureq::post(GITHUB_GRAPHQL_ENDPOINT)
            .auth_kind("bearer", include_str!("../github_token"))
            .send_json(serde_json::json!({ "query": query }))
            .into_json()?;

        let issues_json: &serde_json::Value = response
            .pointer("/data/repository/issues/nodes")
            .ok_or_else(|| {
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

        let page_info = response
            .pointer("/data/repository/issues/pageInfo")
            .ok_or_else(|| {
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
}

impl IssueBoard {
    pub fn new() -> Self {
        IssueBoard {
            columns: Default::default(),
        }
    }
}

impl IssueColumn {
    pub fn new() -> Self {
        IssueColumn {
            issues: Default::default(),
        }
    }
}

fn main() {
    let title = LocalizedString::new("Issue Board");
    let window = WindowDesc::new(IssueBoard::widget).title(title);

    let backend = Github::new();
    let (owner, repo) = ("xi-editor", "druid");

    let mut board = IssueBoard::new();

    let mut next_page = GithubPage::default();
    for _ in 0..3 {
        let query = Query { owner, repo, page: &next_page };
        let result = backend.query(query).expect("Failed to query Github");
        next_page = result.next_page;
        let mut column = IssueColumn::new();
        Arc::make_mut(&mut column.issues).extend(result.issues.into_iter());
        Arc::make_mut(&mut board.columns).push(column);
    }

    AppLauncher::with_window(window)
        .launch(board)
        .expect("Failed to launch Issue Board");
}

impl IssueBoard {
    pub fn widget() -> impl Widget<IssueBoard> {
        Scroll::new(List::columns(IssueColumn::widget).lens(Self::columns))
            .horizontal()
            .expand()
    }
}

impl IssueColumn {
    pub fn widget() -> impl Widget<IssueColumn> {
        Scroll::new(
            List::rows(Issue::widget)
                .padding(10.0)
                .fix_width(330.0)
                .lens(Self::issues),
        )
        .vertical()
        .align_vertical(UnitPoint::TOP)
        .align_horizontal(UnitPoint::CENTER)
    }
}

impl Issue {
    pub fn widget() -> impl Widget<Issue> {
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(Label::dynamic(|data: &Issue, _| data.title.to_string()))
            .with_spacer(10.0)
            .with_child(Label::dynamic(|data: &Issue, _| {
                format!("- {}", data.author)
            }))
            .padding(10.0)
            .border(druid::theme::BORDER_LIGHT, 2.0)
            .rounded(5.0)
            .padding(5.0)
    }
}
