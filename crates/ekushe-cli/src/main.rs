//! `ekushe-cli`: Phase 0 Command-Line Interface for Ekushe Core Engine.
//!
//! Subcommands:
//!   `ekushe-cli phonetic "<roman text>"`      -> Transliterate Roman to Bangla
//!   `ekushe-cli suggest  "<roman token>"`     -> Show candidates + vowel sign (কার) & conjunct (যুক্তবর্ণ) hints
//!   `ekushe-cli signs    "[search query]"`    -> Search or list all Bangla vowel signs (কার), ফলা, & যুক্তবর্ণ
//!   `ekushe-cli spell    "<bangla text>"`     -> Check Bangla spelling and show ranked corrections
//!   `ekushe-cli bijoy2uni "<bijoy text>"`     -> Convert Bijoy (ANSI / SutonnyMJ) to Unicode
//!   `ekushe-cli uni2bijoy "<unicode text>"`   -> Convert Unicode Bangla to Bijoy

use ekushe_core::{
    conjuncts::{decompose_bangla, search_signs_and_conjuncts},
    convert::{bijoy_to_unicode, unicode_to_bijoy},
    phonetic::PhoneticEngine,
    suggest::SuggestionEngine,
};
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        return;
    }

    let cmd = args[0].as_str();
    let rest = args[1..].join(" ");

    match cmd {
        "phonetic" | "p" => {
            let eng = PhoneticEngine::new();
            println!("{}", eng.transliterate(&rest));
        }
        "suggest" | "s" => {
            let eng = PhoneticEngine::new();
            let sugg = SuggestionEngine::new();
            let (candidates, hints) = sugg.suggest(&rest, &eng, false, true, 9);
            println!("Input Roman : {rest}");
            println!("Candidates  :");
            for (i, c) in candidates.iter().enumerate() {
                let tag = c.annotation.as_deref().unwrap_or("");
                println!("  {}. {} [{:?}] {}", i + 1, c.text, c.kind, tag);
            }
            if !hints.is_empty() {
                println!("\nNext-Key Sign (কার) & Connected-Letter (যুক্তবর্ণ) Hints:");
                for h in hints.iter().take(18) {
                    println!(
                        "  {:6} -> {:4}  ({}) [{}]",
                        h.key_hint,
                        h.result_bangla,
                        h.label,
                        decompose_bangla(&h.result_bangla)
                    );
                }
            }
        }
        "signs" | "conjuncts" => {
            let matches = search_signs_and_conjuncts(&rest);
            println!("Found {} Bangla Signs / Connected Letters:", matches.len());
            for m in matches {
                let alts = if m.alt_keys.is_empty() {
                    String::new()
                } else {
                    format!(" (or: {})", m.alt_keys.join(", "))
                };
                println!(
                    "  {:4} | key: {:4}{} | {:18} | {} | e.g. {}",
                    m.bangla, m.primary_key, alts, m.decomposition, m.name, m.example
                );
            }
        }
        "spell" => {
            let sugg = SuggestionEngine::new();
            let issues = sugg.spell_checker().check_text(&rest);
            if issues.is_empty() {
                println!("✓ No spelling issues found.");
            } else {
                for issue in issues {
                    println!(
                        "✗ '{}' (at byte {}) -> suggestions: {}",
                        issue.word,
                        issue.byte_offset,
                        issue.suggestions.join(", ")
                    );
                }
            }
        }
        "bijoy2uni" => {
            println!("{}", bijoy_to_unicode(&rest));
        }
        "uni2bijoy" => {
            println!("{}", unicode_to_bijoy(&rest));
        }
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("Ekushe Next CLI (একুশে নেক্সট) — Phase 0 Core Engine Verifier");
    println!("Usage:");
    println!("  ekushe-cli phonetic  \"<roman text>\"");
    println!("  ekushe-cli suggest   \"<roman token>\"");
    println!("  ekushe-cli signs     \"[query]\"");
    println!("  ekushe-cli spell     \"<bangla text>\"");
    println!("  ekushe-cli bijoy2uni \"<bijoy text>\"");
    println!("  ekushe-cli uni2bijoy \"<unicode text>\"");
}
