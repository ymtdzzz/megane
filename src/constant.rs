use std::time::Duration;

use lazy_static::lazy_static;
use tui::style::{Color, Style};

lazy_static! {
    pub static ref TAIL_RATE: Duration = Duration::from_secs(1);
    pub static ref DATE_FORMAT: String = String::from("%Y-%m-%d %H:%M:%S");
    pub static ref MORE_LOG_GROUP_NAME: String = String::from("More...");
    pub static ref MORE_LOG_GROUP_ARN: String = String::from("more");
    pub static ref MORE_LOG_EVENT_ID: String = String::from("999");
    pub static ref DESELECTED_COLOR: Color = Color::White;
    pub static ref SELECTED_COLOR: Color = Color::Yellow;
    pub static ref NORMAL_STYLE: Style = Style::default().fg(*DESELECTED_COLOR);
    pub static ref ACTIVE_STYLE: Style = Style::default().fg(*SELECTED_COLOR);
    pub static ref MAX_LOG_GROUP_SELECTION: usize = 4;
    pub static ref HELP_INSTRUCTION: String = String::from("'?' to help");
    pub static ref LOADER: String = String::from("⣾⣽⣻⢿⡿⣟⣯⣷");
    pub static ref HELP_MESSAGE: String = String::from(
        r#"
<Global>
  [TAB]   - Close/open side menu
  [Arrow] - Move focus
  [?]     - Show/hide help
  [C+Ctrl] - Exit

<Side Menu>
  [Up/Down] - Move cursor
  [Enter] - Select log group
  [WORD] - Incremental filtering (add)
  [BackSpace] - Incremental filtering (remove)

<Log Event>
  [Enter] - Copy the selected log event to clipboard
  [J/K] - Move cursor
  [TAB] - Close/open current log event
  [S+Ctrl] - Open search dialog

<Search Dialog>
  [Esc] - Cancel search dialog
  [Enter] - Confirm search dialog and start to search with the new conditions
  [Up/Down] - Move cursor
  [Space] - Select the period
  [TAB] - Toggle period input focus

<Text Box>
  [WORD] - Input text
  [BackSpace] - Delete text
  [Left/Right] - Move cursor
    "#
    );
}
