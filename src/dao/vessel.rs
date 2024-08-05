
#[derive(Debug, sqlx::FromRow)]
struct Vessel {
    id: i32,
    name: String,
    email: String,
}
