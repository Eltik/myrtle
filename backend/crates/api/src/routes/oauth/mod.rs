mod apps;
mod consent;
mod protocol;

use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .merge(apps::routes())
        .merge(consent::routes())
        .merge(protocol::routes())
}
