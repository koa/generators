/*
 * Copyright (C) 2020 Erik Fleckstein <erik@tinkerforge.com>
 *
 * Redistribution and use in source and binary forms of this file,
 * with or without modification, are permitted. See the Creative
 * Commons Zero (CC0 1.0) License for more details.
 */

#include "hal_common.h"

#include <string.h>
#include <stdio.h>

#include "tfp.h"
#include "bricklet_unknown.h"
#include "base58.h"
#include "macros.h"
#include "errors.h"

int tf_hal_common_init(TF_HalContext *hal) {
    TF_HalCommon *hal_common = tf_hal_get_common(hal);
    memset(hal_common, 0, sizeof(TF_HalCommon));
    return TF_E_OK;
}

int tf_hal_finish_init(TF_HalContext *hal, uint8_t port_count, uint32_t port_discovery_timeout_us) {
    TF_HalCommon *hal_common = tf_hal_get_common(hal);
    hal_common->timeout = port_discovery_timeout_us;
    hal_common->port_count = port_count;

    TF_Unknown unknown;
    hal_common->used = 1;

    for(int i = 0; i < port_count; ++i) {
        if (hal_common->used >= sizeof(hal_common->uids) / sizeof(hal_common->uids[0]))
            return TF_E_TOO_MANY_DEVICES;
        tf_unknown_create(&unknown, "1", hal, (uint8_t)i, 0);

        int rc = tf_unknown_comcu_enumerate(&unknown);
        if (rc == TF_E_OK) {
            tf_unknown_callback_tick(&unknown, port_discovery_timeout_us);
        }

        tf_unknown_destroy(&unknown);
    }

    if (hal_common->used > sizeof(hal_common->uids) / sizeof(hal_common->uids[0]))
        return TF_E_TOO_MANY_DEVICES;

    hal_common->timeout = 2500000;

    return TF_E_OK;
}

static void enum_handler(TF_HalContext* hal,
                  uint8_t port_id,
                  char uid[8],
                  char connected_uid[8],
                  char position,
                  uint8_t hw_version[3],
                  uint8_t fw_version[3],
                  uint16_t dev_id,
                  uint8_t enumeration_type) {
    (void) connected_uid;
    (void) position;
    (void) hw_version;
    (void) fw_version;
    (void) enumeration_type;
    TF_HalCommon *hal_common = tf_hal_get_common(hal);
    if (hal_common->used >= sizeof(hal_common->uids) / sizeof(hal_common->uids[0]))
        return;

    uint32_t numeric_uid;
    if(tf_base58_decode(uid, &numeric_uid) != TF_E_OK)
        return;

    for(size_t i = 0; i < hal_common->used; ++i)
        if(hal_common->uids[i] == numeric_uid) {
            hal_common->port_ids[i] = port_id;
            hal_common->dids[i] = dev_id;
            if(hal_common->tfps[i] != NULL)
                hal_common->tfps[i]->spitfp.port_id = port_id;
            return;
        }

    tf_hal_log_info("Found device %s of type %d at port %c", uid, dev_id, tf_hal_get_port_name(hal, port_id));

    hal_common->port_ids[hal_common->used] = port_id;
    hal_common->uids[hal_common->used] = numeric_uid;
    hal_common->dids[hal_common->used] = dev_id;
    ++hal_common->used;
}

bool tf_hal_enumerate_handler(TF_HalContext *hal, uint8_t port_id, TF_Packetbuffer *payload) {
    int i;
    char uid[8]; tf_packetbuffer_pop_n(payload, (uint8_t*)uid, 8);
    char connected_uid[8]; tf_packetbuffer_pop_n(payload, (uint8_t*)connected_uid, 8);
    char position = tf_packetbuffer_read_char(payload);
    uint8_t hardware_version[3]; for (i = 0; i < 3; ++i) hardware_version[i] = tf_packetbuffer_read_uint8_t(payload);
    uint8_t firmware_version[3]; for (i = 0; i < 3; ++i) firmware_version[i] = tf_packetbuffer_read_uint8_t(payload);
    uint16_t device_identifier = tf_packetbuffer_read_uint16_t(payload);
    uint8_t enumeration_type = tf_packetbuffer_read_uint8_t(payload);

    //No device before us has patched in the position and connected_uid.
    if(connected_uid[0] == 0)
        position = tf_hal_get_port_name(hal, port_id);

    enum_handler(hal, port_id, uid, connected_uid, position, hardware_version, firmware_version, device_identifier, enumeration_type);

    return true;
}

TF_ATTRIBUTE_FMT_PRINTF(1, 2)
void tf_hal_log_formatted_message(const char *fmt, ...){
    char buf[128];
    memset(buf, 0, sizeof(buf)/sizeof(buf[0]));

    va_list args;
    va_start (args, fmt);
    vsnprintf(buf, sizeof(buf)/sizeof(buf[0]), fmt, args);
    va_end (args);

    tf_hal_log_message(buf);
}

void tf_hal_set_timeout(TF_HalContext *hal, uint32_t timeout_us) {
    tf_hal_get_common(hal)->timeout = timeout_us;
}

uint32_t tf_hal_get_timeout(TF_HalContext *hal) {
    return tf_hal_get_common(hal)->timeout;
}

int tf_hal_get_port_id(TF_HalContext *hal, uint32_t uid, uint8_t *port_id, int *inventory_index) {
    TF_HalCommon *hal_common = tf_hal_get_common(hal);

    for(int i = 0; i < (int)hal_common->used; ++i) {
        if(hal_common->uids[i] == uid) {
            *port_id = hal_common->port_ids[i];
            *inventory_index = i;
            return TF_E_OK;
        }
    }

    return TF_E_DEVICE_NOT_FOUND;
}

bool tf_hal_get_device_info(TF_HalContext *hal, size_t index, char ret_uid[7], char *ret_port_name, uint16_t *ret_device_id) {
    TF_HalCommon *hal_common = tf_hal_get_common(hal);

    // Increment index to skip over the 0th inventory entry
    // (the unknown bricklet used for device discovery).
    ++index;

    if (index >= hal_common->used) {
        return false;
    }

    tf_base58_encode(hal_common->uids[index], ret_uid);
    *ret_port_name = tf_hal_get_port_name(hal, hal_common->port_ids[index]);
    *ret_device_id = hal_common->dids[index];
    return true;
}