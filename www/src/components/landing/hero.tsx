"use client";

import { motion } from "motion/react";
import { Link } from "waku";
import { Button } from "@/components/ui/button";
import { DynamicCodeBlock } from "fumadocs-ui/components/dynamic-codeblock";

export function Hero() {
  return (
    <section className="relative flex flex-col items-center justify-center px-4 pt-24 pb-16 text-center md:pt-32 md:pb-24 bg-background">
      <motion.div
        initial={{ opacity: 0, y: 24 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, ease: "easeOut" }}
        className="max-w-3xl"
      >
        <h1 className="font-heading text-4xl/tight font-bold tracking-tight md:text-5xl/tight">
          Catch ORM bugs before <span className="text-primary">they reach production</span>
        </h1>

        <p className="mt-6 text-base/relaxed text-muted-foreground md:text-lg/relaxed max-w-2xl mx-auto">
          Pulsar is a Rust-powered static analyzer that understands your TypeScript, ORM calls, and
          database schema — catching bugs, performance issues, and mismatches that generic linters
          miss.
        </p>

        <div className="mt-8 flex flex-wrap items-center justify-center gap-4">
          <a
            href="https://github.com/CarlosEduJs/pulsar/releases"
            target="_blank"
            rel="noopener noreferrer"
          >
            <Button size="lg">Download Now</Button>
          </a>

          <DynamicCodeBlock
            lang="bash"
            code="brew install carlosedujs/tap/pulsar-cli"
            codeblock={{ className: "border-none ring-0 shadow-none py-0" }}
          />
          <Link to="/docs/guide">
            <Button size="lg" variant="outline">
              Documentation
            </Button>
          </Link>
        </div>
      </motion.div>

      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.2, ease: "easeOut" }}
        className="mt-12 flex items-center gap-2 text-xs text-muted-foreground"
      >
        v0.6.1 &middot; MIT &middot; Written in Rust
      </motion.div>
    </section>
  );
}
