import { appName } from "@/lib/shared";
import { ImageResponse } from "@takumi-rs/image-response";
import { OgTemplate } from "@/lib/og-template";
import type { ApiContext } from "waku/router";

export async function GET(_: Request, { params }: ApiContext<"/og/home/image.webp">) {
  return new ImageResponse(
    <OgTemplate
      title="Pulsar"
      description="A static analyzer for TypeScript ORM code"
      site={appName}
    />,
    {
      width: 1200,
      height: 630,
      format: "webp",
    },
  );
}

export async function getConfig() {
  return {
    render: "static" as const,
  } as const;
}
