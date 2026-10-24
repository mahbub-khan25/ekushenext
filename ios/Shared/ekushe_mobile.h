// The Ekushe engine for the iOS keyboard (crates/ekushe-mobile). Strings returned by any ekm_*
// call are freed with ekm_free.
#pragma once
#include <stdbool.h>
#include <stdint.h>

int32_t ekm_init(const char *dict_path, const char *dir);
void ekm_free(char *s);
char *ekm_key(uint32_t codepoint);
char *ekm_backspace(void);
char *ekm_select(uint32_t index, uint32_t trailing);
char *ekm_commit(uint32_t trailing);
void ekm_reset(void);
void ekm_set_private(bool on);
bool ekm_has_undo(void);
bool ekm_is_banglish(const char *word);
char *ekm_get_config(void);
bool ekm_set_config(const char *json);
void ekm_save(void);
void ekm_forget_learned(void);
char *ekm_release(void);
char *ekm_report_url(const char *os);
