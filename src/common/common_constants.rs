use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
pub const TASK_INFO_KEY_TEMPLATE: &str = "tuna:task:";
pub const TASK_LOCK_KEY_TEMPLATE: &str = "tuna:task_lock:";

pub const TASK_GID_KEY_TEMPLATE: &str = "tuna:task:gtid_set:";
pub const COMMON_TIME_FORMAT: &[BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]");
