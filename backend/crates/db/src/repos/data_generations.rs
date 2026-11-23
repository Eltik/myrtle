use myrtle_types::Server;

use crate::{Conn, DbError};

const ALL: &str = "SELECT server_id, generation, res_version, pipeline_seq FROM data_generations";

const BUMP: &str = "
INSERT INTO data_generations (server_id, generation, res_version, pipeline_seq)
VALUES ($1, 1, $2, ($2::text IS NOT NULL)::int)
ON CONFLICT (server_id) DO UPDATE
   SET generation = data_generations.generation + 1,
       res_version = COALESCE($2, data_generations.res_version),
       pipeline_seq = data_generations.pipeline_seq + ($2::text IS NOT NULL)::int,
       updated_at = now()
RETURNING generation, res_version, pipeline_seq";

const CREATE: &str =
    "INSERT INTO data_generations (server_id) VALUES ($1) ON CONFLICT (server_id) DO NOTHING";

const LOCK_RECORDED: &str =
    "SELECT recorded_seq FROM data_generations WHERE server_id = $1 FOR UPDATE";

const SET_RECORDED: &str = "UPDATE data_generations SET recorded_seq = $2 WHERE server_id = $1";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Generation {
    pub generation: i64,
    /// The asset pipeline's latest version for the server
    pub res_version: Option<String>,
    /// Bumps that carried a pipeline version
    pub pipeline_seq: i64,
}

/// Every server with a recorded generation
pub async fn all(conn: &mut Conn) -> Result<Vec<(Server, Generation)>, DbError> {
    let rows: Vec<(i16, i64, Option<String>, i64)> = sqlx::query_as(ALL).fetch_all(conn).await?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, generation, res_version, pipeline_seq)| {
            Some((
                Server::from_index(id)?,
                Generation {
                    generation,
                    res_version,
                    pipeline_seq,
                },
            ))
        })
        .collect())
}

pub async fn bump(
    conn: &mut Conn,
    server: Server,
    res_version: Option<&str>,
) -> Result<Generation, DbError> {
    let (generation, res_version, pipeline_seq): (i64, Option<String>, i64) = sqlx::query_as(BUMP)
        .bind(server.index())
        .bind(res_version)
        .fetch_one(conn)
        .await?;
    Ok(Generation {
        generation,
        res_version,
        pipeline_seq,
    })
}

pub async fn lock_recorded(conn: &mut Conn, server: Server) -> Result<i64, DbError> {
    sqlx::query(CREATE)
        .bind(server.index())
        .execute(&mut *conn)
        .await?;
    Ok(sqlx::query_scalar(LOCK_RECORDED)
        .bind(server.index())
        .fetch_one(conn)
        .await?)
}

pub async fn set_recorded(conn: &mut Conn, server: Server, seq: i64) -> Result<(), DbError> {
    sqlx::query(SET_RECORDED)
        .bind(server.index())
        .bind(seq)
        .execute(conn)
        .await?;
    Ok(())
}
