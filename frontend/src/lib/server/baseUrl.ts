import { env } from "~/env.mjs";

export function getBaseUrl() {
  if (typeof window !== "undefined") {
    return "";
  }

  if (env.APP_BACKEND_URL) {
    return env.APP_BACKEND_URL;
  }

  if (process.env.VERCEL_URL) {
    return `https://${process.env.VERCEL_URL}`;
  }

  return `http://localhost:${process.env.PORT ?? 3000}`;
}

export const BASE_URL = getBaseUrl();
