// C interface of the Rust engine (crates/ekushe-ffi), shared by the macOS and Linux front ends.
#pragma once
#include <stdbool.h>
#include <stdint.h>

void ek_init(void);
char *ek_key(uint32_t codepoint);
char *ek_backspace(void);
char *ek_select(uint32_t index, uint32_t trailing);
void ek_reset(void);
bool ek_is_composing(void);
bool ek_can_undo(void);
uint8_t ek_language(void);
uint8_t ek_toggle_language(void);
uint8_t ek_shift_tap(uint64_t now_ms);
char *ek_flip_last(void);
uint8_t ek_app_focus(const char *app_id);
void ek_reload_config(void);
void ek_save_learned(void);
uint16_t ek_studio_start(void);
char *ek_studio_url(const char *tab);
void ekushe_string_free(char *s);
char *ek_out_preedit(void);
char *ek_out_committed(void);
uint32_t ek_out_candidate_count(void);
char *ek_out_candidate(uint32_t i);
uint32_t ek_out_selected(void);
