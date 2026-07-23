use std::{
    cell::RefCell,
    collections::HashMap,
    path::{MAIN_SEPARATOR_STR, PathBuf},
    rc::Rc,
    str::FromStr,
};

use crate::{
    Parser, Scanner,
    Value::{self},
    compiler::{
        Expr, Stmt, WovenExpr, WovenStmt,
        ast::decl::{Decl, WovenDecl},
        compiler::CompileState,
        diagnostics::{Augury, CompilationPhase, SourceLocation},
        mark::{WovenEtchedMark, WovenMark},
        parser::types::ParsedWeave,
        scanner::Token,
        scroll_reader::ScrollReader,
        strand::{
            ADDITIVE_STRAND, CALLABLE_STRAND, CONCATINABLE_STRAND, CONDITIONAL_STRAND,
            DIVISIVE_STRAND, EQUATABLE_STRAND, INDEXIVE_STRAND, ITERABLE_STRAND, MAYBE_STRAND,
            MULTIPLICATIVE_STRAND, NO_STRAND, ORDINAL_STRAND, SUBTRACTIVE_STRAND,
        },
        symbol_table::{Symbol, SymbolKind, SymbolTable},
        token_type::TokenType,
        types::Visibility,
        weave_analyser::WeaveAnalyzerContext,
        weaves::{Weave, Weaver},
    },
    values::{
        native_spell::NativeSpell,
        sign::{SignInfo, SignSchema},
        spell::{SpellInfo, UpValue},
    },
};

#[derive(Debug, Clone)]
pub struct WeaveError {
    pub msg: String,
    pub token: Token,
}

impl WeaveError {
    pub fn new(msg: &str, token: Token) -> Self {
        WeaveError {
            msg: msg.to_owned(),
            token: token,
        }
    }
}

pub type WeaveResult<T> = Result<T, WeaveError>;

#[derive(PartialEq, Clone)]
pub enum Realm {
    Genesis, // script level scope
    Spell,   // spell level scope
}

pub struct WeaveAnalyzer<'a> {
    pub(super) context: &'a mut WeaveAnalyzerContext,
    pub(super) augury: &'a mut Augury,

    pub(super) symbol_table: SymbolTable,
    pub(super) loop_depth: usize,
    pub(super) current_realm: Realm, // track the realm (scope type) the analyzer is in!
    pub(super) spell_stack: Vec<String>, // track the current spell name

    pub(super) current_upvalues: Vec<UpValue>, // upvalue for currently resolving spell
    pub(super) spell_base_depth: usize, // depth where current spell body starts (parameters live here)
    pub(super) spell_slot_counter: usize, // continuous slot counter within current spell
}

impl<'a> WeaveAnalyzer<'a> {
    pub fn new(context: &'a mut WeaveAnalyzerContext, augury: &'a mut Augury) -> Self {
        let st = SymbolTable::new();
        WeaveAnalyzer {
            context,
            augury,
            symbol_table: st,
            loop_depth: 0,
            current_realm: Realm::Genesis,
            spell_stack: vec![],
            current_upvalues: vec![],
            spell_base_depth: 0,
            spell_slot_counter: 0,
        }
    }

    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub(super) fn error(&mut self, msg: &str, token: Token) {
        let source = SourceLocation {
            file: PathBuf::from(self.context.source_path.clone()),
            line: token.line,
            column: token.column,
        };
        self.augury
            .forsee_curse(source, msg.to_string(), CompilationPhase::Weave);
        // Err(WeaveError::new(msg, token))
    }

    pub(super) fn warn(&mut self, msg: &str, token: Token) {
        let source = SourceLocation {
            file: PathBuf::from(self.context.source_path.clone()),
            line: token.line,
            column: token.column,
        };
        self.augury
            .forsee_omen(source, msg.to_string(), CompilationPhase::Weave);
    }

    pub fn analyze(&mut self, ast: Vec<Decl>) -> WeaveResult<Vec<WovenDecl>> {
        self.analyze_decls(ast)
    }

    pub(super) fn analyze_decls(&mut self, decls: Vec<Decl>) -> WeaveResult<Vec<WovenDecl>> {
        let mut w_decls: Vec<WovenDecl> = Vec::new();
        for decl in decls {
            let woven = self.analyze_decl(decl)?;
            w_decls.push(woven);
        }
        Ok(w_decls)
    }

    pub(super) fn analyze_decl(&mut self, decl: Decl) -> WeaveResult<WovenDecl> {
        let is_tether_top_level = self.context.import_mode
            && self.symbol_table.get_depth() == 0
            && self.current_realm == Realm::Genesis;

        let woven = self.analyze_decl_inner(decl)?;

        match woven {
            WovenDecl::Attune { .. }
            | WovenDecl::Sign { .. }
            | WovenDecl::Spell { .. }
            | WovenDecl::VarDeclaration { .. }
            | WovenDecl::Tether { .. } => {}
            WovenDecl::Statement { ref token, stmt: _ } => {
                if is_tether_top_level {
                    return self.error(
                        "Only declarations are allowed at top level in a tethered scroll.",
                        token.clone(),
                    );
                }
                // return self.error("Only declarations are allowed at top level.", token.clone());
            }
            WovenDecl::Cursed { .. } => {}
        }
        Ok(woven)
    }

    pub(super) fn analyze_statements(&mut self, stmts: Vec<Stmt>) -> WeaveResult<Vec<WovenStmt>> {
        let mut w_stmts: Vec<WovenStmt> = Vec::new();
        for stmt in stmts {
            let woven = self.analyze_statement(stmt)?;
            w_stmts.push(woven);
        }

        Ok(w_stmts)
    }

    pub(super) fn analyze_statement(&mut self, stmt: Stmt) -> WeaveResult<WovenStmt> {
        // let is_tether_top_level = self.context.import_mode
        //     && self.symbol_table.get_depth() == 0
        //     && self.current_realm == Realm::Genesis;

        let woven = self.analyze_statement_inner(stmt)?;

        Ok(woven)
    }
    pub(super) fn analyze_decl_inner(&mut self, decl: Decl) -> WeaveResult<WovenDecl> {
        match decl {
            Decl::VarDeclaration {
                name,
                mutable,
                initializer,
                weave,
                visibility,
            } => {
                // allow variable shadowing from outer scopes
                if let Some(_symbol) = self.symbol_table.resolve_in_current_scope(&name.lexeme) {
                    return self.error(
                        &format!(
                            "The variable '{}' already exists in the current scope!",
                            name.lexeme
                        ),
                        name,
                    );
                }

                let visibility = visibility.unwrap_or(Visibility::default());

                let expr_weave: Result<Weave, WeaveError>;
                let mut specified_weave: Option<Weave> = None;

                if weave.is_some() {
                    specified_weave = Some(self.analyze_parsed_weave(weave.clone().unwrap())?);
                }

                let w_initializer = match initializer {
                    Some(val) => {
                        let w = if specified_weave.is_some() {
                            Some(specified_weave.as_ref().unwrap())
                        } else {
                            None
                        };
                        Some(self.analyze_expression(val, w)?)
                    }
                    _ => None,
                };

                let mut parent: Option<Rc<Symbol>> = None;

                match &w_initializer {
                    Some(val) => {
                        // Try to get weave from symbol first (for composite weaves like SpellWeave<TextWeave>)
                        expr_weave = if let Some(symbol) = val.symbol() {
                            parent = Some(Rc::new(symbol.clone()));
                            Ok(val.weave())
                        } else {
                            Ok(val.weave())
                        };
                    }
                    None => {
                        if !mutable {
                            // this shouldnt occur since parser should already have handled this
                            return self.error(
                                "bind values must be initialized with an expression!",
                                name,
                            );
                        }

                        // if no initializer, the weave must be specified. Try to get weave from the specified weave name
                        expr_weave = match specified_weave {
                            Some(ref s_w) => Ok(s_w.clone()),
                            None => {
                                return self.error("Couldn't determine a weave for the variable! You shall specify a weave for uninitialized variables!",
                                name.clone(),)
                            }
                        }
                    }
                }

                let slot = if matches!(self.current_realm, Realm::Spell) {
                    // Inside a spell, use continuous slot counter
                    let current_slot = self.spell_slot_counter;
                    self.spell_slot_counter += 1;
                    current_slot
                } else {
                    // Outside spells, use scope-local slot assignment
                    self.symbol_table.get_current_scope_size()
                };

                // use the explicit weave if defined/available
                let weave_for_symbol = if specified_weave.is_some() {
                    specified_weave.unwrap()
                } else {
                    expr_weave?
                };

                let s = self
                    .symbol_table
                    .define_variable(
                        name.lexeme.clone(),
                        weave_for_symbol,
                        mutable,
                        slot,
                        parent,
                        visibility,
                    )
                    .unwrap();

                Ok(WovenDecl::VarDeclaration {
                    name: name,
                    mutable: mutable,
                    initializer: w_initializer,
                    symbol: s,
                })
            }
            Decl::Spell {
                name,
                reagents,
                body,
                return_weave,
                attuned_to,
                visibility,
            } => self.analyze_spell(name, reagents, visibility, return_weave, attuned_to, body),
            Decl::Sign {
                name,
                marks,
                visibility,
            } => {
                if let Some(_) = self.symbol_table.resolve_in_current_scope(&name.lexeme) {
                    return self.error(
                        "A variable has been declared with same name as the sign.",
                        name,
                    );
                }

                let visibility = visibility.unwrap_or(Visibility::default());

                let mut sign_info = SignInfo {
                    schema: SignSchema::new(name.lexeme.clone()),
                    marks: HashMap::new(),
                    attunements: HashMap::new(),
                };

                let symbol = self.symbol_table.define_sign(
                    name.lexeme.clone(),
                    Weave::Sign(name.lexeme.clone()),
                    sign_info.clone(),
                    None,
                    self.symbol_table.get_current_scope_size(),
                    visibility.clone(),
                );

                let symbol = match symbol {
                    Some(s) => s,
                    _ => {
                        // this shouldnt be thrown
                        return self.error("", name);
                    }
                };

                let mut names: Vec<String> = vec![];
                let mut w_marks: Vec<WovenMark> = vec![];

                for m in marks {
                    if names.contains(&m.name.lexeme) {
                        return self.error(
                            "A different mark with same name exists in the sign!",
                            m.name,
                        );
                    }
                    names.push(m.name.lexeme.clone());
                    let mark_weave = self.analyze_parsed_weave(m.parsed_weave)?;
                    w_marks.push(WovenMark {
                        name: m.name.clone(),
                        weave: mark_weave.clone(),
                    });

                    sign_info.marks.insert(m.name.lexeme.clone(), mark_weave);
                    sign_info.schema.add_field(m.name.lexeme);
                }

                let new_symbol = Symbol {
                    name: symbol.name,
                    weave: symbol.weave,
                    depth: symbol.depth,
                    kind: RefCell::new(SymbolKind::Sign(sign_info)),
                    slot_idx: symbol.slot_idx,
                    parent: None,
                    visibility: visibility,
                };

                self.symbol_table.modify_symbol(new_symbol.clone());

                Ok(WovenDecl::Sign {
                    name,
                    marks: w_marks,
                    sign_symbol: new_symbol,
                    // schema
                })
            }
            Decl::Attune { sign, spells } => Ok(self.analyze_attune(sign, spells)?),

            Decl::Tether {
                token,
                path,
                bind_to,
                is_path,
            } => {
                if self.symbol_table.get_depth() != 0 {
                    return self.error("Tethering can only be done in the global scope!", token);
                }

                // impossible, but just in case
                if path.len() == 0 {
                    return self.error("Tether path cannot be empty!", token);
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
                            return self.error(
                                &format!(
                                    "Failed to read scroll '{}': {}",
                                    path_buf.display(),
                                    e.msg
                                ),
                                token,
                            );
                        }
                    }
                } else {
                    if path.len() == 1 {
                        return self.error(
                            "Tethering directly to a project directory is not how it works. Try changing your tether path to include the scroll you want to import from the project.",
                            token,
                        );
                    }

                    let contents = if path[0].lexeme == "eira" {
                        // core library/archive/project, whatever you wanna call it
                        let Some(core_scroll) = self.get_core_scroll(&path[1].lexeme) else {
                            return self.error(
                                &format!(
                                    "The archive or scroll '{}' was not found inside '{}'",
                                    path[1].lexeme, path[0].lexeme
                                ),
                                path[0].clone(),
                            );
                        };

                        core_scroll
                    } else if let Some(proj) = self.context.project.as_ref() {
                        // case of local tethering (imports)
                        if proj.name == path[0].lexeme {
                            let mut file_path: PathBuf = PathBuf::from_str("./").unwrap();
                            for p in path[1..].iter() {
                                file_path.push(p.lexeme.clone());
                                if !file_path.exists() {
                                    return self.error(
                                        &format!(
                                            "The archive/scroll '{}'  does not exist.",
                                            file_path.display()
                                        ),
                                        token,
                                    );
                                }
                            }
                        }

                        // TODO: Handle external dependencies
                        return self.error(
                            &format!(
                                "Couldn't find project '{}'. External dependencies are not yet supported!",
                                path[0].lexeme
                            ),
                            token,
                        );
                    } else {
                        return self.error(
                            &format!(
                                "No project found for tethering with name '{}'. External dependencies are not yet supported!",
                                path[0].lexeme
                            ),
                            path[0].clone(),
                        );
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
                            return self.error(
                                "Circular tethering detected! The scroll you are trying to tether is already being tethered in the current tethering chain.",
                                token,
                            );
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
                        return self
                            .error(&format!("Failed to parse tethered content: {}", e.0), token);
                    }
                };

                let mut analyzer = WeaveAnalyzer::new(self.context, self.augury);

                let w_ast = match analyzer.analyze(ast) {
                    Ok(w) => w,
                    Err(e) => {
                        return self.error(
                            &format!("Failed to analyze tethered content: {}", e.msg),
                            token,
                        );
                    }
                };

                let st = analyzer.get_symbol_table();

                // build exports table
                let exports = st.get_exports().clone();

                if let Some(ident) = &bind_to {
                    if let Some(_) = self.symbol_table.resolve_in_current_scope(&ident.lexeme) {
                        return self.error(
                            &format!(
                                "A mark '{}' already exists in the defined realm!",
                                ident.lexeme
                            ),
                            ident.clone(),
                        );
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
                            return self.error(
                            &format!(
                                "Name collision for exported symbol '{}' from tethered module '{}'. Consider renaming the symbol or the module.",
                                name, path
                            ),
                            token,
                        );
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

            Decl::Statement { stmt, token } => {
                let w_stmt = self.analyze_statement(*stmt)?;
                Ok(WovenDecl::Statement {
                    stmt: Box::new(w_stmt),
                    token: token,
                })
            }
            Decl::Cursed { span } => todo!(),
        }
    }

    fn analyze_statement_inner(&mut self, stmt: Stmt) -> WeaveResult<WovenStmt> {
        match stmt {
            Stmt::Declaration(decl) => {
                let w_decl = self.analyze_decl(*decl)?;
                Ok(WovenStmt::Declaration(Box::new(w_decl)))
            }
            Stmt::Block { statements } => {
                self.symbol_table.new_scope();
                let w_block = self.analyze_statements(statements)?;
                self.symbol_table.end_scope();
                return Ok(WovenStmt::Block {
                    statements: w_block,
                });
            }
            Stmt::Chant { expression } => {
                let w_expr = self.analyze_expression(expression, None)?;
                Ok(WovenStmt::Chant { expression: w_expr })
            }
            Stmt::ExprStmt { expr } => {
                let w_expr = self.analyze_expression(expr, None)?;
                Ok(WovenStmt::ExprStmt { expr: w_expr })
            }
            Stmt::Fate {
                condition,
                then_branch,
                else_branch,
            } => {
                let w_condition = self.analyze_expression(condition, None)?;

                if !w_condition
                    .weave()
                    .get_tapestry()
                    .has_strand(CONDITIONAL_STRAND)
                {
                    return self.error(
                        "The condition provided to determine the fate does not contain the 'Conditional' strand.",
                        w_condition.token(),
                    );
                }
                // scoping n stuff will be added by the block!
                let w_then = self.analyze_statement(*then_branch)?;

                // self.symbol_table.end_scope();

                let w_else: Option<Box<WovenStmt>> = match else_branch {
                    Some(e_b) => Some(Box::new(self.analyze_statement(*e_b)?)),
                    None => None,
                };
                Ok(WovenStmt::Fate {
                    condition: w_condition,
                    then_branch: Box::new(w_then),
                    else_branch: w_else,
                })
            }

            Stmt::While { condition, body } => {
                let w_condition = self.analyze_expression(condition, None)?;

                if !w_condition
                    .weave()
                    .get_tapestry()
                    .has_strand(CONDITIONAL_STRAND)
                {
                    return self.error(
                        "The condition provided to determine the fate of loop does not contain the 'Conditional' strand.",
                        w_condition.token(),
                    );
                }

                // enter loop scope (for sever, flow purposes)
                self.loop_depth += 1;

                let w_body = self.analyze_statement(*body)?;

                // loop scope exit
                self.loop_depth -= 1;

                Ok(WovenStmt::While {
                    condition: w_condition,
                    body: Box::new(w_body),
                })
            }
            Stmt::Sever { token } => {
                if self.loop_depth == 0 {
                    return self.error("'sever' cannot be used outside a loop circle!", token);
                }
                Ok(WovenStmt::Sever { token })
            }
            Stmt::Flow { token } => {
                if self.loop_depth == 0 {
                    return self.error("'flow' cannot be used outside a loop circle!", token);
                }
                Ok(WovenStmt::Flow { token })
            }
            Stmt::Release { token, expr } => {
                // Ensure 'release' is only used within a spell realm
                if self.current_realm == Realm::Genesis {
                    return self.error(
                        "Values cannot be released from the 'Genesis' realm!\n\
                        Error: Usage of 'release' outside the spell scope.",
                        token,
                    );
                }

                let curr_spell_name = match self.spell_stack.last() {
                    Some(name) => name.clone(),
                    None => {
                        return self.error("Release used outside of any spell scope.", token);
                    }
                };

                // Ensure spell exists and check if already released
                let spell_entry = match self.symbol_table.resolve(&curr_spell_name) {
                    Some(v) => match v.kind.borrow().clone() {
                        SymbolKind::Spell(info) => info,
                        _ => {
                            return self.error(
                                &format!(
                                    "No Spell found in the realm with the name '{}'",
                                    curr_spell_name
                                ),
                                token,
                            );
                        }
                    },
                    None => {
                        return self.error(
                            &format!(
                                "No Spell found in the realm with the name '{}'",
                                curr_spell_name
                            ),
                            token,
                        );
                    }
                };

                let expected_weave = spell_entry.release_weave.clone();

                if let Some(e) = expr {
                    let w_expr = self.analyze_expression(e, Some(&expected_weave))?;

                    // Try to get the weave from the symbol first (for variables with composite weaves)
                    // Otherwise fall back to tapestry lookup
                    let actual_weave = if let Some(symbol) = w_expr.symbol() {
                        symbol.weave.clone()
                    } else {
                        w_expr.weave()
                    };

                    // Exact tapestry check (spells should return the exact weave)
                    match &expected_weave {
                        Weave::Maybe(inner) => {
                            if actual_weave == Weave::Empty || actual_weave == **inner {
                                // valid release
                            } else {
                                return self.error(
                                   &format!(
                                       "The spell '{}' was expected to release '{}' but '{}' was released",
                                       curr_spell_name,
                                       expected_weave.get_name(),
                                       actual_weave.get_name()
                                   ),
                                   token,
                               );
                            }
                        }
                        _ => {
                            if expected_weave != actual_weave {
                                return self.error(
                                    &format!(
                                        "The spell '{}' was expected to release '{}' but '{}' was released",
                                        curr_spell_name,
                                        expected_weave.get_name(),
                                        actual_weave.get_name()
                                    ),
                                    token,
                                );
                            }
                        }
                    }
                    // if expected_weave != w_expr.weave() {
                    //     return self.error(
                    //         &format!(
                    //             "The spell '{}' was expected to release '{}' but '{}' was released",
                    //             curr_spell_name,
                    //             expected_weave.get_name(),
                    //             actual_weave.get_name()
                    //         ),
                    //         token,
                    //     );
                    // }

                    Ok(WovenStmt::Release {
                        token: token,
                        expr: Some(w_expr),
                    })
                } else {
                    // release; with no expression implies Emptiness.
                    // If the spell expects a non-empty weave, this is an error.
                    if expected_weave != Weave::Empty {
                        return self.error(
                            &format!(
                                "The spell '{}' expects a value of weave '{}' to be released, but no value was provided.",
                                curr_spell_name, expected_weave.get_name()
                            ),
                            token,
                        );
                    }

                    Ok(WovenStmt::Release {
                        token: token,
                        expr: None,
                    })
                }
            }

            Stmt::Vanish { target, token } => {
                let w_target = self.analyze_expression(target, None)?;

                match w_target.weave() {
                    Weave::Maybe(_) => {}
                    _ => {
                        return self.error(
                            "The weave of the target expression does not support vanishing (not a Maybe<W> weave).",
                            token,
                        );
                    }
                }

                match w_target.symbol() {
                    Some(symbol) => match *symbol.kind.borrow() {
                        SymbolKind::Variable { mutable } if !mutable => {
                            return self
                                .error("Cannot perform vanish for a bind-ed variable.", token);
                        }
                        _ => {}
                    },
                    None => {
                        return self
                            .error("Cannot perform vanish on a non-variable expression.", token);
                    }
                }

                let empty_literal = WovenExpr::Literal {
                    value: Value::Emptiness,
                    token: token.clone(),
                    weave: Weave::Empty,
                };

                // aka desugared
                let sugar_less = match w_target {
                    WovenExpr::Access {
                        material,
                        property,
                        field_name_idx,
                        weave,
                    } => WovenExpr::FieldSet {
                        material,
                        property,
                        value: Box::new(empty_literal),
                        field_name_idx,
                        weave,
                    },
                    // WovenExpr::Assignment { name, value, weave, symbol } => {},
                    WovenExpr::Variable {
                        name,
                        weave,
                        symbol,
                    } => WovenExpr::Assignment {
                        name,
                        value: Box::new(empty_literal),
                        weave,
                        symbol,
                    },
                    WovenExpr::Extract {
                        deck,
                        index,
                        token,
                        weave,
                    } => WovenExpr::DeckSet {
                        deck,
                        index,
                        value: Box::new(empty_literal),
                        token,
                        weave,
                    },

                    _ => {
                        return self.error("Cannot vanish from provided expression.", token);
                    }
                };

                return Ok(WovenStmt::ExprStmt { expr: sugar_less });
            }
            Stmt::Cursed { span } => todo!(),
        }
    }

    fn analyze_expression(
        &mut self,
        expr: Expr,
        expected_weave: Option<&Weave>,
    ) -> WeaveResult<WovenExpr> {
        match expr {
            Expr::Binary {
                left,
                right,
                operator,
            } => {
                let w_left = self.analyze_expression(*left, None)?;
                let w_right = self.analyze_expression(*right, None)?;

                if operator.token_type == TokenType::Plus {
                    let left_has_additive =
                        w_left.weave().get_tapestry().has_strand(ADDITIVE_STRAND);
                    let left_has_concat = w_left
                        .weave()
                        .get_tapestry()
                        .has_strand(CONCATINABLE_STRAND);
                    let right_has_additive =
                        w_right.weave().get_tapestry().has_strand(ADDITIVE_STRAND);
                    let right_has_concat = w_right
                        .weave()
                        .get_tapestry()
                        .has_strand(CONCATINABLE_STRAND);

                    // Both must support the same type of operation
                    if (left_has_additive && right_has_additive)
                        || (left_has_concat && right_has_concat)
                    {
                        // Valid operation
                    } else {
                        return self.error(
                            "Cannot perform '+' operation: operands must both contain either 'Additive' or 'Concatinable' strand.",
                            operator,
                        );
                    }
                } else {
                    if let Some(req_strand) = self.strand_from_op(operator.token_type) {
                        if !w_left.weave().get_tapestry().has_strand(req_strand) {
                            return self.error(
                                &format!(
                                    "The weave of one of the operands is not composed of {} strand.",
                                    self.strand_string_from_bits(req_strand)
                                ),
                                operator,
                            );
                        }

                        if !w_right.weave().get_tapestry().has_strand(req_strand) {
                            return self.error(
                                &format!(
                                    "The weave of one of the operands is not composed of {} strand.",
                                    self.strand_string_from_bits(req_strand)
                                ),
                                operator,
                            );
                        }
                    } else {
                        return self.error(
                            &format!("Unknown operation '{}'", operator.lexeme),
                            operator,
                        );
                    }
                }

                let result_weave = match operator.token_type {
                    TokenType::Greater
                    | TokenType::Less
                    | TokenType::EqualEqual
                    | TokenType::LessEqual
                    | TokenType::GreaterEqual
                    | TokenType::BangEqual => Weave::Truth,
                    TokenType::Plus => {
                        // hard coded for now. Should be dynamic later
                        if w_left.weave().get_tapestry().has_strand(ADDITIVE_STRAND)
                            && w_right.weave().get_tapestry().has_strand(ADDITIVE_STRAND)
                        {
                            Weave::Num
                        } else {
                            Weave::Text
                        }
                    }
                    _ => w_left.weave(), // Assumes left-hand side's type
                };

                Ok(WovenExpr::Binary {
                    left: Box::new(w_left),
                    right: Box::new(w_right),
                    operator: operator,
                    weave: result_weave,
                })
            }
            Expr::Grouping { expression } => self.analyze_expression(*expression, None),
            Expr::Literal { value, token } => {
                let weave = match value {
                    Value::Number(_) => Weave::Num,
                    Value::Emptiness => Weave::Empty,
                    Value::Bool(_) => Weave::Truth,
                    Value::String(_) => Weave::Text,
                    _ => {
                        return self.error("Couldnt find a weave for the value", token.clone());
                    }
                };
                return Ok(WovenExpr::Literal {
                    value: value,
                    token: token,
                    weave,
                });
            }
            Expr::Unary { operand, operator } => {
                if operator.token_type != TokenType::Minus && operator.token_type != TokenType::Bang
                {
                    return self.error("Unknown Unary Operation", operator);
                }
                if let Some(strand) = self.strand_from_op(operator.token_type) {
                    let expr = self.analyze_expression(*operand, None)?;
                    if !expr.weave().get_tapestry().has_strand(strand) {
                        return self.error(
                            &format!(
                                "The operand does not contain the '{}' strand as required by '{}' operation",
                                self.strand_string_from_bits(strand),
                                operator.lexeme
                            ),
                            operator,
                        );
                    }
                    let weave = expr.weave();
                    Ok(WovenExpr::Unary {
                        operand: Box::new(expr),
                        operator: operator,
                        weave: weave,
                    })
                } else {
                    return self.error("Unknown Operation", operator);
                }
            }
            Expr::Variable { name } => {
                if let Some(symbol) = self.symbol_table.resolve(&name.lexeme).cloned() {
                    //The symbol(variable) has been found
                    self.resolve_n_add_upvalue(&symbol)?;

                    let weave = &symbol.weave;
                    let woven = WovenExpr::Variable {
                        name: name,
                        weave: weave.clone(),
                        symbol: symbol,
                    };

                    Ok(woven)
                } else {
                    return self.error(
                        &format!("'{}' was undefined in the eira-verse!", name.lexeme),
                        name,
                    );
                }
            }
            Expr::Assignment { name, value } => {
                if let Some(resolved) = self.symbol_table.resolve(&name.lexeme).cloned() {
                    match *resolved.kind.borrow() {
                        SymbolKind::Variable { mutable } => {
                            if !mutable {
                                return self.error(
                            "Tried to reassign a value to a 'bind'. Binds cannot be reassigned!",
                            name,
                        );
                            }
                        }
                        _ => return self.error("The value isnt a variable!", name),
                    };

                    let woven_expr = self.analyze_expression(*value, None)?;
                    let weave = woven_expr.weave();

                    // Assignment requires an exact match of the tapestry!
                    if resolved.weave == woven_expr.weave() {
                        return Ok(WovenExpr::Assignment {
                            name: name,
                            value: Box::new(woven_expr),
                            weave: weave,
                            symbol: resolved,
                        });
                    }

                    return self.error(
                        "The assignee and the value to be assigned are of different Weaves!\nAssignment failed.",
                        name,
                    );
                } else {
                    return self.error(
                        "The mark was no where to be found from this realm!\nVariable resolution failed.",
                        name,
                    );
                }
            }
            Expr::Cast {
                reagents,
                callee,
                token,
            } => {
                let w_callee = self.analyze_expression(*callee, None);

                if let Ok(WovenExpr::BoundSpell {
                    is_safe,
                    material,
                    spell_symbol,
                    token,
                    weave: _,
                }) = &w_callee
                {
                    let method_symbol = spell_symbol;
                    self.resolve_n_add_upvalue(&method_symbol)?;
                    let spell_info = method_symbol.kind.borrow().get_spell_info().unwrap();

                    if let Some(expected) = expected_weave {
                        if *expected != spell_info.release_weave {
                            return self.error(
                                    &format!(
                                        "The release weave of spell '{}' does not match the expected weave '{}'",
                                        method_symbol.name,
                                        expected.get_name()
                                    ),
                                    token.clone(),
                                );
                        }
                    }

                    let mut final_reagents = vec![*material.clone()];

                    for (i, r) in reagents.iter().enumerate() {
                        let w_r = self.analyze_expression(
                            r.clone(),
                            Some(&spell_info.reagents.get(i).unwrap().weave),
                        )?;
                        final_reagents.push(w_r);
                    }

                    if final_reagents.len() != spell_info.reagents.len() {
                        return self.error(
                                &format!(
                                    "The spell '{}' expected {} reagent(s), but you provided {} of them!",
                                    method_symbol.name,
                                    spell_info.reagents.len().saturating_sub(1), // one is ego
                                    final_reagents.len().saturating_sub(1)
                                ),
                                token.clone(),
                            );
                    }

                    if *is_safe {
                        return Ok(WovenExpr::SafeCast {
                            callee: token.clone(),
                            reagents: final_reagents,
                            spell_symbol: method_symbol.clone(),
                            weave: spell_info.release_weave.clone(),
                        });
                    } else {
                        return Ok(WovenExpr::Cast {
                            callee: token.clone(),
                            reagents: final_reagents,
                            spell_symbol: method_symbol.clone(),
                            weave: spell_info.release_weave.clone(),
                        });
                    }
                    // } else {
                    // return self.error(
                    // "for now... just be satisfied with spell casting only on signs!",
                    // w_material.token(),
                    // );
                    // }
                }

                let native = match &w_callee {
                    Err(_) => {
                        let nat = NativeSpell::resolve(&token.lexeme);

                        if nat.is_ok() {
                            Some(nat.unwrap())
                        } else {
                            None
                        }
                    }
                    Ok(_) => None,
                };

                if native.is_some() {
                    let native_spell = native.unwrap();

                    let native_info = NativeSpell::get_spell_info(native_spell.clone()).unwrap();

                    if native_info.reagents.len() != reagents.len() {
                        return self.error(
                            &format!(
                                "The spell '{}' expected {} reagents, but you provided {} of them!",
                                native_info.name,
                                native_info.reagents.len(),
                                reagents.len()
                            ),
                            token,
                        );
                    }

                    if let Some(expected) = expected_weave {
                        if *expected != native_info.release_weave {
                            return self.error(
                                    &format!(
                                        "The release weave of spell '{}' does not match the expected weave '{}'",
                                        native_info.name,
                                        expected.get_name()
                                    ),
                                    token,
                                );
                        }
                    }

                    let mut w_reagents: Vec<WovenExpr> = vec![];
                    for (i, reagent) in reagents.iter().enumerate() {
                        let expected = native_info.reagents.get(i).unwrap();
                        let w_expr =
                            self.analyze_expression(reagent.clone(), Some(&expected.weave))?;
                        if w_expr.weave() != expected.weave {
                            return self.error(
                                &format!(
                                    "The reagent #{} was expected to be {}, but got {}",
                                    i + 1,
                                    expected.weave.get_name(),
                                    w_expr.weave().get_name()
                                ),
                                token,
                            );
                        }
                        w_reagents.push(w_expr.clone());
                    }

                    return Ok(WovenExpr::NativeCast {
                        reagents: w_reagents,
                        callee: token,
                        weave: native_info.release_weave,
                        native_spell,
                    });
                }

                let w_callee = w_callee?;

                if !w_callee.weave().get_tapestry().has_strand(CALLABLE_STRAND) {
                    return self.error(
                        "Cannot perform cast on a compile-time unknown spell. Only direct sign method calls are allowed to be casted for now.",
                        token,
                    );
                }

                // atp its usually a variable expr. If its not, well... good luck ig
                let (spell_info, spell_symbol) = match w_callee {
                    WovenExpr::Variable { symbol, .. } => {
                        if let SymbolKind::Spell(si) = &*symbol.kind.borrow() {
                            (si.clone(), symbol.clone())
                        } else {
                            let mut spell_info: Option<SpellInfo> = None;
                            let mut s = symbol.clone();
                            while let Some(p) = s.parent {
                                if let SymbolKind::Spell(si) = &*p.kind.borrow() {
                                    spell_info = Some(si.clone());
                                    break;
                                }
                                s = Rc::unwrap_or_clone(p);
                            }

                            match spell_info {
                                Some(si) => (si, symbol),

                                // if not found, try checking Native Spells
                                None => {
                                    return self.error("Only spells can be casted!", token);
                                }
                            }
                        }
                    }
                    _ => {
                        // this should be unreachable.. if im not wrong
                        return self.error("Eira can only cast a spell from a variable!", token);
                    }
                };

                if reagents.len() != spell_info.reagents.len() {
                    return self.error(
                        &format!(
                            "The spell '{}' expected {} reagent(s), but you provided {} of them!",
                            spell_info.name,
                            spell_info.reagents.len(),
                            reagents.len()
                        ),
                        token,
                    );
                }

                if let Some(expected) = expected_weave {
                    if *expected != spell_info.release_weave {
                        return self.error(
                                    &format!(
                                        "The release weave of spell '{}' does not match the expected weave '{}'",
                                        spell_info.name,
                                        expected.get_name()
                                    ),
                                    token,
                                );
                    }
                }

                let mut final_reagents: Vec<WovenExpr> = vec![];

                for r in reagents {
                    final_reagents.push(self.analyze_expression(r, None)?);
                }

                // let Some(spell_symbol) = self.symbol_table.resolve(&spell_info.name) else {
                //     return self.error(
                //         &format!("Spell symbol not found while casting! for {}", token),
                //         token,
                //     );
                // };

                Ok(WovenExpr::Cast {
                    callee: token.clone(),
                    reagents: final_reagents,
                    spell_symbol: spell_symbol,
                    weave: spell_info.release_weave,
                })
            }
            Expr::Draw { marks, callee } => {
                let Some(symbol) = self.symbol_table.resolve(&callee.lexeme).cloned() else {
                    return self.error(
                        &format!("The sign '{}' was not found!", callee.lexeme),
                        callee,
                    );
                };

                let sign_info = {
                    let Some(info) = symbol.kind.borrow().get_sign_info() else {
                        return self.error(&format!("'{}' is not a sign!", symbol.name), callee);
                    };

                    info.clone()
                };

                // Will have to change for optional fields
                if sign_info.marks.len() != marks.len() {
                    return self.error(
                        &format!(
                            "The sign '{}' expected {} marks, but you provided{} {} of them!",
                            callee.lexeme,
                            sign_info.marks.len(),
                            if marks.len() < sign_info.marks.len() {
                                " only"
                            } else {
                                ""
                            },
                            marks.len()
                        ),
                        callee,
                    );
                }

                let mut w_marks: Vec<WovenEtchedMark> = vec![];
                for mark in marks {
                    if let Some(field) = sign_info.marks.get(&mark.name.lexeme) {
                        // set blank as a way to set empty value
                        let mark_val = match mark.expr {
                            Expr::Blank { token } => WovenExpr::Literal {
                                value: Value::Emptiness,
                                token: token,
                                weave: Weave::Empty,
                            },
                            _ => self.analyze_expression(mark.expr, None)?,
                        };

                        let mark_weave = mark_val.weave();
                        if self.can_assign(field, &mark_weave) {
                            w_marks.push(WovenEtchedMark {
                                name: mark.name.clone(),
                                expr: mark_val.clone(),
                            })
                        } else {
                            return self.error(
                                &format!(
                                    "The mark '{}' was expected to have weave '{}' but got '{}'",
                                    mark.name.lexeme,
                                    field.get_name(),
                                    mark_weave.get_name()
                                ),
                                mark.name,
                            );
                        }
                    } else {
                        return self.error(
                            &format!(
                                "The mark '{}' doesn't exist inside {}",
                                mark.name.lexeme, callee.lexeme
                            ),
                            mark.name,
                        );
                    }
                }

                Ok(WovenExpr::Draw {
                    marks: w_marks,
                    callee: callee.clone(),
                    weave: Weave::Sign(sign_info.schema.name.clone()),
                    sign_symbol: symbol.clone(),
                })
            }
            Expr::Access { .. } | Expr::SafeAccess { .. } => {
                let (is_safe_access, material, property) = match expr {
                    Expr::Access { material, property } => (false, material, property),
                    Expr::SafeAccess { material, property } => (true, material, property),
                    _ => unreachable!(),
                };

                let w_material = self.analyze_expression(*material, None)?;
                if let Weave::Module(module_name) = w_material.weave() {
                    if is_safe_access {
                        return self.error(
                            "You don't have to use '?.' for accessing tethered scrolls.",
                            property,
                        );
                    }

                    let symbol = match w_material {
                        WovenExpr::Variable { ref symbol, .. } => symbol.clone(),
                        _ => {
                            return self
                                .error(&format!("{} is not a mark or a bind", property), property);
                        }
                    };

                    let mod_info = symbol.kind.borrow();

                    let Some(module) = mod_info.get_module_info() else {
                        // this shouldnt be thrown (if im not wrong)
                        return self.error(
                            &format!("The scroll tethered to '{}' doesn't exist.", module_name),
                            property,
                        );
                    };

                    let Some(sym) = module.get(&property.lexeme) else {
                        return self.error(
                            &format!(
                                "The symbol '{}' cannot be found in the scroll '{}'",
                                property.lexeme, module_name
                            ),
                            property,
                        );
                    };

                    if sym.visibility == Visibility::Secret {
                        return self.error(
                            &format!(
                                "The '{}' is currently a secret inside the scroll!",
                                property.lexeme
                            ),
                            property,
                        );
                    }

                    // return match sym.kind.borrow().clone() {
                    // SymbolKind::Variable { mutable } =>
                    return Ok(WovenExpr::Variable {
                        name: property.clone(),
                        weave: sym.weave.clone(),
                        symbol: sym.clone(),
                    });
                    // SymbolKind::Spell(spell_info) => Ok(WovenExpr::BoundSpell {
                    //     is_safe: false,
                    //     material: Box::new(w_material),
                    //     spell_symbol: sym.clone(),
                    //     token: property.clone(),
                    //     weave: spell_info.release_weave,
                    // }),
                    // _ => {
                    // return self.error("h,", property);
                    // }
                    // };
                }

                let sign_name = match (is_safe_access, w_material.weave()) {
                    (false, Weave::Sign(s)) => s,
                    (true, Weave::Maybe(inner)) => {
                        if let Weave::Sign(s) = *inner {
                            s
                        } else {
                            return self.error(
                                &format!(
                                    "Only signs can be accessed with '.' operator! Got {}.",
                                    inner.get_name()
                                ),
                                property,
                            );
                        }
                    }
                    (true, _) => {
                        return self.error(
                            "Safe access operation (?.) is only possible for Maybe<W>.",
                            property,
                        );
                    }

                    (false, w) => {
                        return self.error(
                            &format!(
                                "Only signs can be accessed with '.' operator! Got {}.",
                                w.get_name()
                            ),
                            property,
                        );
                    }
                };

                // wether the material passed is the defined name of sign
                let is_declared_symbol = sign_name == w_material.token().lexeme.as_str();
                // let w_material = self.analyze_expression(*material, None)?;
                // // it should be a variable expression
                // let sign_name = match w_material.weave() {
                //     Weave::Sign(s) => s,
                //     _ => {
                //         return self
                //             .error("Only signs can be accessed with '.' operator!", property);
                //     }
                // };

                let Some(sign_symbol) = self.symbol_table.resolve(&sign_name) else {
                    return self.error(
                        &format!(
                            "The sign '{}' was not found across the eira realms!",
                            sign_name
                        ),
                        property,
                    );
                };

                let Some(sign_info) = sign_symbol.kind.borrow().get_sign_info() else {
                    return self.error(&format!("'{}' is not a sign!", sign_symbol.name), property);
                };

                if let Some(mark) = sign_info.schema.get_field_index(property.lexeme.clone()) {
                    let Some(property_weave) = sign_info.marks.get(&property.lexeme) else {
                        return self.error(
                            &format!(
                                "Eira couldn't find the weave for property '{}'",
                                property.lexeme
                            ),
                            property,
                        );
                    };

                    if is_safe_access {
                        return Ok(WovenExpr::SafeAccess {
                            material: Box::new(w_material),
                            property,
                            field_name_idx: mark as u16,
                            weave: Weave::Maybe(Box::new(property_weave.clone())),
                        });
                    }

                    return Ok(WovenExpr::Access {
                        material: Box::new(w_material),
                        property,
                        field_name_idx: mark as u16,
                        weave: property_weave.clone(),
                    });
                }

                if let Some(attunement) = sign_info.attunements.get(&property.lexeme) {
                    if is_declared_symbol && !attunement.is_static {
                        return self.error(
                            "Attunements cannot be invoked directly from the sign!",
                            property,
                        );
                    } else if !is_declared_symbol && attunement.is_static {
                        return self.error(
                            "Static attunements can only be invoked directly from the sign!",
                            property,
                        );
                    }

                    if attunement.visibility == Visibility::Secret
                        && w_material.token().token_type != TokenType::Ego
                    {
                        return self.error(
                                &format!(
                                    "The spell '{}' attuned to sign '{}' is a secret and cannot be casted here!",
                                    attunement.method_name, sign_name,
                                ),
                                property,
                            );
                    }

                    let spell_symbol = match self.symbol_table.resolve(&attunement.method_name) {
                        Some(s) => s.clone(),
                        None => {
                            return self.error(
                                &format!(
                                    "The spell '{}' was not found for sign '{}'!",
                                    attunement.method_name, sign_name
                                ),
                                property,
                            );
                        }
                    };

                    return Ok(WovenExpr::BoundSpell {
                        is_safe: is_safe_access,
                        material: Box::new(w_material),
                        spell_symbol: spell_symbol.clone(),
                        token: property.clone(),
                        weave: spell_symbol.weave.clone(),
                    });
                }

                return self.error(
                    &format!(
                        "The mark or spell '{}' is not defined for '{}'",
                        property.lexeme, sign_name
                    ),
                    property,
                );
            }
            Expr::Deck { elements, token } => {
                let mut w_elements = vec![];

                let mut expected_capacity: Option<usize> = None;
                let mut prev_elem_weave: Option<Weave> = match expected_weave {
                    Some(w) => match w {
                        Weave::Deck(inner, c) => {
                            expected_capacity = *c;
                            Some(*inner.clone())
                        }
                        _ => {
                            return self.error(
                                &format!(
                                    "Hows this possible? a {} weave passed on to a deck!",
                                    w.get_name()
                                ),
                                token,
                            );
                        }
                    },
                    None => None,
                };

                if elements.len() > u8::MAX as usize {
                    return self.error("Deck size exceeds the maximum of 255 elements!", token);
                }

                if let Some(c) = expected_capacity {
                    if elements.len() > c {
                        return self.error(
                            &format!(
                                "The deck's specified capacity is {} while the length is {}",
                                c,
                                elements.len()
                            ),
                            token,
                        );
                    }
                }

                for element in &elements {
                    let w_element = self.analyze_expression(element.clone(), None)?;
                    let elem_weave = w_element.weave();
                    if let Some(prev_weave) = prev_elem_weave {
                        if elem_weave != prev_weave {
                            return self
                                .error("All elements of a deck must be of the same weave!", token);
                        }
                    }
                    prev_elem_weave = Some(elem_weave);
                    w_elements.push(w_element);
                }

                let weave = Weave::Deck(
                    Box::new(prev_elem_weave.unwrap_or(Weave::Empty)),
                    expected_capacity,
                );

                Ok(WovenExpr::Deck {
                    elements: w_elements,
                    weave: weave,
                })
            }
            Expr::Extract { deck, index, token } => {
                let w_deck = self.analyze_expression(*deck, None)?;
                let elem_weave = match w_deck.weave() {
                    Weave::Deck(weave, _) => *weave,
                    _ => {
                        return self.error(
                            &format!(
                                "'{}' was expected to be a 'Deck' but its a '{}'!",
                                w_deck.token().lexeme,
                                w_deck.weave().get_name(),
                            ),
                            token,
                        );
                    }
                };

                let w_index = self.analyze_expression(*index, Some(&Weave::Num))?;

                let index_weave = w_index.weave();

                if index_weave != Weave::Num {
                    return self.error(
                        "The index expression of a deck set operation must be of NumWeave!",
                        token.clone(),
                    );
                }

                Ok(WovenExpr::Extract {
                    deck: Box::new(w_deck),
                    index: Box::new(w_index),
                    weave: elem_weave,
                    token,
                })
            }
            Expr::DeckSet {
                deck,
                index,
                value,
                token,
            } => {
                let w_deck = self.analyze_expression(*deck, None)?;
                let w_index = self.analyze_expression(*index, Some(&Weave::Num))?;
                let w_value = self.analyze_expression(*value, None)?;

                let index_weave = w_index.weave();

                if index_weave != Weave::Num {
                    return self.error(
                        "The index expression of a deck set operation must be of NumWeave!",
                        token.clone(),
                    );
                }

                Ok(WovenExpr::DeckSet {
                    deck: Box::new(w_deck),
                    index: Box::new(w_index),
                    value: Box::new(w_value.clone()),
                    weave: w_value.weave(),
                    token,
                })
            }
            Expr::FieldSet {
                material,
                property,
                value,
            } => {
                let w_material_token = match self.analyze_expression(*material, None)? {
                    WovenExpr::Variable { name, .. } => name,
                    _ => {
                        return self.error(
                            "Only variables can be accessed with '.' operator!",
                            property,
                        );
                    }
                };

                let Some(symbol) = self.symbol_table.resolve(&w_material_token.lexeme).cloned()
                else {
                    return self.error(
                        &format!(
                            "The mark '{}' was not found across the eira realms!",
                            w_material_token.lexeme
                        ),
                        w_material_token,
                    );
                };

                let sign_name = match symbol.weave {
                    Weave::Sign(ref name) => name,
                    _ => {
                        return self.error(
                            "The mark 'n' is not a material of a sign!",
                            w_material_token,
                        );
                    }
                };

                let Some(sign_symbol) = self.symbol_table.resolve(sign_name) else {
                    return self.error(
                        &format!(
                            "The sign '{}' was not found across the eira realms!",
                            sign_name
                        ),
                        w_material_token,
                    );
                };

                let Some(sign_info) = sign_symbol.kind.borrow().get_sign_info() else {
                    return self.error(
                        &format!("'{}' is not a sign!", sign_symbol.name),
                        w_material_token,
                    );
                };

                let Some(mark) = sign_info.schema.get_field_index(property.lexeme.clone()) else {
                    return self.error(
                        &format!(
                            "The mark '{}' is not defined for '{}'",
                            property.lexeme, sign_name
                        ),
                        property,
                    );
                };

                let Some(property_weave) = sign_info.marks.get(&property.lexeme) else {
                    return self.error(
                        &format!(
                            "Eira couldn't find the weave for property '{}'",
                            property.lexeme
                        ),
                        w_material_token,
                    );
                };

                let w_material_expr = WovenExpr::Variable {
                    name: w_material_token,
                    weave: symbol.weave.clone(),
                    symbol: symbol,
                };

                let w_value = self.analyze_expression(*value, None)?;
                Ok(WovenExpr::FieldSet {
                    material: Box::new(w_material_expr),
                    property,
                    value: Box::new(w_value),
                    field_name_idx: mark as u16,
                    weave: property_weave.clone(),
                })
            }
            Expr::Blank { token } => {
                return self.error(
                    "Invalid '_' usage. '_' is used to assign a Empty value to Maybe<W> weaves!",
                    token,
                );
            }
            Expr::Manifests { value, token } => {
                let w_value = self.analyze_expression(*value, None)?;

                // if !matches!(w_value.weave(), Weave::Maybe(_)) {
                //     return self.error(
                //         "The weave of the manifest expression must be a Maybe weave.",
                //         token,
                //     );
                // }

                Ok(WovenExpr::Manifests {
                    value: Box::new(w_value),
                    token,
                    weave: Weave::Truth,
                })
            }
            Expr::AssertSafe { operand, operator } => {
                let w_operand = self.analyze_expression(*operand, None)?;

                let weave = match w_operand.weave() {
                    Weave::Maybe(inner) => *inner,
                    _ => {
                        return self.error(
                            "Safe Assertion can only be performed on Maybe<W> weaves!",
                            operator,
                        );
                    }
                };

                Ok(WovenExpr::AssertSafe {
                    operand: Box::new(w_operand),
                    operator,
                    weave: weave,
                })
            }
            Expr::Cursed { span } => todo!(),
        }
    }

    pub(super) fn analyze_parsed_weave(&mut self, parsed_weave: ParsedWeave) -> WeaveResult<Weave> {
        let Some(base_weave) = self.get_weave_from_name(&parsed_weave.base.lexeme) else {
            return self.error(
                &format!(
                    "Couldn't find {} weave across the realms of eira!",
                    parsed_weave.base.lexeme
                ),
                parsed_weave.base,
            );
        };

        if !base_weave.can_sub_weave() && parsed_weave.inner.is_some() {
            return self.error(
                &format!(
                    "{} weave cannot contain sub weaves!",
                    parsed_weave.base.lexeme
                ),
                parsed_weave.base,
            );
        }

        // at this point, its sure that only the sub weave-able weaves are processed
        let Some(inner_parsed_weave) = parsed_weave.inner else {
            return Ok(base_weave);
        };

        let inner_weave = self.analyze_parsed_weave(*inner_parsed_weave.clone())?;

        let weave = match base_weave {
            Weave::Deck(..) => {
                let res =
                    Weaver::weave_deck(base_weave, inner_weave.clone(), parsed_weave.capacity);
                if res.is_err() {
                    return self.error(
                        &format!(
                            "Couldnt weave {} to {}",
                            parsed_weave.base.lexeme,
                            inner_weave.get_name()
                        ),
                        inner_parsed_weave.base,
                    );
                }
                res.unwrap()
            }
            Weave::Spell { .. } => {
                let res = Weaver::weave_spell(base_weave, inner_weave);
                if res.is_err() {
                    return self.error(
                        &format!(
                            "Couldnt weave {} to {}",
                            parsed_weave.base.lexeme, inner_parsed_weave.base.lexeme
                        ),
                        inner_parsed_weave.base,
                    );
                }
                res.unwrap()
            }
            Weave::Maybe(_) => {
                let res = Weaver::weave_maybe(base_weave, inner_weave);
                if res.is_err() {
                    return self.error(
                        &format!(
                            "Couldnt weave {} to {}",
                            parsed_weave.base.lexeme, inner_parsed_weave.base.lexeme
                        ),
                        inner_parsed_weave.base,
                    );
                }
                res.unwrap()
            }
            _ => {
                return self.error(
                    &format!(
                        "{} weave cannot contain any sub weaves!",
                        parsed_weave.base.lexeme
                    ),
                    parsed_weave.base,
                );
            }
        };

        Ok(weave)
    }

    fn get_core_scroll(&self, name: &str) -> Option<&str> {
        match name {
            "math" => Some(include_str!("../../../core_scrolls/math.eira")),
            _ => None,
        }
    }

    fn can_assign(&self, expected: &Weave, provided: &Weave) -> bool {
        if expected == provided {
            return true;
        }

        match (expected, provided) {
            (Weave::Maybe(ai), b) => {
                if *b == Weave::Empty {
                    return true;
                }
                ai.as_ref() == provided
            }
            // (_, Weave::Maybe(bi)) => self.match_weave(expected, bi),
            _ => false,
        }
    }

    /// Resolve and add an upvalue for a symbol
    fn resolve_n_add_upvalue(&mut self, symbol: &Symbol) -> WeaveResult<()> {
        // Only capture as upvalue if variable is from the spell's defining scope or outer
        // Parameters and locals have depth greater than the spell base depth
        if self.current_realm == Realm::Spell && symbol.depth <= self.spell_base_depth {
            // check if new. use both index and depth to avoid duplicates
            let slot = symbol.slot_idx;
            let is_new = !self
                .current_upvalues
                .iter()
                .any(|it| it.index == slot && it.depth == symbol.depth);

            if is_new {
                self.current_upvalues.push(UpValue {
                    index: slot,
                    closed: Value::Emptiness.into(),
                    depth: symbol.depth,
                });
            }
        }
        Ok(())
    }

    fn strand_from_op(&self, op: TokenType) -> Option<u64> {
        match op {
            TokenType::Plus => Some(ADDITIVE_STRAND | CONCATINABLE_STRAND),
            TokenType::Minus => Some(SUBTRACTIVE_STRAND),
            TokenType::Star => Some(MULTIPLICATIVE_STRAND),
            TokenType::Slash => Some(DIVISIVE_STRAND),
            TokenType::Percent => Some(DIVISIVE_STRAND),
            TokenType::Bang => Some(CONDITIONAL_STRAND),
            TokenType::Greater
            | TokenType::Less
            | TokenType::GreaterEqual
            | TokenType::LessEqual => Some(ORDINAL_STRAND),
            TokenType::EqualEqual | TokenType::BangEqual => Some(EQUATABLE_STRAND),
            _ => None,
        }
    }

    /// Get the strand's name from its bit representation
    fn strand_string_from_bits(&self, strand: u64) -> &str {
        match strand {
            ADDITIVE_STRAND => "ADDITIVE",
            SUBTRACTIVE_STRAND => "SUBTRACTIVE",
            MULTIPLICATIVE_STRAND => "MULTIPLICATIVE",
            DIVISIVE_STRAND => "DIVISIVE",
            ORDINAL_STRAND => "ORDINAL",
            CONDITIONAL_STRAND => "CONDITIONAL",
            CONCATINABLE_STRAND => "CONCATINABLE",
            INDEXIVE_STRAND => "INDEXIVE",
            ITERABLE_STRAND => "ITERABLE",
            EQUATABLE_STRAND => "EQUATABLE",
            CALLABLE_STRAND => "CALLABLE",
            MAYBE_STRAND => "MAYBE",
            NO_STRAND => "NONE",
            _ => "UNKNOWN",
        }
    }

    fn get_weave_from_name(&mut self, name: &str) -> Option<Weave> {
        match name {
            "Num" => Some(Weave::Num),
            "Text" => Some(Weave::Text),
            "Truth" => Some(Weave::Truth),
            "Empty" => Some(Weave::Empty),
            "Sign" => Some(Weave::Sign(String::new())),
            "Spell" => Some(Weave::Spell {
                // reagents: vec![],
                release: Box::new(Weave::Empty),
            }),
            "Deck" => Some(Weave::Deck(Box::new(Weave::Empty), None)),
            "Maybe" => Some(Weave::Maybe(Box::new(Weave::Empty))),
            _ => {
                // match user defined types!
                let Some(symbol) = self.symbol_table.resolve(&name.to_string()) else {
                    return None;
                };

                if symbol.kind.borrow().get_sign_info().is_some() {
                    Some(Weave::Sign(name.to_owned()))
                } else {
                    None
                }
            }
        }
    }
}
