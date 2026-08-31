# droneengage_server_common

Shared configuration loader, helpers, and utilities for DroneEngage server
modules (authenticator, comm server, storage server).

## Install

Add to a server's `package.json`:

```json
"dependencies": {
  "droneengage_server_common": "git+ssh://git@github.com:DroneEngage/droneengage_server_common.git#v1.0.0"
}
```

Then `npm install`.

## Usage

### Config loader

Create a thin wrapper in your server (e.g. `src/js_serverConfig.js`):

```js
const common = require('droneengage_server_common');
const path = require('path');

module.exports = common.create({
    configDir: path.join(__dirname, '..'),
    enableHashHandling: true,   // process $$HASH$$('...') directives
    envOverrides: {
        'DE_MY_ENV': 'config_key',                    // simple: assign env value to config key
        'DE_MY_FLAG': (cfg, val) => { cfg.flag = (val === '1'); }  // transform
    }
});
```

Then use as before:

```js
const m_serverconfig = require('./js_serverConfig.js');
m_serverconfig.init();              // or init('server.config.production')
const cfg = m_serverconfig.m_configuration;
```

### Local config override

Place a `server.config.local` file next to `server.config`. Its top-level
keys are merged on top (shallow merge). The file is git-ignored and supports
C-style comments, exactly like `server.config`.

### Helpers

```js
const { helpers, dumperror, configHandler, password } = require('droneengage_server_common');

helpers.stripJsonComments(jsonText);
helpers.validation.isEmail(str);
helpers.strings.generateRandomString(32);
helpers.colors.Colors;
helpers.args.getArgs();
dumperror.fn_dumperror(err);
configHandler.handleConfig(configObj, configPath);
password.hash(plaintext);
```

## What's included

| Module | Description |
|---|---|
| `lib/js_serverConfig.js` | Generic config loader factory (local override + `$$HASH$$` + env hooks) |
| `lib/dumperror.js` | Error-printing helpers (all historical API names) |
| `lib/helpers/js_3rd_StripJsonComments.js` | Strip C-style comments from JSON |
| `lib/helpers/hlp_args.js` | CLI argument parser |
| `lib/helpers/hlp_strings.js` | String utilities + re-exported validation |
| `lib/helpers/hlp_validation.js` | Input validation (alphanumeric, email, admin creds, ...) |
| `lib/helpers/js_colors.js` | ANSI color codes |
| `lib/helpers/js_styleHelper.js` | CSS traffic-level class helper |
| `lib/helpers/js_config_handler.js` | `$$HASH$$('...')` → bcrypt directive processor |
| `lib/helpers/hlp_password.js` | bcrypt hash/verify with legacy plaintext fallback |
