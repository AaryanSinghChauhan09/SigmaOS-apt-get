// SPDX-License-Identifier: Apache-2.0
//! SigmaOS-apt-get — apt-get-style command-line parsing, Debian-inspired.
//!
//! Parses argv into a typed `AptCommand`. Fail-fast: unknown commands,
//! missing package arguments, and invalid flags are rejected with errors
//! rather than silently ignored (defensive programming, least astonishment).

use std::fmt;

/// Errors parsing apt-get arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AptError {
    /// No command given (bare `apt-get`).
    MissingCommand,
    /// Unrecognized command or option.
    UnknownArg(String),
    /// A command that requires package names got none.
    MissingPackages(&'static str),
}

impl fmt::Display for AptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCommand => write!(f, "no command specified"),
            Self::UnknownArg(a) => write!(f, "unknown argument: {a}"),
            Self::MissingPackages(cmd) => write!(f, "{cmd} requires package names"),
        }
    }
}

impl std::error::Error for AptError {}

/// A parsed apt-get invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AptCommand {
    /// `apt-get update` — refresh package lists.
    Update,
    /// `apt-get upgrade` — upgrade installed packages conservatively.
    Upgrade,
    /// `apt-get dist-upgrade` — allow removals to satisfy deps.
    DistUpgrade,
    /// `apt-get install <pkgs...>`
    Install(Vec<String>),
    /// `apt-get remove <pkgs...>`
    Remove(Vec<String>),
    /// `apt-get purge <pkgs...>` — remove including config files.
    Purge(Vec<String>),
    /// `apt-get autoremove` — remove orphaned dependencies.
    Autoremove,
    /// `apt-get clean` — clear downloaded package cache.
    Clean,
}

impl AptCommand {
    /// Parse argv (excluding argv[0]). Leading `-` flags we don't model are
    /// rejected rather than ignored — callers must handle them explicitly.
    pub fn parse(args: &[String]) -> Result<Self, AptError> {
        let (cmd, rest) = args.split_first().ok_or(AptError::MissingCommand)?;
        if cmd.starts_with('-') {
            return Err(AptError::UnknownArg(cmd.clone()));
        }
        let pkgs: Vec<String> = rest
            .iter()
            .filter(|a| !a.starts_with('-'))
            .cloned()
            .collect();
        let takes_pkgs = matches!(cmd.as_str(), "install" | "remove" | "purge");
        if takes_pkgs && pkgs.is_empty() {
            return Err(AptError::MissingPackages(match cmd.as_str() {
                "install" => "install",
                "remove" => "remove",
                _ => "purge",
            }));
        }
        match cmd.as_str() {
            "update" => Ok(Self::Update),
            "upgrade" => Ok(Self::Upgrade),
            "dist-upgrade" => Ok(Self::DistUpgrade),
            "install" => Ok(Self::Install(pkgs)),
            "remove" => Ok(Self::Remove(pkgs)),
            "purge" => Ok(Self::Purge(pkgs)),
            "autoremove" => Ok(Self::Autoremove),
            "clean" => Ok(Self::Clean),
            other => Err(AptError::UnknownArg(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_install() {
        assert_eq!(
            AptCommand::parse(&args(&["install", "bash", "vim"])),
            Ok(AptCommand::Install(vec!["bash".into(), "vim".into()]))
        );
    }

    #[test]
    fn parse_update_and_dist_upgrade() {
        assert_eq!(AptCommand::parse(&args(&["update"])), Ok(AptCommand::Update));
        assert_eq!(
            AptCommand::parse(&args(&["dist-upgrade"])),
            Ok(AptCommand::DistUpgrade)
        );
    }

    #[test]
    fn missing_command_fails() {
        assert_eq!(AptCommand::parse(&[]), Err(AptError::MissingCommand));
    }

    #[test]
    fn install_without_packages_fails() {
        assert_eq!(
            AptCommand::parse(&args(&["install"])),
            Err(AptError::MissingPackages("install"))
        );
    }

    #[test]
    fn unknown_command_fails() {
        assert!(matches!(
            AptCommand::parse(&args(&["frobnicate"])),
            Err(AptError::UnknownArg(_))
        ));
    }

    #[test]
    fn leading_flag_rejected() {
        assert!(matches!(
            AptCommand::parse(&args(&["-y", "install", "x"])),
            Err(AptError::UnknownArg(_))
        ));
    }
}
