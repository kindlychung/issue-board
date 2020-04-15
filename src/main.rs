use druid::{
    widget::{CrossAxisAlignment, Flex, Label, List, Scroll},
    AppLauncher, Data, Lens, LocalizedString, Selector, UnitPoint, Widget, WidgetExt, WindowDesc,
};
use std::sync::Arc;

pub const QUERY_COMPLETE: Selector = Selector::new("issue-board.query-complete");

#[derive(Clone, Data, Lens)]
struct IssueBoard {
    issues: Arc<Vec<Issue>>,
}

#[derive(Clone, Data, Lens)]
struct Issue {
    title: Arc<str>,
    author: Arc<str>,
}

impl IssueBoard {
    pub fn new() -> Self {
        IssueBoard {
            issues: Default::default(),
        }
    }
}

fn main() {
    let title = LocalizedString::new("Issue Board");
    let window = WindowDesc::new(IssueBoard::widget).title(title);

    let (owner, repo) = ("xi-editor", "druid");

    let mut args = std::collections::HashMap::new();
    args.insert("owner", owner);
    args.insert("repo", repo);
    let query = string_template::Template::new(include_str!("query.graphql")).render(&args);

    let mut board = IssueBoard::new();

    let reponse = ureq::post("https://api.github.com/graphql")
        .auth_kind("bearer", include_str!("../github_token"))
        .send_json(serde_json::json!({ "query": query }))
        .into_json()
        .expect("Failed to query Github");

    let issues: &serde_json::Value = &reponse["data"]["repository"]["issues"]["nodes"];

    for issue in issues.as_array().unwrap() {
        let author = issue["author"]["name"].as_str().unwrap_or("Nobody").into();
        let title = issue["title"].as_str().unwrap().into();
        Arc::make_mut(&mut board.issues).push(Issue { title, author });
    }

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
            .with_child(Label::dynamic(|data: &Issue, _| format!("- {}", data.author)))
            .padding(10.0)
            .border(druid::theme::BORDER_LIGHT, 2.0)
            .rounded(5.0)
            .padding(5.0)
    }
}
