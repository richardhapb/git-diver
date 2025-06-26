use std::fmt::Display;

use git2::{Repository, Time};
use tracing::{debug, info, trace, warn};

type GitResult<T> = Result<T, git2::Error>;

pub struct CommitChange {
    pub message: String,
    pub time: Time,
}

impl Display for CommitChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.get_time(), self.message)
    }
}

impl CommitChange {
    fn get_time(&self) -> String {
        let seconds = self.time.seconds();
        match chrono::DateTime::from_timestamp(seconds, 0) {
            Some(time) => time.format("%Y-%m-%d").to_string(),
            None => "".to_string(),
        }
    }
}

pub fn get_commits_by_email(
    repo_path: &str,
    email: &str,
    branch: &str,
) -> GitResult<Vec<CommitChange>> {
    trace!(%repo_path, "Opening repository");

    let repo = Repository::open(repo_path)?;
    let branch_obj = repo.revparse_single(branch)?;
    let oid = branch_obj.id();
    let mut walker = repo.revwalk()?;
    let mut commits: Vec<CommitChange> = vec![];

    info!(%oid, %branch, "Walking");
    walker.push(oid)?;

    for step in walker {
        trace!(?step, "Step");
        let commit = match step {
            Ok(oid) => repo.find_commit(oid),
            Err(e) => Err(e),
        };

        match commit {
            Ok(commit) => {
                if let Some(auth_email) = commit.author().email() {
                    if auth_email != email {
                        trace!(%email, %auth_email, "Email doesn't match, skipping");
                        continue;
                    }
                } else {
                    // If it can't be compared, continue.
                    warn!(?commit, "Cannot retrieve author email");
                    continue;
                }
                if let Some(message) = commit.message() {
                    trace!(%message, "Capturing message");
                    commits.push(CommitChange {
                        message: message.into(),
                        time: commit.time(),
                    })
                }
            }
            Err(e) => warn!(%e, "Error retrieving commit"),
        };
    }

    debug!("Found {} messages", commits.len());
    Ok(commits)
}
