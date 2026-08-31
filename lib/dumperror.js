"use strict";

/**
 * Error-printing helpers.
 *
 * Reconciles the divergent dumperror implementations across the DroneEngage
 * servers. All historical function names are exported for backward
 * compatibility:
 *
 *   dumperror(err)      — prints the error object directly (auth name)
 *   fn_dumperror(err)   — same behaviour (comm server name)
 *   dumperror2(err)     — prints message + stacktrace for Error objects
 *   fn_dumpdebug(err)   — enhanced: handles null, string, and object errors
 */

exports.dumperror = function dumperror(err) {
    console.log(err);
};

// Alias for the comm-server naming convention.
exports.fn_dumperror = function fn_dumperror(err) {
    console.log(err);
};

exports.dumperror2 = function dumperror2(err) {
    if (typeof err === 'object') {
        if (err.message) {
            console.log('\nMessage: ' + err.message);
        }
        if (err.stack) {
            console.log('\nStacktrace:');
            console.log('====================');
            console.log(err.stack);
        }
    } else {
        console.log('dumpError :: argument is not an object');
    }
};

exports.fn_dumpdebug = function fn_dumpdebug(err) {
    if (err == null) return;
    if (typeof err === 'object') {
        if (err.message) {
            console.log('\nMessage: ' + err.message);
        }
        if (err.stack) {
            console.log('\nStacktrace:');
            console.log('====================');
            console.log(err.stack);
        }
    } else if (typeof err === 'string') {
        console.log(err);
    } else {
        console.log('dumpError :: argument is not an object');
    }
};
