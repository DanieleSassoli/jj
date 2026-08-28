// Copyright 2025 The Jujutsu Authors
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

use testutils::git;

use crate::common::TestEnvironment;
use crate::common::TestWorkDir;

/// Creates a remote with change 1234 at patchsets 1 and 2, plus the NoteDb
/// `meta` ref that Gerrit exposes alongside them.
fn init_gerrit_remote(test_env: &TestEnvironment, work_dir: &TestWorkDir) {
    let git_repo = git::init(test_env.env_root().join("gerrit"));
    let base =
        git::add_commit(&git_repo, "refs/heads/main", "file", b"base", "base", &[]).commit_id;
    git::add_commit(
        &git_repo,
        "refs/changes/34/1234/1",
        "file",
        b"one",
        "patchset one",
        &[base],
    );
    git::add_commit(
        &git_repo,
        "refs/changes/34/1234/2",
        "file",
        b"two",
        "patchset two",
        &[base],
    );
    git::add_commit(
        &git_repo,
        "refs/changes/34/1234/meta",
        "file",
        b"meta",
        "meta",
        &[base],
    );
    work_dir
        .run_jj(["git", "remote", "add", "gerrit", "../gerrit"])
        .success();
}

#[must_use]
fn log_output(work_dir: &TestWorkDir) -> crate::common::CommandOutput {
    work_dir.run_jj([
        "log",
        "-r",
        "remote_bookmarks()",
        "--no-graph",
        "-T",
        r#"separate(" ", remote_bookmarks, description.first_line()) ++ "\n""#,
    ])
}

#[test]
fn test_gerrit_fetch_defaults_to_latest_patchset() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    init_gerrit_remote(&test_env, &work_dir);

    // The `meta` ref is not a patchset and must not be selected.
    let output = work_dir.run_jj(["gerrit", "fetch", "1234", "--remote=gerrit"]);
    insta::assert_snapshot!(output, @"
    ------- stderr -------
    Fetched change 1234 patchset 2 as 1234@changes
    bookmark: 1234@changes [new] untracked
    [EOF]
    ");

    insta::assert_snapshot!(log_output(&work_dir), @"
    1234@changes patchset two
    [EOF]
    ");
}

#[test]
fn test_gerrit_fetch_explicit_patchset() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    init_gerrit_remote(&test_env, &work_dir);

    work_dir
        .run_jj(["gerrit", "fetch", "1234", "--remote=gerrit", "--patchset=1"])
        .success();
    insta::assert_snapshot!(log_output(&work_dir), @"
    1234@changes patchset one
    [EOF]
    ");
}

#[test]
fn test_gerrit_fetch_unknown_change() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    init_gerrit_remote(&test_env, &work_dir);

    let output = work_dir.run_jj(["gerrit", "fetch", "7777", "--remote=gerrit"]);
    insta::assert_snapshot!(output, @"
    ------- stderr -------
    Error: Change 7777 was not found on remote gerrit
    [EOF]
    [exit status: 1]
    ");

    let output = work_dir.run_jj(["gerrit", "fetch", "1234", "--remote=gerrit", "--patchset=9"]);
    insta::assert_snapshot!(output, @"
    ------- stderr -------
    Error: No such ref on the remote: refs/changes/34/1234/9
    [EOF]
    [exit status: 1]
    ");
}

/// A plain `jj git fetch` prunes `refs/remotes/<remote>/*`, so fetched changes
/// must not live there.
#[test]
fn test_gerrit_fetch_survives_git_fetch() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    init_gerrit_remote(&test_env, &work_dir);

    work_dir
        .run_jj(["gerrit", "fetch", "1234", "--remote=gerrit"])
        .success();
    work_dir
        .run_jj(["git", "fetch", "--remote=gerrit"])
        .success();

    insta::assert_snapshot!(log_output(&work_dir), @"
    1234@changes patchset two
    main@gerrit base
    [EOF]
    ");
}
