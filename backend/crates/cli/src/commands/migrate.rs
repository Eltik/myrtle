use crate::Env;

pub async fn run(env: &Env) -> anyhow::Result<()> {
    myrtle_db::migrate::run(&env.db().await?).await?;
    println!("migrations applied");
    Ok(())
}
