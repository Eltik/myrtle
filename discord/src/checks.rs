use crate::types::{Context, Error};

pub async fn owner_check(ctx: Context<'_>) -> Result<bool, Error> {
    Ok(ctx.framework().options().owners.contains(&ctx.author().id))
}
