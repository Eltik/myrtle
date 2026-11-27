#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_story::parser::*;

fn one(line: &str) -> StoryCommand {
    let v = parse(line);
    assert_eq!(v.len(), 1, "{line}");
    v.into_iter().next().unwrap()
}

#[test]
fn name_form_is_kind_name_with_text() {
    let c = one(r#"[name="Guard"]   Is it just me, or is the sky getting darker?"#);
    assert_eq!(c.kind, "name");
    assert_eq!(c.args.get("name").map(String::as_str), Some("Guard"));
    assert_eq!(
        c.text.as_deref(),
        Some("Is it just me, or is the sky getting darker?")
    );
    assert_eq!(c.line, 1);
}

#[test]
fn bare_command_has_no_args_or_text() {
    let c = one("[Dialog]");
    assert_eq!(c.kind, "dialog");
    assert!(c.args.is_empty());
    assert_eq!(c.text, None);
}

#[test]
fn header_keeps_trailing_text_and_lowercases_keys() {
    let c =
        one(r#"[HEADER(key="title_test", is_skippable=true, fit_mode="BLACK_MASK")] 第七关（前）"#);
    assert_eq!(c.kind, "header");
    assert_eq!(c.args["key"], "title_test");
    assert_eq!(c.args["is_skippable"], "true");
    assert_eq!(c.args["fit_mode"], "BLACK_MASK");
    assert_eq!(c.text.as_deref(), Some("第七关（前）"));
}

#[test]
fn arg_keys_lowercase_values_keep_case() {
    let c = one(r#"[Background(image="bg_Cher_1", x=0, xScale=1.1, Delay=0.5)]"#);
    assert_eq!(c.args["image"], "bg_Cher_1");
    assert_eq!(c.args["xscale"], "1.1");
    assert_eq!(c.args["delay"], "0.5");
    assert!(!c.args.contains_key("xScale"));
}

#[test]
fn quoted_commas_survive() {
    let c = one(
        r#"[Decision(options="You sure are dedicated.;...How stubborn, really.", values="1;2")]"#,
    );
    assert_eq!(
        c.args["options"],
        "You sure are dedicated.;...How stubborn, really."
    );
    assert_eq!(c.args["values"], "1;2");
}

#[test]
fn spaces_around_equals_and_escaped_quotes() {
    let c = one(
        r#"[Sticker(id="st1", multi = true, text="He said \"no\", twice", x=300,block = true)]"#,
    );
    assert_eq!(c.kind, "sticker");
    assert_eq!(c.args["multi"], "true");
    assert_eq!(c.args["text"], r#"He said "no", twice"#);
    assert_eq!(c.args["x"], "300");
    assert_eq!(c.args["block"], "true");
}

#[test]
fn bare_text_line_is_kind_text() {
    let c = one("   The rain had not stopped for three days.  ");
    assert_eq!(c.kind, "text");
    assert!(c.args.is_empty());
    assert_eq!(
        c.text.as_deref(),
        Some("The rain had not stopped for three days.")
    );
}

#[test]
fn typos_are_normalised() {
    assert_eq!(one(r#"[palysound(key="$x")]"#).kind, "playsound");
    assert_eq!(one("[Delya(time=1)]").kind, "delay");
    assert_eq!(one("[stopmucis]").kind, "stopmusic");
    assert_eq!(one("[musicvolune(volume=0)]").kind, "musicvolume");
    assert_eq!(one("[dialo]").kind, "dialog");
    assert_eq!(one("[charslsot]").kind, "charslot");
    assert_eq!(one("[chaa]").kind, "character");
    let c = one("[delay9ti=2]");
    assert_eq!(c.kind, "delay");
    assert_eq!(c.args["delay"], "2");
}

#[test]
fn key_value_form_generalises() {
    let c = one("[isAvatarRight=false]Databank search detected.");
    assert_eq!(c.kind, "isavatarright");
    assert_eq!(c.args["isavatarright"], "false");
    assert_eq!(c.text.as_deref(), Some("Databank search detected."));
    let c = one("[Delay=(time=1.5)]");
    assert_eq!(c.kind, "delay");
    assert_eq!(c.args["time"], "1.5");
}

#[test]
fn unknown_commands_pass_through_lowercased() {
    let c = one(r#"[SandboxV2(mode="x")]"#);
    assert_eq!(c.kind, "sandboxv2");
    assert_eq!(c.args["mode"], "x");
}

#[test]
fn empty_lines_are_skipped_and_line_numbers_are_file_lines() {
    let v = parse("[Dialog]\n\n\n[name=\"A\"] hi\n");
    assert_eq!(v.len(), 2);
    assert_eq!(v[1].line, 4);
}

#[test]
fn doubled_brackets_and_empty_brackets() {
    let c = one("[[character(fadetime=0.3)]]");
    assert_eq!(c.kind, "character");
    assert_eq!(c.args["fadetime"], "0.3");
    assert_eq!(c.text, None);
    assert!(parse("[]\n").is_empty());
}

#[test]
fn unclosed_bracket_is_text() {
    let c = one("[Broken(line");
    assert_eq!(c.kind, "text");
    assert_eq!(c.text.as_deref(), Some("[Broken(line"));
}

#[test]
fn a_trailing_backslash_joins_the_next_line() {
    let script = concat!(
        "[Tutorial(focusX=178, focusY=176,  \\\n",
        "          animStyle=\"Highlight\", black=\"$f_tut_black\", \\\n",
        "          protectTime=0.5)] \\\n",
        "[name=\"A\"] hi\n"
    );
    let (v, stats) = parse_with_stats(script);
    assert_eq!(v.len(), 1, "{v:?}");
    assert_eq!(v[0].kind, "tutorial");
    assert_eq!(
        v[0].line, 1,
        "the joined line keeps the FIRST line's number"
    );
    assert_eq!(v[0].args["focusx"], "178");
    assert_eq!(v[0].args["animstyle"], "Highlight");
    assert_eq!(v[0].args["protecttime"], "0.5");
    assert_eq!(v[0].text.as_deref(), Some("[name=\"A\"] hi"));
    assert_eq!(stats.continued_lines, 3);
}

#[test]
fn a_backslash_followed_by_a_space_does_not_continue() {
    let (v, stats) = parse_with_stats("[Dialog] \\ \n[Blocker]\n");
    assert_eq!(v.len(), 2);
    assert_eq!(stats.continued_lines, 0);
}

#[test]
fn argument_values_are_json_decoded() {
    let (v, stats) =
        parse_with_stats(r#"[Subtitle(text="\n_ESTABLISHING\tSHOT—done", alignment="center")]"#);
    assert_eq!(v[0].args["text"], "\n_ESTABLISHING\tSHOT\u{2014}done");
    assert_eq!(v[0].args["alignment"], "center");
    assert_eq!(stats.decoded_values, 1);
    assert_eq!(stats.dropped_arg_blocks, 0);
}

#[test]
fn an_invalid_escape_keeps_the_raw_value_and_is_recorded() {
    let (v, stats) = parse_with_stats(r#"[Sticker(id="s1", text="a\ b\nc")]"#);
    assert_eq!(v[0].args["text"], r"a\ b\nc");
    assert_eq!(v[0].args["id"], "s1");
    assert_eq!(stats.dropped_arg_blocks, 1);
    assert_eq!(stats.decoded_values, 0);
    assert_eq!(stats.diagnostics.len(), 1);
    assert_eq!(stats.diagnostics[0].deviation, Deviation::InvalidEscape);
    assert_eq!(stats.diagnostics[0].detail, "text");
    assert_eq!(stats.diagnostics[0].line, 1);
}

#[test]
fn a_folded_typo_is_recorded_as_a_deviation() {
    let (_, stats) = parse_with_stats("[palysound(key=\"$x\")]\n[playsound(key=\"$y\")]\n");
    assert_eq!(stats.diagnostics.len(), 1);
    assert_eq!(stats.diagnostics[0].deviation, Deviation::TypoFolded);
    assert_eq!(stats.diagnostics[0].detail, "palysound");
}

#[test]
fn escaped_quotes_still_decode_and_surrogate_pairs_join() {
    let c = one(r#"[Sticker(text="He said \"no\", twice 🙂")]"#);
    assert_eq!(c.args["text"], "He said \"no\", twice \u{1f642}");
    let (v, stats) = parse_with_stats(r#"[Sticker(text="\ud83d only")]"#);
    assert_eq!(v[0].args["text"], r"\ud83d only");
    assert_eq!(stats.dropped_arg_blocks, 1);
}

#[test]
fn word_count_covers_prose_kinds_only() {
    let v = parse(
        "[name=\"A\"] one two three\nfour five\n[multiline(name=\"B\")] six\n[Subtitle(text=\"seven eight\")]\n[Sticker(text=\"nine\")]\n[Background(image=\"ten eleven\")]\n[animtext(id=\"x\")] twelve",
    );
    assert_eq!(word_count(&v), 9);
}
