use std::fs::File;

use crate::filters::Filterable;

use config::Config;
use git_changes::{get_commits_by_email, get_unmerged_commits};
use git2::Time;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod filters;

fn main() -> Result<(), git2::Error> {
    let log_file = File::create("/tmp/git-diver.log").expect("permission to create a file");
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_writer(log_file))
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

        println!("\n\nREPOSITORY: {}\n", repo.path);

        for commit in filtered {
            println!("{}", commit);
        }

        let messages = get_unmerged_commits(
            &repo.path,
            &repo.author_email,
            &repo.branch,
            repo.ignored_branches.clone(),
        )?;
        let filtered = messages.since(time);

        if filtered.len() > 0 {
            println!(
                "\n\n=========== Unmerged commits in {} =========== \n\n",
                repo.branch
            );
        }
        for commit in filtered {
            println!("{}", commit);
        }
    }

    Ok(())
}
