use erg_compiler::varinfo::AbsLocation;
use lsp_types::Command;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

use erg_compiler::artifact::BuildRunnable;
use erg_compiler::hir::Expr;

use lsp_types::{ExecuteCommandParams, Location, Url};

use crate::server::{ELSResult, Server};
use crate::util;

impl<Checker: BuildRunnable> Server<Checker> {
    pub(crate) fn execute_command(&mut self, msg: &Value) -> ELSResult<()> {
        let params = ExecuteCommandParams::deserialize(&msg["params"])?;
        Self::send_log(format!("command requested: {}", params.command))?;
        #[allow(clippy::match_single_binding)]
        match &params.command[..] {
            other => {
                Self::send_log(format!("unknown command: {other}"))?;
                Self::send(
                    &json!({ "jsonrpc": "2.0", "id": msg["id"].as_i64().unwrap(), "result": Value::Null }),
                )
            }
        }
    }

    pub(crate) fn gen_show_trait_impls_command(
        &self,
        trait_loc: AbsLocation,
    ) -> ELSResult<Command> {
        let refs = self.get_refs_from_abs_loc(&trait_loc);
        let uri =
            util::normalize_url(Url::from_file_path(trait_loc.module.as_ref().unwrap()).unwrap());
        let opt_visitor = self.get_visitor(&uri);
        let filter = |loc: Location| {
            let token = self.file_cache.get_token(&loc.uri, loc.range.start)?;
            let min_expr = opt_visitor
                .as_ref()
                .and_then(|visitor| visitor.get_min_expr(&token))?;
            matches!(min_expr, Expr::ClassDef(_)).then_some(loc)
        };
        let impls = refs.into_iter().filter_map(filter).collect::<Vec<_>>();
        let impl_len = impls.len();
        let locations = serde_json::to_value(impls)?;
        let uri = util::normalize_url(Url::from_file_path(trait_loc.module.unwrap()).unwrap());
        let uri = serde_json::to_value(uri)?;
        let position = util::loc_to_pos(trait_loc.loc).unwrap();
        let position = serde_json::to_value(position)?;
        Ok(Command {
            title: format!("{impl_len} implementations"),
            // the command is defined in: https://github.com/erg-lang/vscode-erg/blob/20e6e2154b045ab56fedbc8769d03633acfd12e0/src/extension.ts#L92-L94
            command: "erg.showReferences".to_string(),
            arguments: Some(vec![uri, position, locations]),
        })
    }
}