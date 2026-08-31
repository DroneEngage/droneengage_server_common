"use strict";

/**
 * Generic server configuration loader for DroneEngage server modules.
 *
 * Reconciles the per-server js_serverConfig.js implementations into a single
 * factory. Each server creates its own instance via create(options), passing
 * its config directory and optional hooks. The loader provides:
 *
 *   - JSON-with-comments parsing (via js_3rd_StripJsonComments)
 *   - Optional local override: <configFile>.local is merged on top
 *     (shallow merge; git-ignored; supports C-style comments)
 *   - Optional $$HASH$$('...') directive processing (bcrypt) with persistence
 *   - Optional environment-variable overrides
 *
 * Usage (thin wrapper in a server's src/js_serverConfig.js):
 *
 *   const common = require('droneengage_server_common');
 *   const path = require('path');
 *   module.exports = common.create({
 *       configDir: path.join(__dirname, '..'),
 *       enableHashHandling: true,
 *       envOverrides: {
 *           'MY_ENV_VAR': 'config_key',
 *           'MY_ENV_TRANSFORM': (cfg, value) => { cfg.flag = (value === '1'); }
 *       }
 *   });
 */

const fs = require('fs');
const path = require('path');
const stripJsonComments = require('./helpers/js_3rd_StripJsonComments.js');
const configHandler = require('./helpers/js_config_handler.js');
const dumperror = require('./dumperror.js');

const DEFAULT_CONFIG_FILENAME = 'server.config';

/**
 * Create a config-loader instance.
 *
 * @param {object} options
 * @param {string} options.configDir       - absolute dir containing the config file
 * @param {string} [options.configFileName] - override default "server.config"
 * @param {boolean} [options.enableHashHandling] - process $$HASH$$ directives (default: true)
 * @param {object} [options.envOverrides]  - map of env-var-name → config-key-string | transform(config, value)
 * @returns {{ getFileName: Function, init: Function, m_configuration: any }}
 */
function create(options) {
    const configDir = options.configDir;
    if (!configDir) {
        throw new Error('droneengage_server_common: options.configDir is required');
    }
    const configFileNameDefault = options.configFileName || DEFAULT_CONFIG_FILENAME;
    const enableHash = options.enableHashHandling !== false;
    const envOverrides = options.envOverrides || {};

    let configFileName = configFileNameDefault;
    let configuration = null;

    function getFileName() {
        return configFileName;
    }

    function init(configFileNameParam) {
        if (configFileNameParam) {
            configFileName = configFileNameParam;
        }

        const configFilePath = path.join(configDir, configFileName);

        // --- Load base config ---
        let fileContent;
        try {
            fileContent = fs.readFileSync(configFilePath, 'utf8');
        } catch (err) {
            console.error('FATAL: could not find ' + configFileName + ' at ' + configFilePath);
            dumperror.fn_dumperror(err);
            process.exit(1);
        }

        try {
            configuration = JSON.parse(stripJsonComments(fileContent));
        } catch (err) {
            console.error('FATAL: Bad File Format ' + configFileName);
            dumperror.fn_dumperror(err);
            process.exit(1);
        }

        // --- $$HASH$$ directive handling (base file) ---
        if (enableHash) {
            configHandler.handleConfig(configuration, configFilePath);
        }

        // --- Optional local override: <configFile>.local ---
        // Merged on top (shallow). Git-ignored. Supports C-style comments.
        const localConfigPath = configFilePath + '.local';
        if (fs.existsSync(localConfigPath)) {
            try {
                const localContent = fs.readFileSync(localConfigPath, 'utf8');
                const localConfig = JSON.parse(stripJsonComments(localContent));
                Object.assign(configuration, localConfig);
                // Hash any $$HASH$$ directives coming from the local file
                // in memory only — never persist back to the local file.
                if (enableHash) {
                    configHandler.processConfigInMemory(configuration);
                }
                console.log('[Config] Local override applied: ' + localConfigPath);
            } catch (localErr) {
                console.warn('[Config] WARNING: could not parse local override ' +
                    localConfigPath + ': ' + localErr.message);
            }
        }

        // --- Environment-variable overrides ---
        for (const envName of Object.keys(envOverrides)) {
            const envValue = process.env[envName];
            if (envValue === undefined) continue;
            const mapping = envOverrides[envName];
            if (typeof mapping === 'function') {
                mapping(configuration, envValue);
            } else {
                configuration[mapping] = envValue;
            }
        }
    }

    return {
        getFileName,
        init,
        get m_configuration() {
            return configuration;
        }
    };
}

module.exports = { create };
