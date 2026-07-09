#ifndef SMART_SOCKET_H
#define SMART_SOCKET_H

/*
 * Smart socket C ABI.
 *
 * A smart socket has an on/off switch and a rated power consumption (watts).
 * When the socket is off, smart_socket_power() reports 0; when it is on,
 * smart_socket_power() reports the rated power.
 *
 * Lifetime: handles returned by smart_socket_new() must be released with
 * smart_socket_free(). Passing a freed/null handle to any function is safe
 * (treated as a no-op / default return value) EXCEPT freeing the same handle
 * twice, which is undefined behaviour.
 */

#include <stdbool.h> /* bool */
#include <stddef.h>  /* NULL */

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handle to a smart socket. */
typedef struct SmartSocket SmartSocket;

/* Creates a new smart socket.
 *
 * is_on:        initial on/off state.
 * power_watts:  rated power consumption (must be >= 0).
 *
 * Returns a handle, or NULL if power_watts is negative or NaN.
 * The caller owns the handle and must free it with smart_socket_free(). */
SmartSocket *smart_socket_new(bool is_on, float power_watts);

/* Frees a smart socket. No-op if ptr is NULL.
 * Double-free is undefined behaviour. */
void smart_socket_free(SmartSocket *ptr);

/* Turns the socket on. No-op if ptr is NULL. */
void smart_socket_turn_on(SmartSocket *ptr);

/* Turns the socket off. No-op if ptr is NULL. */
void smart_socket_turn_off(SmartSocket *ptr);

/* Returns true if the socket is on, false if off or ptr is NULL. */
bool smart_socket_is_on(const SmartSocket *ptr);

/* Returns the current power consumption in watts (0 when off, or if NULL). */
float smart_socket_power(const SmartSocket *ptr);

/* Returns the rated power in watts (0 if ptr is NULL). */
float smart_socket_rated_power(const SmartSocket *ptr);

/* Returns the library version as a static, NUL-terminated C string.
 * The returned pointer is valid for the program lifetime and must NOT be freed. */
const char *smart_socket_version(void);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* SMART_SOCKET_H */
