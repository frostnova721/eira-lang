#[cfg(test)]
mod native_spells_test {
    use eira::{
        Parser, Scanner,
        compiler::{
            ast::decl::WovenDecl,
            diagnostics::Augury,
            weave_analyser::WeaveAnalyzerContext,
            weaves::Weave,
        },
        values::native_spell::NativeSpell,
        weave_analyser::WeaveAnalyzer,
    };

    fn analyze_helper(source: &str) -> Result<Vec<WovenDecl>, String> {
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();
        let parser = Parser::new(tokens, "native_test.eira".to_string());
        let ast = parser
            .parse()
            .map_err(|e| format!("Parse error: {:?}", e))?;
        let mut augury = Augury::new();
        let mut context = WeaveAnalyzerContext::new("native_test.eira".to_string(), None, false);
        let mut wa = WeaveAnalyzer::new(&mut context, &mut augury);
        let decls = wa.analyze(ast).map_err(|e| format!("{}", e.msg))?;
        if augury.is_cursed() {
            return Err(augury
                .curses
                .iter()
                .map(|c| c.message.clone())
                .collect::<Vec<_>>()
                .join("\n"));
        }
        Ok(decls)
    }

    #[test]
    fn test_deck_methods_resolution() {
        let deck_weave = Weave::Deck(Box::new(Weave::Num), None);

        let push = NativeSpell::resolve_methods("core:Deck:push", deck_weave.clone()).unwrap();
        let info = NativeSpell::get_spell_info(push).unwrap();
        assert_eq!(info.name, "push");
        assert_eq!(info.reagents.len(), 2);
        assert_eq!(info.release_weave, Weave::Empty);

        let size = NativeSpell::resolve_methods("core:Deck:size", deck_weave.clone()).unwrap();
        let info = NativeSpell::get_spell_info(size).unwrap();
        assert_eq!(info.name, "size");
        assert_eq!(info.release_weave, Weave::Num);

        let pop = NativeSpell::resolve_methods("core:Deck:pop", deck_weave.clone()).unwrap();
        let info = NativeSpell::get_spell_info(pop).unwrap();
        assert_eq!(info.name, "pop");
        assert_eq!(info.release_weave, Weave::Maybe(Box::new(Weave::Num)));

        let is_empty = NativeSpell::resolve_methods("core:Deck:is_empty", deck_weave.clone()).unwrap();
        let info = NativeSpell::get_spell_info(is_empty).unwrap();
        assert_eq!(info.name, "is_empty");
        assert_eq!(info.release_weave, Weave::Truth);
    }

    #[test]
    fn test_text_methods_resolution() {
        let text_weave = Weave::Text;

        let upper = NativeSpell::resolve_methods("core:Text:to_upper", text_weave.clone()).unwrap();
        let info = NativeSpell::get_spell_info(upper).unwrap();
        assert_eq!(info.name, "to_upper");
        assert_eq!(info.release_weave, Weave::Text);

        let contains = NativeSpell::resolve_methods("core:Text:contains", text_weave.clone()).unwrap();
        let info = NativeSpell::get_spell_info(contains).unwrap();
        assert_eq!(info.name, "contains");
        assert_eq!(info.reagents.len(), 2);
        assert_eq!(info.release_weave, Weave::Truth);

        let extract = NativeSpell::resolve_methods("core:Text:extract", text_weave.clone()).unwrap();
        let info = NativeSpell::get_spell_info(extract).unwrap();
        assert_eq!(info.name, "extract");
        assert_eq!(info.reagents.len(), 2);
        assert_eq!(info.release_weave, Weave::Text);
    }

    #[test]
    fn test_num_methods_resolution() {
        let num_weave = Weave::Num;

        let floor = NativeSpell::resolve_methods("core:Num:floor", num_weave.clone()).unwrap();
        let info = NativeSpell::get_spell_info(floor).unwrap();
        assert_eq!(info.name, "floor");
        assert_eq!(info.release_weave, Weave::Num);

        let clamp = NativeSpell::resolve_methods("core:Num:clamp", num_weave.clone()).unwrap();
        let info = NativeSpell::get_spell_info(clamp).unwrap();
        assert_eq!(info.name, "clamp");
        assert_eq!(info.reagents.len(), 3); // receiver + min + max
        assert_eq!(info.release_weave, Weave::Num);
    }

    #[test]
    fn test_standalone_spells_resolution() {
        let floor = NativeSpell::resolve("floor").unwrap();
        let info = NativeSpell::get_spell_info(floor).unwrap();
        assert_eq!(info.name, "floor");
        assert_eq!(info.reagents.len(), 1);

        let ask = NativeSpell::resolve("ask").unwrap();
        let info = NativeSpell::get_spell_info(ask).unwrap();
        assert_eq!(info.name, "ask");
        assert_eq!(info.reagents.len(), 1);

        let listen = NativeSpell::resolve("listen").unwrap();
        let info = NativeSpell::get_spell_info(listen).unwrap();
        assert_eq!(info.name, "listen");
        assert_eq!(info.reagents.len(), 0);
    }

    #[test]
    fn test_weave_analysis_of_native_method_calls() {
        let src = r#"
            mark arr = [1, 2, 3];
            cast arr.push with 4;
            mark s = "hello world";
            mark upper = cast s.to_upper;
        "#;
        let stmts = analyze_helper(src).expect("should analyze native method calls successfully");
        assert_eq!(stmts.len(), 4);
    }
}
