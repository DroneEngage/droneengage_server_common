"use strict";

/**
 * droneengage_server_common — shared code for DroneEngage server modules.
 *
 * Exposes:
 *   - create(options)            → config loader instance (see lib/js_serverConfig.js)
 *   - dumperror                  → error-printing helpers (see lib/dumperror.js)
 *   - helpers.*                  → utility helpers (validation, strings, args, colors, ...)
 *   - configHandler              → $$HASH$$ directive processing (see lib/helpers/js_config_handler.js)
 *   - password                   → bcrypt hashing/verification (see lib/helpers/hlp_password.js)
 */

const dumperror = require('./lib/dumperror.js');
const helpers = {
    stripJsonComments: require('./lib/helpers/js_3rd_StripJsonComments.js'),
    args:              require('./lib/helpers/hlp_args.js'),
    strings:           require('./lib/helpers/hlp_strings.js'),
    validation:        require('./lib/helpers/hlp_validation.js'),
    colors:            require('./lib/helpers/js_colors.js'),
    styleHelper:       require('./lib/helpers/js_styleHelper.js'),
};
const configHandler = require('./lib/helpers/js_config_handler.js');
const password = require('./lib/helpers/hlp_password.js');
const { create } = require('./lib/js_serverConfig.js');

module.exports = {
    create,
    dumperror,
    helpers,
    configHandler,
    password,
};
