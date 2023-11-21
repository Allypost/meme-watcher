import "~/assets/styles/tailwind.css";
import "~/assets/styles/globals.css";

import { commitMono } from "~/assets/font";
import { BASE_METADATA, BASE_VIEWPORT } from "~/lib/page/metadata";

export const metadata = BASE_METADATA;
export const viewport = BASE_VIEWPORT;

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body
        className={`font-mono text-white antialiased ${commitMono.variable}`}
      >
        {children}
      </body>
    </html>
  );
}
