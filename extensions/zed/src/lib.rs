use zed_extension_api::{self, Command, LanguageServerId, Result, Worktree};

struct CairosExtension;

impl zed_extension_api::Extension for CairosExtension {
    fn new() -> Self {
        Self {}
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        Ok(Command {
            args: vec![],
            command: String::from("/Users/samuel/fun/cairos/target/debug/language-server"),
            env: worktree.shell_env(),
        })
    }
}

zed_extension_api::register_extension!(CairosExtension);
