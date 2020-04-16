use druid::{
    widget::{CrossAxisAlignment, Flex, Label, List, Scroll},
    Data, Lens, UnitPoint, Widget, WidgetExt,
};
use std::sync::Arc;

#[derive(Clone, Data, Lens)]
pub struct IssueBoard {
    pub columns: Arc<Vec<IssueColumn>>,
}

#[derive(Clone, Data, Lens)]
pub struct IssueColumn {
    pub issues: Arc<Vec<Issue>>,
}

#[derive(Clone, Data, Lens)]
pub struct Issue {
    pub title: Arc<str>,
    pub author: Arc<str>,
}

impl IssueBoard {
    pub fn new() -> Self {
        IssueBoard {
            columns: Default::default(),
        }
    }

    pub fn widget() -> impl Widget<IssueBoard> {
        Scroll::new(List::columns(IssueColumn::widget).lens(Self::columns))
            .horizontal()
            .expand()
    }
}

impl IssueColumn {
    pub fn new() -> Self {
        IssueColumn {
            issues: Default::default(),
        }
    }

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