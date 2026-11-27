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
 WHERE $2::text IS NULL OR data_generations.res_version IS DISTINCT FROM $2
RETURNING generation, res_version, pipeline_seq";

const ONE: &str =
    "SELECT generation, res_version, pipeline_seq FROM data_generations WHERE server_id = $1";

const CREATE: &str =
    "INSERT INTO data_generations (server_id) VALUES ($1) ON CONFLICT (server_id) DO NOTHING";

const LOCK_RECORDED: &str =
    "SELECT recorded_seq FROM data_generations WHERE server_id = $1 FOR UPDATE";

const SET_RECORDED: &str = "UPDATE data_generations SET recorded_seq = $2 WHERE server_id = $1";

type Row = (i64, Option<String>, i64);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Generation {
    pub generation: i64,
    /// The asset pipeline's latest version for the server
    pub res_version: Option<String>,
    /// Bumps that carried a pipeline version
    pub pipeline_seq: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bump {
    Published(Generation),
    /// The row already held this pipeline version, so nothing moved
    AlreadyPublished(Generation),
}

impl Bump {
    #[must_use]
    pub const fn current(&self) -> &Generation {
        match self {
            Self::Published(g) | Self::AlreadyPublished(g) => g,
        }
    }
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
) -> Result<Bump, DbError> {
    let bumped: Option<Row> = sqlx::query_as(BUMP)
        .bind(server.index())
        .bind(res_version)
        .fetch_optional(&mut *conn)
        .await?;
    if let Some(row) = bumped {
        return Ok(Bump::Published(generation(row)));
    }
    let held: Row = sqlx::query_as(ONE)
        .bind(server.index())
        .fetch_one(conn)
        .await?; // a statement of its own, so it sees the commit the upsert waited on
    Ok(Bump::AlreadyPublished(generation(held)))
}

fn generation((generation, res_version, pipeline_seq): Row) -> Generation {
    Generation {
        generation,
        res_version,
        pipeline_seq,
    }
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
