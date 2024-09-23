use sqlx::FromRow;

#[derive(FromRow)]
pub struct DBStruct {
    pub id: i32,
    pub country: String,
    pub tags: Vec<String>,
    pub path: String,
    pub size: i32,
}
