use crate::ui::theme::Theme;




pub mod splash;
pub mod dashboard;
pub mod theme;
pub mod file_manager;

pub enum Page{
    Dashboard,
    FileManager,
}

pub async fn run() {
    splash::show().await;
    dashboard::show().await;

}