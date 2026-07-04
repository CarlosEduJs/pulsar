import { getDocsStaticParams, getLLMText, source } from "@/lib/source";
import { ApiContext } from "waku/router";
import { unstable_notFound } from "waku/router/server";

export async function GET(
  _: Request,
  { params }: ApiContext<"/llms.mdx/docs/[...slugs]/content.md">,
) {
  const slugs = params.slugs;
  const page = source.getPage(slugs);
  if (!page) unstable_notFound();

  return new Response(await getLLMText(page), {
    headers: {
      "Content-Type": "text/markdown",
    },
  });
}

export async function getConfig() {
  return {
    render: "static" as const,
    staticPaths: getDocsStaticParams(),
  } as const;
}
