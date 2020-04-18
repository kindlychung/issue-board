use druid::{AppLauncher, LocalizedString, WindowDesc};
use std::sync::Arc;

mod backend;
use backend::*;

mod board;
use board::*;

fn main() {
    let title = LocalizedString::new("Issue Board");
    let window = WindowDesc::new(IssueBoard::widget).title(title);

    let backend = Github::new();
    let (owner, repo) = ("xi-editor", "druid");

    let mut board = IssueBoard::new();

    let mut next_page = GithubPage::default();
    for _ in 0..3 {
        let query = Query {
            repo: Repository {
                owner,
                name: repo,
            },
            page: &next_page,
        };
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