use crate::compiler::{WovenStmt, ast::decl::WovenDecl};

/// Defines the possible exits from a statement or block of code, such as return, break, continue, or throw.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Exits {
    pub fallthrough: bool,
    pub returns: bool,
    pub breaks: bool,
    pub continues: bool,
    pub throws: bool,
}

impl Exits {
    fn next() -> Self {
        Exits {
            fallthrough: true,
            ..Self::default()
        }
    }

    // path where any branch can execute
    fn union(&self, other: Self) -> Self {
        Exits {
            fallthrough: self.fallthrough || other.fallthrough,
            returns: self.returns || other.returns,
            breaks: self.breaks || other.breaks,
            continues: self.continues || other.continues,
            throws: self.throws || other.throws,
        }
    }

    /// Sequential statements where only fallthrough reaches next statement
    fn then(&self, next: Self) -> Self {
        if !self.fallthrough {
            return *self;
        }

        Self {
            fallthrough: next.fallthrough,
            returns: self.returns || next.returns,
            breaks: self.breaks || next.breaks,
            continues: self.continues || next.continues,
            throws: self.throws || next.throws,
        }
    }
}

pub(super) fn exits(stmt: &WovenStmt) -> Exits {
    match stmt {
        WovenStmt::Release { .. } => Exits {
            returns: true,
            ..Exits::default()
        },
        WovenStmt::Flow { .. } => Exits {
            continues: true,
            ..Exits::default()
        },
        WovenStmt::Sever { .. } => Exits {
            breaks: true,
            ..Exits::default()
        },
        WovenStmt::Block { statements } => {
            let mut result = Exits::next();

            for s in statements {
                if !result.fallthrough {
                    break;
                }
                result = result.then(exits(s));
            }
            result
        }
        WovenStmt::Fate {
            then_branch,
            else_branch,
            ..
        } => {
            let then_exits = exits(then_branch);
            let else_exits = if let Some(else_branch) = else_branch {
                exits(else_branch)
            } else {
                Exits::next()
            };
            then_exits.union(else_exits)
        }
        WovenStmt::While { body, .. } | WovenStmt::Cycle { body, .. } => {
            let body_exits = exits(body);

            Exits {
                fallthrough: true,
                returns: body_exits.returns,
                breaks: false,
                continues: false,
                throws: false,
            }
        }

        WovenStmt::Declaration(decl) => declaration_exits(decl),

        WovenStmt::ExprStmt { .. } | WovenStmt::Chant { .. } | WovenStmt::Cursed { .. } => {
            Exits::next()
        }
    }
}

fn declaration_exits(decl: &WovenDecl) -> Exits {
    match decl {
        WovenDecl::Statement { stmt, .. } => exits(stmt),
        WovenDecl::Spell { .. }
        | WovenDecl::Attune { .. }
        | WovenDecl::Tether { .. }
        | WovenDecl::Sign { .. }
        | WovenDecl::VarDeclaration { .. }
        | WovenDecl::Cursed { .. } => Exits::next(),
    }
}
