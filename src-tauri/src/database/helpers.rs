use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{
    sqlite::{SqliteQueryResult, SqliteRow},
    FromRow, Sqlite,
};

use super::DbResult;

/// Executes the provided SQL query
#[inline]
pub async fn sql_exec<'e, E, Q>(executor: E, query: &Q) -> DbResult<()>
where
    Q: SqlxBinder,
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sql_exec_with_result(executor, query).await?;
    Ok(())
}

/// Executes the provided SQL query
pub async fn sql_exec_with_result<'e, E, Q>(executor: E, query: &Q) -> DbResult<SqliteQueryResult>
where
    Q: SqlxBinder,
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(result)
}

/// Executes the provided SQL query getting the results as a list of a
/// object typed as <O>
pub async fn sql_query_all<'e, E, Q, O>(executor: E, query: &Q) -> DbResult<Vec<O>>
where
    Q: SqlxBinder,
    E: sqlx::Executor<'e, Database = Sqlite>,
    O: for<'r> FromRow<'r, SqliteRow> + Send + Unpin,
{
    let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
    let result = sqlx::query_as_with(&sql, values)
        .fetch_all(executor)
        .await?;

    Ok(result)
}

/// Executes the provided SQL query getting the results as a single
/// object typed as <O> will return an error if no rows were found
pub async fn sql_query_one<'e, E, Q, O>(executor: E, query: &Q) -> DbResult<O>
where
    Q: SqlxBinder,
    E: sqlx::Executor<'e, Database = Sqlite>,
    O: for<'r> FromRow<'r, SqliteRow> + Send + Unpin,
{
    let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
    let result = sqlx::query_as_with(&sql, values)
        .fetch_one(executor)
        .await?;

    Ok(result)
}
/// Executes the provided SQL query getting the results as a single
/// object typed as a tuple starting with (O,) will return an error
/// if no rows were found
///
/// Extracts O where O is the first value within the returned row
/// tuple
pub async fn sql_query_one_single<'e, E, Q, O>(executor: E, query: &Q) -> DbResult<O>
where
    Q: SqlxBinder,
    E: sqlx::Executor<'e, Database = Sqlite>,
    (O,): for<'r> FromRow<'r, SqliteRow> + Send + Unpin,
{
    let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
    let result: (O,) = sqlx::query_as_with(&sql, values)
        .fetch_one(executor)
        .await?;

    Ok(result.0)
}

/// Executes the provided SQL query getting the results as a optional
/// object typed as <O> [None] if no rows are found
pub async fn sql_query_maybe_one<'e, E, Q, O>(executor: E, query: &Q) -> DbResult<Option<O>>
where
    Q: SqlxBinder,
    E: sqlx::Executor<'e, Database = Sqlite>,
    O: for<'r> FromRow<'r, SqliteRow> + Send + Unpin,
{
    let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
    let result = sqlx::query_as_with(&sql, values)
        .fetch_optional(executor)
        .await?;

    Ok(result)
}
