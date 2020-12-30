use lazy_static::lazy_static;
use tui::style::Color;

lazy_static! {
    pub static ref MORE_LOG_GROUP_NAME: String = String::from("More...");
    pub static ref MORE_LOG_GROUP_ARN: String = String::from("more");
    pub static ref MORE_LOG_EVENT_ID: String = String::from("999");
    pub static ref DESELECTED_COLOR: Color = Color::White;
    pub static ref SELECTED_COLOR: Color = Color::Yellow;
    pub static ref MAX_LOG_GROUP_SELECTION: usize = 4;
    pub static ref HELP_INSTRUCTION: String = String::from("'?' to help");
    pub static ref LOADER: String = String::from("⣾⣽⣻⢿⡿⣟⣯⣷");
    pub static ref HELP_MESSAGE: String = String::from(
        r#"
<Navigation>
  [TAB]   - Toggle folding side menu
  [Arrow] - Move focus 

<Side Menu>

<Log Event>
    "#
    );
}
