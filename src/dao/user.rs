pub const SUPER_ADMIN_AUTHORITY: u32 = 100;
pub const ADMIN_AUTHORITY: u32 = 99;
pub const UPDATE_AUTHORITY: u32 = 98;
pub const SELECT_AUTHORITY: u32 = 97;
#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub user_id: String,
    pub user_password: String,
    pub user_account: String,
    pub user_authority: u32,
    pub timestamp: String,
}
