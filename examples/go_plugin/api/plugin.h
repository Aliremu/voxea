// Generated by `wit-bindgen` 0.30.0. DO NOT EDIT!
#ifndef __BINDINGS_PLUGIN_H
#define __BINDINGS_PLUGIN_H
#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>

typedef struct plugin_string_t {
  uint8_t*ptr;
  size_t len;
} plugin_string_t;

// Imported Functions from `sdk:component/logger`
extern void sdk_component_logger_log(plugin_string_t *text);

// Imported Functions from `sdk:component/registry`
extern double sdk_component_registry_get_signal(uint64_t idx);
extern double sdk_component_registry_set_signal(uint64_t idx, double val);

// Exported Functions from `sdk:component/plugin-api`
int32_t exports_sdk_component_plugin_api_enable(void);
int32_t exports_sdk_component_plugin_api_disable(void);
void exports_sdk_component_plugin_api_process_signal(uint64_t ptr);

// Helper Functions

// Transfers ownership of `s` into the string `ret`
void plugin_string_set(plugin_string_t *ret, const char*s);

// Creates a copy of the input nul-terminate string `s` and
// stores it into the component model string `ret`.
void plugin_string_dup(plugin_string_t *ret, const char*s);

// Deallocates the string pointed to by `ret`, deallocating
// the memory behind the string.
void plugin_string_free(plugin_string_t *ret);

#ifdef __cplusplus
}
#endif
#endif
