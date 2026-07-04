import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import { gitConfig } from "./shared";
import { ThemeLogo } from "@/components/theme-logo";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: <ThemeLogo />,
    },
    githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
  };
}
