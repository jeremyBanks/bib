/// <reference no-default-lib="true" />
/// <reference lib="dom" />
/// <reference lib="dom.iterable" />
/// <reference lib="dom.asynciterable" />
/// <reference lib="deno.ns" />

import { start } from "$fresh/server.ts";
import manifest from "./fresh.gen.ts";

import twindPlugin from "$fresh/plugins/twind.ts";
import fontsPlugin from "./plugins/fonts.ts";
import twindConfig from "./twind.config.ts";

await start(manifest, {
  port: Deno.env.get("DENO_DEPLOYMENT_ID") ? 8000 : 80,
  plugins: [twindPlugin(twindConfig), fontsPlugin],
});
