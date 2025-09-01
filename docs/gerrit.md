# Using Jujutsu with Gerrit Code Review

JJ and Gerrit share the same mental model, which makes Gerrit feel like a
natural collaboration tool for JJ. JJ tracks a "change identity" across
rewrites, and Gerrit’s `Change-Id` tracks the same logical change across patch
sets. JJ and Gerrit's `Change-Id`s aren’t natively compatible yet, but they’re
philosophically aligned. `jj gerrit upload` bridges the gap today by adding a
Gerrit-style `Change-Id` while JJ keeps its own notion of change identity on the
client. In practice, that means small, clean commits that evolve over
time—exactly how Gerrit wants you to work.

This guide assumes a basic understanding of Git, Gerrit, and Jujutsu.

## Set up a Gerrit remote

Jujutsu communicates with Gerrit by pushing commits to a Git remote. If you're
starting from an existing Git repository with Gerrit remotes already configured,
you can use `jj git init --colocate` to start using JJ in that repo. Otherwise,
set up your Gerrit remote.

```shell
# Option 1: Start JJ in an existing Git repo with Gerrit remotes
$ jj git init --colocate

# Option 2: Add a Gerrit remote to a JJ repo
$ jj git remote add gerrit ssh://gerrit.example.com:29418/your/project
```

You can configure default values in your repository config by appending the
below to `.jj/repo/config.toml`, like so:

```toml
[gerrit]
default_remote = "gerrit"  # name of the Git remote to push to
default_for = "main"        # target branch in Gerrit
```

## Basic workflow

`jj gerrit upload` takes one or more revsets, ensures each selected commit has a
Gerrit-compatible `Change-Id:` footer (adding one if missing), and pushes the
resulting heads to `refs/for/<branch>` on your Gerrit remote.

> Note
> Gerrit identifies and updates changes by `Change-Id`. When you reupload a
> commit with the same `Change-Id`, Gerrit creates a new patch set.

### upload a single change

```shell
# upload the last real commit (@-) for review to main
$ jj gerrit upload -r @-
```

## Selecting revisions (revsets)

`jj gerrit upload` accepts one or more `-r/--revisions` arguments. Each argument
may expand to multiple commits. Common patterns:

- `-r @-`: the last non-empty commit
- `-r 'trunk()..@-'`: everything on top of trunk
- `-r 'A..B'`: commits reachable from `B` but not `A`

See the [revsets](./revsets.md) guide for more.

> Warning
> The working-copy commit `@` is empty and is rejected. Use `@-` or another
> concrete commit.

### Preview without pushing

Use `--dry-run` to see which commits would be modified and pushed, and where,
without changing anything or contacting the remote.

```shell
$ jj gerrit upload -r 'trunk()..@-' --for main --dry-run
```

## Target branch and remote selection

You must specify the target branch for review with `--for <branch>` or by
configuring `[gerrit].default_for`.

The remote used to push is determined as follows:

1. `--remote <name>` if provided
2. `[gerrit].default_remote` if configured
3. The sole configured Git remote, if exactly one exists
4. A Git remote named `gerrit`, if present
5. Otherwise, the command errors

## Updating changes after review

To address review feedback, amend or rewrite your commits, then run `jj gerrit
upload` again with the same revsets. Because the `Change-Id` footer is preserved,
Gerrit will add new patch sets to the existing changes instead of creating new
ones.

Examples:

```shell
# Edit and squash into an earlier commit in the stack
$ jj edit xcv  # position on the stack to edit
 --- Apply needed edits ---
$ jj gerrit upload -r 'xcv'
```

## Future: native Jujutsu metadata in Gerrit

The Gerrit project is exploring support for native Jujutsu metadata ("jj
headers") so Gerrit could understand Jujutsu change identity without requiring
`Change-Id` footers. Until then, `jj gerrit upload` will add a Gerrit-compatible
`Change-Id` when needed and push to `refs/for/<branch>`.


