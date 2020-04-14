use druid::{AppLauncher, WindowDesc, LocalizedString, Data, Lens, Widget};

#[derive(Clone, Data, Lens)]
struct IssueBoard {}

impl IssueBoard {
    pub fn new() -> Self {
        IssueBoard {}
    }
}

fn main() {
    let title = LocalizedString::new("Issue Board");
    let window = WindowDesc::new(build_ui)
        .title(title);

    let state = IssueBoard::new();

    AppLauncher::with_window(window)
        .launch(state)
        .expect("Failed to launch Issue Board");
}

fn build_ui() -> impl Widget<IssueBoard> {
    druid::widget::Label::new("Test")
}
