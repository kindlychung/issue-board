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
    issues: Arc<Vec<Issue>>,
}

#[derive(Clone, Data, Lens)]
struct Issue {
    title: Arc<str>,
    author: Arc<str>,
}

#[derive(Debug, Clone, Copy)]
struct Query<'a> {
    owner: &'a str,
    repo: &'a str,
}

impl<'a> Query<'a> {
    pub fn new(owner: &'a str, repo: &'a str) -> Self {
        Query { owner, repo }
    }
}

impl IssueBoard {
    pub fn new() -> Self {
        IssueBoard {
            issues: Default::default(),
        }
    }
}

fn query_github(query: Query) -> Result<Vec<Issue>> {
    let mut args = std::collections::HashMap::new();
    args.insert("owner", query.owner);
    args.insert("repo", query.repo);
    let query = Template::new(include_str!("query.graphql")).render(&args);

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
    Ok(issues)
}

fn main() {
    let title = LocalizedString::new("Issue Board");
    let window = WindowDesc::new(IssueBoard::widget).title(title);

    let (owner, repo) = ("xi-editor", "druid");

    let mut board = IssueBoard::new();

    let query = Query::new(owner, repo);
    let issues = query_github(query).expect("Failed to query Github");
    Arc::make_mut(&mut board.issues).extend(issues.into_iter());

    AppLauncher::with_window(window)
        .launch(board)
        .expect("Failed to launch Issue Board");
}

impl IssueBoard {
    pub fn widget() -> impl Widget<IssueBoard> {
        Scroll::new(
            List::new(Issue::widget)
                .padding(10.0)
                .fix_width(300.0)
                .lens(IssueBoard::issues),
        )
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
