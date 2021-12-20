mod eolina;
pub use eolina::Eolina;
pub use eolina::ExitCode;

mod config;
pub use config::get_prompt;
pub use config::log_level_after_adjust;
pub use config::IS_ERR_TTY;
pub use config::IS_FANCY;
pub use config::IS_IN_TTY;
pub use config::IS_OUT_TTY;
pub use config::LOG_LEVEL_FILTER;

use config::IS_FANCY_CELL;
