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

  if (!mounted) {
    return (
      <div className="flex items-center gap-2">
        <img src="/logo.svg" alt={appName} className="h-6 w-6" />
        <h1 className="font-semibold text-sm">{appName}</h1>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2">
      {resolvedTheme === "dark" || theme === "dark" ? (
        <img src="/logo.svg" alt={appName} className="h-6 w-6" />
      ) : (
        <img src="/logo-for-white.svg" alt={appName} className="h-6 w-6" />
      )}
      <h1 className="font-semibold text-sm">{appName}</h1>
    </div>
  );
}
