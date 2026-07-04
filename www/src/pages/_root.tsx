import type { ReactNode } from "react";
import { Provider } from "@/components/provider";
import "@/styles/globals.css";

export default async function RootElement({ children }: { children: ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>Pulsar</title>
        <meta
          name="description"
          content="A static analyzer for TypeScript ORM code. Detects quality, performance, and consistency issues before they reach production."
        />
        <link rel="icon" href="/favicon.ico" sizes="any" />
        <meta property="og:type" content="website" />
        <meta property="og:site_name" content="Pulsar" />
        <meta name="twitter:card" content="summary_large_image" />
      </head>
      <body data-version="1.0" className="flex flex-col min-h-screen">
        <Provider>{children}</Provider>
      </body>
    </html>
  );
}

export async function getConfig() {
  return {
    render: "static",
  } as const;
}
