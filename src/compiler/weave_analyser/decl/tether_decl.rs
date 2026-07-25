use crate::{
    Parser, Scanner, Token,
    compiler::{
        ast::decl::WovenDecl, compiler::CompileState, scroll_reader::ScrollReader,
        symbol_table::SymbolKind, types::Visibility, weaves::Weave,
    },
    weave_analyser::{WeaveAnalyzer, WeaveError},
};

use std::{
    path::{MAIN_SEPARATOR_STR, PathBuf},
    str::FromStr,
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_tether(
        &mut self,
        token: Token,
        path: Vec<Token>,
        bind_to: Option<Token>,
        is_path: bool,
    ) -> Result<WovenDecl, WeaveError> {
        if self.symbol_table.get_depth() != 0 {
            self.error("Tethering can only be done in the global scope!", token);
            return Ok(WovenDecl::Cursed { span: None });
        }

        // impossible, but just in case
        if path.len() == 0 {
            self.error("Tether path cannot be empty!", token);
            return Ok(WovenDecl::Cursed { span: None });
        }

        let string_content = if is_path {
            // Handle path-based tethering
            let path_str = &format!(
                "{}{}{}",
                PathBuf::from(&self.context.source_path)
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap(), // dont judge me by this line!
                MAIN_SEPARATOR_STR,
                path.first().unwrap().lexeme.as_str()
            );

            // unwrap cus error is infallibe (never gonna give you- I mean happen)
            let path_buf = PathBuf::from_str(path_str).unwrap();

            let reader = ScrollReader::new();

            let scroll_content = reader.read_scroll(&path_buf);

            match scroll_content {
                Ok(content) => content,
                Err(e) => {
                    self.error(
                        &format!("Failed to read scroll '{}': {}", path_buf.display(), e.msg),
                        token,
                    );
                    return Ok(WovenDecl::Cursed { span: None });
                }
            }
        } else {
            if path.len() == 1 {
                self.error(
                            "Tethering directly to a project directory is not how it works. Try changing your tether path to include the scroll you want to import from the project.",
                            token,
                        );
                return Ok(WovenDecl::Cursed { span: None });
            }

            let contents = if path[0].lexeme == "eira" {
                // core library/archive/project, whatever you wanna call it
                let Some(core_scroll) = self.get_core_scroll(&path[1].lexeme) else {
                    self.error(
                        &format!(
                            "The archive or scroll '{}' was not found inside '{}'",
                            path[1].lexeme, path[0].lexeme
                        ),
                        path[0].clone(),
                    );
                    return Ok(WovenDecl::Cursed { span: None });
                };

                core_scroll
            } else if let Some(proj) = self.context.project.as_ref() {
                // case of local tethering (imports)
                if proj.name == path[0].lexeme {
                    let mut file_path: PathBuf = PathBuf::from_str("./").unwrap();
                    for p in path[1..].iter() {
                        file_path.push(p.lexeme.clone());
                        if !file_path.exists() {
                            self.error(
                                &format!(
                                    "The archive/scroll '{}'  does not exist.",
                                    file_path.display()
                                ),
                                token,
                            );
                            return Ok(WovenDecl::Cursed { span: None });
                        }
                    }
                }

                // TODO: Handle external dependencies
                self.error(
                    &format!(
                        "Couldn't find project '{}'. External dependencies are not yet supported!",
                        path[0].lexeme
                    ),
                    token,
                );
                return Ok(WovenDecl::Cursed { span: None });
            } else {
                self.error(
                            &format!(
                                "No project found for tethering with name '{}'. External dependencies are not yet supported!",
                                path[0].lexeme
                            ),
                            path[0].clone(),
                        );
                return Ok(WovenDecl::Cursed { span: None });
            };

            contents.to_string()
        };

        let path = path
            .iter()
            .map(|t| t.lexeme.clone())
            .collect::<Vec<String>>()
            .join(".");

        if let Some(state) = self.context.tethered_scrolls.get(&path) {
            match state {
                CompileState::Compiled => {
                    // already compiled
                    return Ok(WovenDecl::Tether {
                        statements: vec![],
                        bind_to,
                        path,
                    });
                }
                CompileState::Compiling => {
                    self.error(
                                "Circular tethering detected! The scroll you are trying to tether is already being tethered in the current tethering chain.",
                                token,
                            );
                    return Ok(WovenDecl::Cursed { span: None });
                }
                _ => {} // we will attempt to compile it
            }
        }

        // set state to compiling
        self.context
            .tethered_scrolls
            .insert(path.clone(), CompileState::Compiling);

        // set mode to import
        self.context.import_mode = true;

        let tokens = Scanner::init(&string_content).tokenize();
        let ast = match Parser::new(tokens, path.clone()).parse() {
            Ok(a) => a,
            Err(e) => {
                self.error(&format!("Failed to parse tethered content: {}", e.0), token);
                return Ok(WovenDecl::Cursed { span: None });
            }
        };

        let mut analyzer = WeaveAnalyzer::new(self.context, self.augury);

        let w_ast = match analyzer.analyze(ast) {
            Ok(w) => w,
            Err(e) => {
                self.error(
                    &format!("Failed to analyze tethered content: {}", e.msg),
                    token,
                );
                return Ok(WovenDecl::Cursed { span: None });
            }
        };

        let st = analyzer.get_symbol_table();

        // build exports table
        let exports = st.get_exports().clone();

        if let Some(ident) = &bind_to {
            if let Some(_) = self.symbol_table.resolve_in_current_scope(&ident.lexeme) {
                self.error(
                    &format!(
                        "A mark '{}' already exists in the defined realm!",
                        ident.lexeme
                    ),
                    ident.clone(),
                );
                return Ok(WovenDecl::Cursed { span: None });
            }

            self.symbol_table.add_symbol(
                ident.lexeme.clone(),
                Weave::Module(ident.lexeme.clone()),
                SymbolKind::Module(exports),
                None,
                self.symbol_table.get_current_scope_size(),
                Visibility::Secret,
            );
        } else {
            for (name, sym) in exports.iter() {
                if self.symbol_table.resolve_in_current_scope(name).is_some() {
                    self.error(
                            &format!(
                                "Name collision for exported symbol '{}' from tethered module '{}'. Consider renaming the symbol or the module.",
                                name, path
                            ),
                            token,
                        );
                    return Ok(WovenDecl::Cursed { span: None });
                }

                self.symbol_table.import_symbol(
                    name.clone(),
                    sym.weave.clone(),
                    sym.kind.borrow().clone(),
                    None,
                    self.symbol_table.get_current_scope_size(),
                    sym.visibility.clone(), // preserve visibility but do not re-export
                );
            }
        }

        self.context
            .tethered_scrolls
            .insert(path.clone(), CompileState::Compiled);

        // reset import mode after analyzing the tethered scroll
        self.context.import_mode = false;

        Ok(WovenDecl::Tether {
            statements: w_ast,
            path,
            bind_to,
        })
    }
}
