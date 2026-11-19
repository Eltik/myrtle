use myrtle_types::Server;

use crate::{Conn, DbError};

const ALL: &str = "SELECT server_id, generation, res_version FROM data_generations";

const BUMP: &str = "
INSERT INTO data_generations (server_id, generation, res_version)
VALUES ($1, 1, $2)
ON CONFLICT (server_id) DO UPDATE
   SET generation = data_generations.generation + 1,
       res_version = COALESCE($2, data_generations.res_version),
       updated_at = now()
RETURNING generation, res_version";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Generation {
    pub generation: i64,
    /// The asset pipeline's latest version for the server
    pub res_version: Option<String>,
}

/// Every server with a recorded generation
pub async fn all(conn: &mut Conn) -> Result<Vec<(Server, Generation)>, DbError> {
    let rows: Vec<(i16, i64, Option<String>)> = sqlx::query_as(ALL).fetch_all(conn).await?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, generation, res_version)| {
            Some((
                Server::from_index(id)?,
                Generation {
                    generation,
                    res_version,
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
    let (generation, res_version): (i64, Option<String>) = sqlx::query_as(BUMP)
        .bind(server.index())
        .bind(res_version)
        .fetch_one(conn)
        .await?;
    Ok(Generation {
        generation,
        res_version,
    })
}
