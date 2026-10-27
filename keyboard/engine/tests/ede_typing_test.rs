use chkey_engine::keycode::KeyCode;
use chkey_engine::language::ede::EdeProfile;
use chkey_engine::processor::Processor;
use chkey_engine::state::HookCode;

#[test]
fn test_ede_vowel_telex_transformations() {
    let mut processor = Processor::new(EdeProfile);

    // a + a -> â
    processor.clear();
    let res1 = processor.process(KeyCode::A);
    assert_eq!(res1.backspace_count, 0);

    let res2 = processor.process(KeyCode::A);
    assert_eq!(res2.code, HookCode::Transform as u8);
    assert_eq!(res2.backspace_count, 1);
    assert_eq!(res2.output_string(), "â");

    // a + x -> ă
    processor.clear();
    processor.process(KeyCode::A);
    let res = processor.process(KeyCode::X);
    assert_eq!(res.backspace_count, 1);
    assert_eq!(res.output_string(), "ă");

    // a + a + x -> â̆
    processor.clear();
    processor.process(KeyCode::A);
    processor.process(KeyCode::A);
    let res = processor.process(KeyCode::X);
    assert_eq!(res.backspace_count, 1);
    assert_eq!(res.output_string(), "â\u{0306}");

    // e + e + x -> ê̆
    processor.clear();
    processor.process(KeyCode::E);
    processor.process(KeyCode::E);
    let res = processor.process(KeyCode::X);
    assert_eq!(res.backspace_count, 1);
    assert_eq!(res.output_string(), "ê\u{0306}");

    // o + w + x -> ơ̆
    processor.clear();
    processor.process(KeyCode::O);
    let res_ow = processor.process(KeyCode::W);
    assert_eq!(res_ow.output_string(), "ơ");
    let res_owx = processor.process(KeyCode::X);
    assert_eq!(res_owx.output_string(), "ơ\u{0306}");

    // u + w + x -> ư̆
    processor.clear();
    processor.process(KeyCode::U);
    let res_uw = processor.process(KeyCode::W);
    assert_eq!(res_uw.output_string(), "ư");
    let res_uwx = processor.process(KeyCode::X);
    assert_eq!(res_uwx.output_string(), "ư\u{0306}");
}

#[test]
fn test_ede_consonant_transformations() {
    let mut processor = Processor::new(EdeProfile);

    // d + d -> đ
    processor.clear();
    processor.process(KeyCode::D);
    let res_d = processor.process(KeyCode::D);
    assert_eq!(res_d.output_string(), "đ");

    // b + b -> ƀ
    processor.clear();
    processor.process(KeyCode::B);
    let res_b = processor.process(KeyCode::B);
    assert_eq!(res_b.output_string(), "ƀ");

    // c + x -> č
    processor.clear();
    processor.process(KeyCode::C);
    let res_c = processor.process(KeyCode::X);
    assert_eq!(res_c.output_string(), "č");

    // n + n -> ñ
    processor.clear();
    processor.process(KeyCode::N);
    let res_n = processor.process(KeyCode::N);
    assert_eq!(res_n.output_string(), "ñ");
}

#[test]
fn test_uppercase_and_shift() {
    let mut processor = Processor::new(EdeProfile);

    // Shift + A, Shift + A -> Â
    processor.clear();
    let res1 = processor.process_key(KeyCode::A, true, false);
    assert_eq!(res1.output_string(), "A");

    let res2 = processor.process_key(KeyCode::A, true, false);
    assert_eq!(res2.backspace_count, 1);
    assert_eq!(res2.output_string(), "Â");

    // Shift + D, Shift + D -> Đ
    processor.clear();
    processor.process_key(KeyCode::D, true, false);
    let res_dd = processor.process_key(KeyCode::D, true, false);
    assert_eq!(res_dd.output_string(), "Đ");

    // Shift + B, Shift + B -> Ƀ
    processor.clear();
    processor.process_key(KeyCode::B, true, false);
    let res_bb = processor.process_key(KeyCode::B, true, false);
    assert_eq!(res_bb.output_string(), "Ƀ");
}

#[test]
fn test_revert_undo() {
    let mut processor = Processor::new(EdeProfile);

    // a + a -> â, then a -> revert to aa
    processor.clear();
    processor.process(KeyCode::A);
    let res_aa = processor.process(KeyCode::A);
    assert_eq!(res_aa.output_string(), "â");

    let res_aaa = processor.process(KeyCode::A);
    assert_eq!(res_aaa.code, HookCode::Revert as u8);
    assert_eq!(res_aaa.backspace_count, 1);
    assert_eq!(res_aaa.output_string(), "aa");

    // a + x -> ă, then x -> revert to ax
    processor.clear();
    processor.process(KeyCode::A);
    let res_ax = processor.process(KeyCode::X);
    assert_eq!(res_ax.output_string(), "ă");

    let res_axx = processor.process(KeyCode::X);
    assert_eq!(res_axx.code, HookCode::Revert as u8);
    assert_eq!(res_axx.backspace_count, 1);
    assert_eq!(res_axx.output_string(), "ax");
}

#[test]
fn test_word_boundary_with_punctuation() {
    let mut processor = Processor::new(EdeProfile);

    // Type "klei"
    processor.clear();
    processor.process(KeyCode::K);
    processor.process(KeyCode::L);
    processor.process(KeyCode::E);
    processor.process(KeyCode::I);

    // Type comma ','
    let res_comma = processor.process(KeyCode::Comma);
    assert_eq!(res_comma.code, HookCode::BreakWord as u8);

    // Type "aa" -> â. Must only backspace 1 for 'a', NOT the comma or 'klei'
    processor.process(KeyCode::A);
    let res_a2 = processor.process(KeyCode::A);
    assert_eq!(res_a2.backspace_count, 1);
    assert_eq!(res_a2.output_string(), "â");
}
