use std::fmt::Display;

use git2::{BranchType, Repository, Sort, Time};
use tracing::{debug, error, info, trace, warn};

type GitResult<T> = Result<T, git2::Error>;

/// Store a commit from a change request
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

/// Retrieve all commits made for the provided email.
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

/// Compare all branches with the `base_branch`
pub fn get_unmerged_commits(
    repo_path: &str,
    email: &str,
    base_branch: &str,
    ignored_branches: Option<Vec<String>>,
) -> GitResult<Vec<CommitChange>> {
    let ignored_brances = ignored_branches.unwrap_or(vec![]);
    let repo = Repository::open(repo_path)?;

    // Get the OIDs of the branches
    let branch1_oid = repo.revparse_single(base_branch)?.id();
    let branches = repo.branches(Some(BranchType::Local))?;

    let mut commits: Vec<CommitChange> = vec![];

    for branch in branches {
        if branch.is_err() {
            error!("Error reading branch");
            continue;
        }

        let branch = branch.expect("valid branch");
        let branch_name = branch.0.name()?.expect("branch name");

        // If the branch is ignored
        if ignored_brances.contains(&branch_name.to_string()) {
            debug!(%branch_name, "Ignoring branch");
            continue;
        }

        // If the branch doesn't own by the user, skip
        if !branch_name.contains("richard") {
            continue;
        }

        let branch2_oid = repo.revparse_single(branch_name)?.id();

        // Prepare the revwalk
        let mut revwalk = repo.revwalk()?;
        revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;

        // Only include commits reachable from branch2
        revwalk.push(branch2_oid)?;

        // Exclude commits reachable from branch1
        revwalk.hide(branch1_oid)?;

        // Iterate over the resulting commits
        for oid_result in revwalk {
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            if let Some(author_email) = commit.author().email() {
                if author_email == email {
                    let commit_change = CommitChange {
                        message: commit.message().unwrap_or("<No message>").to_string(),
                        time: commit.time(),
                    };

                    commits.push(commit_change);
                }
            } else {
                warn!(?commit, "author email not found");
            }
        }
    }

    Ok(commits)
}
