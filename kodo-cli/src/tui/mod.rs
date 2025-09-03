pub mod table;
pub mod input;
pub mod dashboard;

// expose a convenient top-level run() so main.rs can call `tui::run(...)`
pub use dashboard::run;
