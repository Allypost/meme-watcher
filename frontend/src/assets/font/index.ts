import localFont from "next/font/local";

export const commitMono = localFont({
  variable: "--font-commit-mono",
  src: "./CommitMono/CommitMono VariableFont.woff2",
  fallback: ["'Fira Code'", "'JetBrains Mono'"],
  preload: true,
  display: "swap",
});
