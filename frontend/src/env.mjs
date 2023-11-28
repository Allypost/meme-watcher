import { createEnv } from "@t3-oss/env-nextjs";
import { z } from "zod";

export const env = createEnv({
  /**
   * Specify your server-side environment variables schema here. This way you can ensure the app
   * isn't built with invalid env vars.
   */
  server: {
    NODE_ENV: z.enum(["development", "test", "production"]),
    APP_BACKEND_URL: z
      .string()
      .url()
      .refine(
        (str) => isParsableUrl(str),
        "The provided backend url is not a valid url",
      ),
  },

  /**
   * Specify your client-side environment variables schema here. This way you can ensure the app
   * isn't built with invalid env vars. To expose them to the client, prefix them with
   * `NEXT_PUBLIC_`.
   */
  client: {
    // NEXT_PUBLIC_CLIENTVAR: z.string(),
    NEXT_PUBLIC_APP_URL: z
      .string()
      .url()
      .refine(
        (str) => !str.includes("YOUR-APP-URL-HERE.example"),
        "You forgot to edit the default app url",
      )
      .refine(
        (str) => isParsableUrl(str),
        "The provided public app url is not a valid url",
      ),
  },

  /**
   * You can't destruct `process.env` as a regular object in the Next.js edge runtimes (e.g.
   * middlewares) or client-side so we need to destruct manually.
   */
  runtimeEnv: {
    NODE_ENV: process.env.NODE_ENV,
    APP_BACKEND_URL: process.env.APP_BACKEND_URL,
    NEXT_PUBLIC_APP_URL: process.env.NEXT_PUBLIC_APP_URL,
    // NEXT_PUBLIC_CLIENTVAR: process.env.NEXT_PUBLIC_CLIENTVAR,
  },
  /**
   * Run `build` or `dev` with `SKIP_ENV_VALIDATION` to skip env validation.
   * This is especially useful for Docker builds.
   */
  skipValidation: !!process.env.SKIP_ENV_VALIDATION,
  /**
   * Makes it so that empty strings are treated as undefined.
   * `SOME_VAR: z.string()` and `SOME_VAR=''` will throw an error.
   */
  emptyStringAsUndefined: true,
});

/**
 * @param {string | URL} str
 */
function isParsableUrl(str) {
  /** @type {URL} */
  let url;
  try {
    url = new URL(str);
  } catch {
    return false;
  }

  return url.protocol === "http:" || url.protocol === "https:";
}
