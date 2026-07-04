import { appName } from "@/lib/shared";
import { getDocsStaticParams, source } from "@/lib/source";
import { ImageResponse } from "@takumi-rs/image-response";
import { OgTemplate } from "@/lib/og-template";
import { ApiContext } from "waku/router";

export async function GET(_: Request, { params }: ApiContext<"/og/docs/[...slugs]/image.webp">) {
  const page = source.getPage(params.slugs);

  if (!page) return new Response(undefined, { status: 404 });

  return new ImageResponse(
    <OgTemplate title={page.data.title} description={page.data.description} site={appName} />,
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
    staticPaths: getDocsStaticParams(),
  } as const;
}
