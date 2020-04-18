use druid::{
    widget::{Button, CrossAxisAlignment, Either, Flex, Label, List, Scroll},
    Data, Lens, UnitPoint, Widget, WidgetExt,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Data, Lens, Serialize, Deserialize)]
pub struct IssueLabel {
    pub name: Arc<str>,
    pub description: Arc<str>,
    pub color: Arc<str>,
}

#[derive(Clone, Data, Serialize, Deserialize)]
pub struct IssueColumnConfig {
    pub labels: Arc<Vec<String>>,
}

#[derive(Clone, Data, Lens, Serialize, Deserialize)]
pub struct IssueBoard {
    pub repo_owner: Arc<str>,
    pub repo_name: Arc<str>,
    pub labels: Arc<Vec<IssueLabel>>,
    pub columns: Arc<Vec<IssueColumn>>,
}

#[derive(Clone, Data, Lens, Serialize, Deserialize)]
pub struct IssueColumn {
    pub config: IssueColumnConfig,
    #[serde(skip)]
    pub config_mode: bool,
    #[serde(skip)]
    pub issues: Arc<Vec<Issue>>,
}

#[derive(Clone, Data, Lens)]
pub struct Issue {
    pub title: Arc<str>,
    pub author: Arc<str>,
}

impl IssueColumnConfig {
    pub fn new() -> Self {
        IssueColumnConfig {
            labels: Default::default(),
        }
    }
}

impl IssueBoard {
    pub fn new() -> Self {
        IssueBoard {
            repo_owner: "xi-editor".into(),
            repo_name: "druid".into(),
            labels: Default::default(),
            columns: Default::default(),
        }
    }

    pub fn widget() -> impl Widget<IssueBoard> {
        Scroll::new(
            List::columns(|| {
                Either::new(
                    |column: &IssueColumn, _| column.config_mode,
                    IssueColumnConfig::widget(),
                    IssueColumn::widget(),
                )
            })
            .lens(Self::columns),
        )
        .horizontal()
        .expand()
    }
}

impl IssueColumnConfig {
    pub fn widget() -> impl Widget<IssueColumn> {
        Flex::column()
            .with_child(
                Button::new("Finish").on_click(|_, col: &mut IssueColumn, _| {
                    col.config_mode = false;
                }),
            )
            .with_spacer(20.0)
            .padding(10.0)
            .border(druid::theme::BORDER_LIGHT, 2.0)
            .rounded(5.0)
            .padding(5.0)
            .fix_width(330.0)
    }
}

impl IssueColumn {
    pub fn new() -> Self {
        IssueColumn {
            config: IssueColumnConfig::new(),
            config_mode: true,
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
