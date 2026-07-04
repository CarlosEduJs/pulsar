"use client";

import { useTheme } from "next-themes";
import { appName } from "@/lib/shared";
import * as React from "react";

export function ThemeLogo() {
  const { theme, resolvedTheme } = useTheme();
  const [mounted, setMounted] = React.useState(false);

  React.useEffect(() => {
    setMounted(true);
  }, []);

  const logoSrc =
    !mounted || resolvedTheme === "dark" || theme === "dark" ? "/logo.svg" : "/logo-for-white.svg";

  return (
    <div className="flex items-center gap-2">
      <img src={logoSrc} alt={appName} className="h-6 w-6" />
      <h1 className="font-semibold text-sm">{appName}</h1>
    </div>
  );
}
