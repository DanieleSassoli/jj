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

use std::fmt::Debug;

use clap::Subcommand;

use crate::cli_util::CommandHelper;
use crate::command_error::CommandError;
use crate::commands::gerrit;
use crate::ui::Ui;

/// Interact with Gerrit Code Review.
// `clap` requires each variant's payload to implement `Args`, which `Box` does
// not, so the size difference between the variants can't be avoided.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Clone, Debug)]
pub enum GerritCommand {
    Fetch(gerrit::fetch::FetchArgs),
    Upload(gerrit::upload::UploadArgs),
}

pub async fn cmd_gerrit(
    ui: &mut Ui,
    command: &CommandHelper,
    subcommand: &GerritCommand,
) -> Result<(), CommandError> {
    match subcommand {
        GerritCommand::Fetch(fetch) => gerrit::fetch::cmd_gerrit_fetch(ui, command, fetch).await,
        GerritCommand::Upload(review) => {
            gerrit::upload::cmd_gerrit_upload(ui, command, review).await
        }
    }
}

mod fetch;
mod upload;
