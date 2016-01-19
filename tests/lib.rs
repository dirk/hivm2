extern crate hivm2;

use hivm2::asm_compiler::CompileModule;

#[test]
fn compiles_asm() {
    use hivm2::asm;
    use hivm2::asm::Statement::*;
    use hivm2::asm::{
        Assignment,
        AssignmentOp,
        BasicBlock,
        Call,
        Const,
        Defn,
        Mod,
        Path,
        Return,
        Value
    };

    let function_defn = Defn::new(
        "bar".to_owned(),
        vec!["baz".to_owned()],
        BasicBlock::with_stmts(vec![
            StatementReturn(Return::new(Some(Value::with_name("baz".to_owned()))))
        ])
    );
    let main_defn = Defn::new(
        "main".to_owned(),
        vec![],
        BasicBlock::with_stmts(vec![
            StatementAssignment(Assignment::new(
                "val".to_owned(),
                AssignmentOp::AllocateAndAssign,
                Value::Path(Path::with_name("@hello_world".to_owned()))
            )),
            StatementCall(Call::new(
                Path::with_name("bar".to_owned()),
                vec![]
            ))
        ])
    );
    let hello_world_const = Const::new(
        "@hello_world".to_owned(),
        Path::with_name("_.std.string.new".to_owned()),
        Some("Hello world!".to_owned())
    );

    let module = asm::Module::with_stmts(vec![
        StatementMod(Mod::new(Path::with_name("foo".to_owned()))),
        StatementConst(hello_world_const),
        StatementDefn(function_defn),
        StatementDefn(main_defn),
    ]);
    assert_eq!(module.stmts.len(), 4);
    assert!(module.validate().is_ok());

    // let compiled = module.compile();
}
