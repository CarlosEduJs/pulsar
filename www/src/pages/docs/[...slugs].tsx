import { getDocsStaticParams, getPageImage, getPageMarkdownUrl, source } from "@/lib/source";
import { PageProps } from "waku/router";
import { createRelativeLink } from "fumadocs-ui/mdx";
import {
  DocsBody,
  DocsDescription,
  DocsPage,
  DocsTitle,
  MarkdownCopyButton,
  ViewOptionsPopover,
} from "fumadocs-ui/layouts/notebook/page";
import { unstable_notFound } from "waku/router/server";
import { gitConfig } from "@/lib/shared";
import { getMDXComponents } from "@/components/mdx";

export default function Page({ slugs }: PageProps<"/docs/[...slugs]">) {
  const page = source.getPage(slugs);
  if (!page) unstable_notFound();

  const MDX = page.data.body;
  const markdownUrl = getPageMarkdownUrl(page).url;
  return (
    <>
      <title>{page.data.title} — Pulsar</title>
      <meta name="description" content={page.data.description} />
      <meta property="og:image" content={getPageImage(slugs).url} />
      <meta property="og:title" content={`${page.data.title} — Pulsar`} />
      <meta property="og:description" content={page.data.description} />
      <DocsPage toc={page.data.toc}>
        <DocsTitle>{page.data.title}</DocsTitle>
        <DocsDescription className="mb-0">{page.data.description}</DocsDescription>
        <div className="flex flex-row gap-2 items-center border-b pt-2 pb-6">
          <MarkdownCopyButton markdownUrl={markdownUrl} />
          <ViewOptionsPopover
            markdownUrl={markdownUrl}
            githubUrl={`https://github.com/${gitConfig.user}/${gitConfig.repo}/blob/${gitConfig.branch}/content/docs/${page.path}`}
          />
        </div>
        <DocsBody>
          <MDX
            components={getMDXComponents({
              // this allows you to link to other pages with relative file paths
              a: createRelativeLink(source, page),
            })}
          />
        </DocsBody>
      </DocsPage>
    </>
  );
}

export async function getConfig() {
  return {
    render: "static" as const,
    staticPaths: getDocsStaticParams(),
  } as const;
}
