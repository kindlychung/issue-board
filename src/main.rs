use druid::{
    widget::{CrossAxisAlignment, Flex, Label, List},
    AppLauncher, Data, Lens, LocalizedString, Widget, WidgetExt, WindowDesc,
};
use std::sync::Arc;

#[derive(Clone, Data, Lens)]
struct IssueBoard {
    issues: Arc<Vec<Issue>>,
}

#[derive(Clone, Data, Lens)]
struct Issue {
    title: Arc<str>,
    creator: Arc<str>,
}

impl IssueBoard {
    pub fn new() -> Self {
        IssueBoard {
            issues: Arc::new(vec![
                Issue {
                    title: "Allow lens derive to add consts for custom lenses".into(),
                    creator: "Colin Rofls".into(),
                },
                Issue {
                    title: "Briefly show the scrollbars any time the Scroll widget's size changes".into(),
                    creator: "Kaur Kuut".into(),
                },
                Issue {
                    title: "Define the current scope of druid in the readme".into(),
                    creator: "Finnerale".into(),
                },
            ]),
        }
    }
}

fn main() {
    let title = LocalizedString::new("Issue Board");
    let window = WindowDesc::new(IssueBoard::widget).title(title);

    let state = IssueBoard::new();

    AppLauncher::with_window(window)
        .launch(state)
        .expect("Failed to launch Issue Board");
}

impl IssueBoard {
    pub fn widget() -> impl Widget<IssueBoard> {
        List::new(Issue::widget)
            .lens(IssueBoard::issues)
            .padding(10.0)
            .fix_width(300.0)
            .center()
    }
}

impl Issue {
    pub fn widget() -> impl Widget<Issue> {
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(Label::dynamic(|data: &Issue, _| data.title.to_string()))
            .with_spacer(5.0)
            .with_child(Label::dynamic(|data: &Issue, _| data.creator.to_string()))
            .padding(10.0)
            .border(druid::theme::BORDER_LIGHT, 2.0)
            .rounded(5.0)
            .padding(5.0)
    }
}
