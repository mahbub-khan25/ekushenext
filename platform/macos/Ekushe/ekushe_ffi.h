// C interface of the Rust engine (crates/ekushe-ffi), as used by the macOS input method.
#pragma once
#include <stdbool.h>
#include <stdint.h>

void ek_init(void);
char *ek_key(uint32_t codepoint);
char *ek_backspace(void);
char *ek_select(uint32_t index, uint32_t trailing);
void ek_reset(void);
void ek_set_private(bool is_private);
bool ek_is_composing(void);
/* Backspace right after a word: reopen it as the word being typed (JSON output, its delete_before
   = the word and its space in UTF-16 units). NULL when there is nothing to reopen. Free with
   ekushe_string_free. */
char *ek_reopen_last_word(void);
/* Forget the Backspace-undo (the caret moved, or a key went to the application directly). */
void ek_forget_undo(void);
bool ek_can_undo(void);
uint8_t ek_language(void);
uint8_t ek_toggle_language(void);
uint8_t ek_shift_tap(uint64_t now_ms);
char *ek_flip_last(void);
uint8_t ek_app_focus(const char *app_id);
void ek_reload_config(void);
void ek_use_start_language(void);
void ek_save_learned(void);
uint16_t ek_studio_start(void);
char *ek_studio_url(const char *tab);
char *ek_report_url(const char *os);
void ekushe_string_free(char *s);
char *ek_out_preedit(void);
char *ek_out_committed(void);
uint32_t ek_out_candidate_count(void);
char *ek_out_candidate(uint32_t i);
uint32_t ek_out_selected(void);
