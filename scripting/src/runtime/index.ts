/// <reference no-default-lib="true" />

/**
 * This is the core runtime script this is stored as a JS snapshot
 * and all the globals created by this script are exposed when
 * running scripts at runtime
 *
 * This contains helpers, wrapper functions and glue for interacting
 * with the Rust side of the runtime
 */

import * as kv from "./kv";
import * as http from "./http";
import * as vtftk from "./vtftk";
import * as twitch from "./twitch";
import * as random from "./random";
import * as logging from "./logging";
import * as internal from "./internal";
import * as integrations from "./integrations";
import "./eventContext";
import "./commandContext";
import "./arrayExt";

interface API {
  twitch: typeof twitch;
  kv: typeof kv;
  http: typeof http;
  logging: typeof logging;
  vtftk: typeof vtftk;
  integrations: typeof integrations;
  random: typeof random;
  internal: typeof internal;
}

const _api: Readonly<API> = Object.freeze({
  twitch,
  kv,
  http,
  logging,
  vtftk,
  integrations,
  random,
  internal,
});

const _console = Object.freeze({
  log: logging.info,
  info: logging.info,
  error: logging.error,
  debug: logging.debug,
});

// Define API globals
Object.defineProperty(globalThis, "api", {
  value: _api,
  writable: false,
  configurable: false,
});

// Define console globals
Object.defineProperty(globalThis, "console", {
  value: _console,
  writable: false,
  configurable: false,
});

declare global {
  export const api: API;

  export const console: typeof _console;
}
