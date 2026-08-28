// Copyright 2024 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use jj_lib::git;
use jj_lib::git::GitFetch;
use jj_lib::git::GitSettings;
use jj_lib::ref_name::RemoteName;
use jj_lib::repo::Repo as _;

use crate::cli_util::CommandHelper;
use crate::command_error::CommandError;
use crate::command_error::user_error;
use crate::commands::gerrit::upload::calculate_push_remote;
use crate::git_util::GitSubprocessUi;
use crate::git_util::load_git_import_options;
use crate::git_util::print_git_import_stats;
use crate::ui::Ui;

/// Remote namespace that fetched changes are stored under.
///
/// This is deliberately not the Gerrit remote itself: `jj git fetch` prunes
/// `refs/remotes/<remote>/*`, which would delete the fetched changes and
/// abandon their commits.
const CHANGE_REMOTE: &str = "changes";

/// Fetch a change from Gerrit for review or further work
///
/// The change is stored under the `changes` pseudo-remote, so change 1234
/// becomes the remote bookmark `1234@changes`.
///
/// Fetched changes are untracked remote bookmarks, which are immutable by
/// default. To amend a change in place, exclude them from `immutable_heads()`.
#[derive(clap::Args, Clone, Debug)]
pub struct FetchArgs {
    /// The Gerrit change number, as shown in the Gerrit UI
    change: u32,

    /// The patchset to fetch
    ///
    /// Defaults to the latest patchset of the change.
    #[arg(long, short)]
    patchset: Option<u32>,

    /// The Gerrit remote to fetch from
    ///
    /// Can be configured with the `gerrit.default-remote` repository option as
    /// well.
    #[arg(long)]
    remote: Option<String>,
}

pub async fn cmd_gerrit_fetch(
    ui: &mut Ui,
    command: &CommandHelper,
    args: &FetchArgs,
) -> Result<(), CommandError> {
    let mut workspace_command = command.workspace_helper(ui).await?;
    let store = workspace_command.repo().store().clone();
    let remote = calculate_push_remote(&store, command.settings(), args.remote.as_deref())?;
    let remote = RemoteName::new(&remote);

    let git_settings = GitSettings::from_settings(workspace_command.settings())?;
    let remote_settings = workspace_command.settings().remote_settings()?;
    let import_options = load_git_import_options(ui, &git_settings, &remote_settings)?;

    let change = args.change;
    // Gerrit shards change refs by the last two digits of the change number.
    let shard = format!("{:02}", change % 100);

    let mut tx = workspace_command.start_transaction();
    {
        let mut git_fetch = GitFetch::new(
            tx.repo_mut(),
            git_settings.to_subprocess_options(),
            &import_options,
        )?;

        let patchset = match args.patchset {
            Some(patchset) => patchset,
            None => {
                let pattern = format!("refs/changes/{shard}/{change}/*");
                let refs = git_fetch.ls_remote(remote, &[pattern])?;
                latest_patchset(&refs).ok_or_else(|| {
                    user_error(format!(
                        "Change {change} was not found on remote {remote}",
                        remote = remote.as_symbol()
                    ))
                })?
            }
        };

        let source = format!("refs/changes/{shard}/{change}/{patchset}");
        let destination = format!("refs/remotes/{CHANGE_REMOTE}/{change}");
        let mut callback = GitSubprocessUi::new(ui);
        git_fetch.fetch_qualified_refs(remote, &[(source, destination)], &mut callback)?;

        writeln!(
            ui.status(),
            "Fetched change {change} patchset {patchset} as {change}@{CHANGE_REMOTE}"
        )?;
    }

    let stats = git::import_refs(tx.repo_mut(), &import_options).await?;
    print_git_import_stats(ui, &tx, &stats)?;
    tx.finish(ui, format!("fetch gerrit change {change}"))
        .await?;
    Ok(())
}

/// Picks the highest patchset number out of `refs/changes/<shard>/<change>/*`.
///
/// Gerrit also exposes a non-numeric `meta` ref in that namespace, which is the
/// NoteDb record rather than a patchset.
fn latest_patchset(refs: &[String]) -> Option<u32> {
    refs.iter()
        .filter_map(|name| name.rsplit_once('/'))
        .filter_map(|(_, patchset)| patchset.parse::<u32>().ok())
        .max()
}
