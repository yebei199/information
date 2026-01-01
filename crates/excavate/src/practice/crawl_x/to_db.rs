//! insert data of crawl_x into database
use sea_orm::entity::prelude::*;

#[derive(
    Clone, Debug, PartialEq, Eq, DeriveEntityModel,
)]
#[sea_orm(
    table_name = "astroturfers_x",
    schema_name = "dev"
)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: String,
    pub name: String,
    pub handle: String,
    pub profile_url: String,
    pub avatar: String,
    pub register_time: String,
    pub changed_name_count: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub async fn save_to_db(db: &DatabaseConnection, models: Vec<Model>) -> anyhow::Result<()> {
    if models.is_empty() {
        return Ok(());
    }
    let active_models: Vec<ActiveModel> = models.into_iter().map(Into::into).collect();

    Entity::insert_many(active_models)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(Column::UserId)
                .update_columns([
                    Column::Name,
                    Column::Handle,
                    Column::ProfileUrl,
                    Column::Avatar,
                    Column::RegisterTime,
                    Column::ChangedNameCount,
                ])
                .to_owned(),
        )
        .exec(db)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, Schema};

    #[tokio::test]
    async fn test_create_table() -> anyhow::Result<()> {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("PG_DB")
            .expect("PG_DB must be set");
        // 建立到数据库的连接
        let db = Database::connect(&database_url).await?;

        // 获取数据库后端类型（PostgreSQL、MySQL等）
        let builder = db.get_database_backend();
        // 创建数据库模式对象
        let schema = Schema::new(builder);
        // 根据实体定义创建表的操作对象
        let mut create_table_op =
            schema.create_table_from_entity(Entity);
        // 设置如果表不存在则创建的选项
        create_table_op.if_not_exists();

        // 执行创建表的操作
        let res = db.execute(&create_table_op).await;
        // 断言操作成功，如果失败则打印错误信息
        assert!(
            res.is_ok(),
            "Failed to create table: {:?}",
            res.err()
        );

        // 验证表是否存在，通过查询表中的记录数来确认
        let count = Entity::find().count(&db).await;
        // 断言查询成功，如果失败则说明表可能不存在
        assert!(
            count.is_ok(),
            "Failed to count rows, table might not exist: {:?}",
            count.err()
        );

        // 返回成功的Result
        Ok(())
    }
}
