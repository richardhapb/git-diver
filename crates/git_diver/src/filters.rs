use git2::Time;
use git_changes::CommitChange;
use tracing::debug;


pub trait Filterable {
    type Item;
    fn since(&self, time: Time) -> Vec<&Self::Item>;
}

impl Filterable for Vec<CommitChange> {
    type Item = CommitChange;
    fn since(&self, time: Time) -> Vec<&Self::Item> {
        let mut filtered = vec![];

        debug!("Filtering since {:?}", time);

        for commit in self {
            if commit.time >= time {
                filtered.push(commit);
            }
        }

        return filtered
    }
}

