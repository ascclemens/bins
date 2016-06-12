extern crate config;

use std::io::prelude::*;
use std::fs::{self, File};
use std::path::PathBuf;
use std::env;
use config::types::Config;
use bins::error::*;

const DEFAULT_CONFIG_FILE: &'static str =
r#"defaults = {
  /*
   * If this is true, all pastes will be created as private or unlisted.
   * Using the command-line option `--public` or `--private` will change this behavior.
   */
  private = true;
  /*
   * If this is true, all pastes will be made to accounts or with API keys defined in this file.
   * Pastebin ignores this setting and the command-line argument, since Pastebin requires an API key to paste.
   * Using the command-line option `--auth` or `--anon` will change this behavior.
   */
  auth = true;
  /*
   * Uncomment this line if you want to set a default service to use with bins. This will make the `--service` option
   * optional and use the configured service if the option is not specified.
   */
  # service = "";
  /*
   * If this is true, all command will copy their output to the system clipboard.
   * Using the command-line option `--copy` or `--no-copy` will change this behavior.
   */
  copy = false;
};

gist = {
  /*
   * The username to use for gist.github.com. This is ignored if access_token is empty.
   */
  username = "";
  /*
   * Access token to use to log in to gist.github.com. If this is empty, an anonymous gist will be made.
   * Generate a token from https://github.com/settings/tokens - only the gist permission is necessary
   */
  access_token = "";
};

pastebin = {
  /*
   * The API key for pastebin.com. Learn more: http://pastebin.com/api
   * If this is empty, all paste attempts to the pastebin service will fail.
   */
  api_key = "";
};
"#;

pub struct BinsConfiguration;

impl BinsConfiguration {
  pub fn new() -> Self {
    BinsConfiguration { }
  }
}

pub trait Configurable {
  fn parse_config(&self) -> Result<Config>;

  fn get_config_paths(&self) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = vec![];
    if let Ok(dir) = env::var("XDG_CONFIG_DIR") {
      let mut xdg = PathBuf::from(dir);
      xdg.push("bins.cfg");
      paths.push(xdg);
    }
    let mut home = match env::home_dir() {
      Some(p) => p,
      None => return paths
    };
    let mut dot_config = home.clone();
    dot_config.push(".config");
    dot_config.push("bins.cfg");
    paths.push(dot_config);
    home.push(".bins.cfg");
    paths.push(home);
    paths
  }

  fn get_config_path(&self) -> Option<PathBuf> {
    self.get_config_paths().into_iter().find(|p| p.exists())
  }
}

impl Configurable for BinsConfiguration {
  fn parse_config(&self) -> Result<Config> {
    let path = match self.get_config_path() {
      Some(p) => p,
      None => {
        let config_paths = self.get_config_paths();
        let priority = some_or_err!(config_paths.first(), "no possible config paths computed".into());
        let parent = some_or_err!(priority.parent(), "config file path had no parent".into());
        let parent_str = some_or_err!(parent.to_str(), "config file path parent could not be converted to string".into());
        try!(fs::create_dir_all(parent_str));
        priority.to_path_buf()
      }
    };
    if !&path.exists() {
      let mut file = try!(File::create(&path));
      try!(file.write_all(DEFAULT_CONFIG_FILE.as_bytes()));
    }
    if (&path).is_dir() || !&path.is_file() {
      return Err("configuration file exists, but is not a valid file".into())
    }
    Ok(try!(config::reader::from_file(path.as_path())))
  }
}
