use crate::filters::Filterable;

use config::Config;
use git_changes::get_commits_by_email;
use git2::Time;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod filters;

fn main() -> Result<(), git2::Error> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    debug!("Initializing application");

    let config = Config::from_file(None).expect("valid config file");


    for repo in config.get_repos() {
        debug!(%repo.author_email, "Retrieving messages by email");
        info!(%repo.path ,"Retrieving data for repo");
        let messages = get_commits_by_email(&repo.path, &repo.author_email, &repo.branch)?;
        let time = Time::new(
            chrono::NaiveDateTime::parse_from_str("2025-06-01 00:00", "%Y-%m-%d %H:%M")
                .expect("parsed timestamp")
                .and_utc()
                .timestamp()
                .into(),
            0,
        );
        let filtered = messages.since(time);

        for commit in filtered {
            println!("{}", commit);
        }
    }

    Ok(())
}
