use std::collections::HashMap;
use tinylang::eval;
use tinylang::types::TinyLangType;

#[test]
fn else_inside_disabled_if() {
    // outer if is false, so NOTHING inside it should be output
    let template = "{% if false %}A{% if true %}B{% else %}C{% end %}D{% end %}";
    let result = eval(template, HashMap::default()).unwrap();
    assert_eq!("", result.as_str());
}

#[test]
fn standalone_negation() {
    let result = eval("{{ -1 }}", HashMap::default());
    println!("standalone neg: {:?}", result);
    assert_eq!("-1", result.unwrap().as_str());
}

#[test]
fn standalone_negation_identifier() {
    let result = eval(
        "{{ -a }}",
        HashMap::from([("a".into(), TinyLangType::Numeric(5.0))]),
    );
    println!("neg ident: {:?}", result);
    assert_eq!("-5", result.unwrap().as_str());
}

#[test]
fn loop_var_restored_after_loop() {
    let template = "{% for a in b %}{{ a }}{% end %}{{ a }}";
    let result = eval(
        template,
        HashMap::from([
            ("b".into(), TinyLangType::Vec(vec![1.into(), 2.into()])),
            ("a".into(), TinyLangType::String("orig".into())),
        ]),
    )
    .unwrap();
    assert_eq!("12orig", result.as_str());
}

#[test]
fn nested_for_loops() {
    let template = "{% for a in v %}{% for b in v %}{{ a }}{{ b }} {% end %}{% end %}";
    let result = eval(
        template,
        HashMap::from([("v".into(), TinyLangType::Vec(vec![1.into(), 2.into()]))]),
    )
    .unwrap();
    assert_eq!("11 12 21 22 ", result.as_str());
}

#[test]
fn if_inside_for() {
    let template = "{% for a in v %}{% if a == 2 %}two{% else %}other{% end %}{% end %}";
    let result = eval(
        template,
        HashMap::from([(
            "v".into(),
            TinyLangType::Vec(vec![1.into(), 2.into(), 3.into()]),
        )]),
    )
    .unwrap();
    assert_eq!("othertwoother", result.as_str());
}

#[test]
fn for_inside_disabled_if_preserves_var() {
    let template = "{% if false %}{% for a in b %}{{ a }}{% end %}{% end %}{{ a }}";
    let result = eval(
        template,
        HashMap::from([
            ("b".into(), TinyLangType::Vec(vec![1.into(), 2.into()])),
            ("a".into(), TinyLangType::String("orig".into())),
        ]),
    )
    .unwrap();
    assert_eq!("orig", result.as_str());
}

#[test]
fn missing_end() {
    let template = "{% if true %}hello";
    let result = eval(template, HashMap::default());
    println!("missing end: {:?}", result);
    assert!(result.is_err());
}

#[test]
fn comparison_less() {
    let result = eval("{{ 1 < 2 }}", HashMap::default()).unwrap();
    assert_eq!("true", result.as_str());
}

#[test]
fn else_inside_enabled_if_still_works() {
    let template = "{% if true %}{% if false %}A{% else %}B{% end %}{% end %}";
    let result = eval(template, HashMap::default()).unwrap();
    assert_eq!("B", result.as_str());

    let template = "{% if true %}{% if true %}A{% else %}B{% end %}{% end %}";
    let result = eval(template, HashMap::default()).unwrap();
    assert_eq!("A", result.as_str());
}

#[test]
fn negation_with_spaces_and_parens() {
    assert_eq!(eval("{{ - 1 }}", HashMap::default()).unwrap(), "-1");
    assert_eq!(eval("{{ -(1 + 2) }}", HashMap::default()).unwrap(), "-3");
    assert_eq!(eval("{{ -1 * -1 }}", HashMap::default()).unwrap(), "1");
}

#[test]
fn missing_end_for() {
    let result = eval("{% for a in b %}{{ a }}", HashMap::default());
    assert!(result.is_err());
}

#[test]
fn negation_inside_if() {
    let result = eval(
        "{% if -a == -1 %}yes{% end %}",
        HashMap::from([("a".into(), TinyLangType::Numeric(1.0))]),
    )
    .unwrap();
    assert_eq!("yes", result.as_str());
}
