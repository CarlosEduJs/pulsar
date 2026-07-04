"use client";

import { useRef } from "react";
import { motion, useInView } from "motion/react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

const features = [
  {
    title: "Quality & Correctness",
    description:
      "Catches SELECT *, missing LIMIT, unbounded finds, always-true WHERE, and missing awaits before they ship.",
    rules: 5,
    icon: "▲",
  },
  {
    title: "Performance",
    description:
      "Eliminates N+1 queries, loop-based queries, and callback-based queries. The IR graph tracks every pattern.",
    rules: 3,
    icon: "◆",
  },
  {
    title: "Schema-Aware",
    description:
      "Cross-references your TypeScript with your Prisma schema. Detects unknown columns, missing indexes, and missing foreign keys.",
    rules: 3,
    icon: "■",
  },
];

export function Features() {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  return (
    <section ref={ref} className="flex flex-col items-center px-4 py-16 md:py-24">
      <motion.h2
        initial={{ opacity: 0, y: 16 }}
        animate={isInView ? { opacity: 1, y: 0 } : {}}
        transition={{ duration: 0.5, ease: "easeOut" }}
        className="font-heading text-2xl/tight font-bold tracking-tight text-center"
      >
        What Pulsar catches
      </motion.h2>

      <motion.p
        initial={{ opacity: 0, y: 12 }}
        animate={isInView ? { opacity: 1, y: 0 } : {}}
        transition={{ duration: 0.5, delay: 0.1, ease: "easeOut" }}
        className="mt-3 text-sm text-muted-foreground text-center max-w-lg"
      >
        12 built-in rules spanning quality, correctness, performance, security, and schema analysis.
      </motion.p>

      <div className="mt-10 grid gap-4 sm:grid-cols-2 lg:grid-cols-3 max-w-5xl w-full">
        {features.map((feature, index) => (
          <motion.div
            key={feature.title}
            initial={{ opacity: 0, y: 20 }}
            animate={isInView ? { opacity: 1, y: 0 } : {}}
            transition={{
              duration: 0.5,
              delay: 0.2 + index * 0.12,
              ease: "easeOut",
            }}
          >
            <Card size="sm" className="h-full">
              <CardHeader>
                <span className="text-lg text-primary" aria-hidden>
                  {feature.icon}
                </span>
                <CardTitle>{feature.title}</CardTitle>
                <CardDescription className="mt-1 text-xs leading-relaxed">
                  {feature.description}
                </CardDescription>
              </CardHeader>
              <CardContent className="mt-auto">
                <span className="text-xs text-muted-foreground/60">{feature.rules} rules</span>
              </CardContent>
            </Card>
          </motion.div>
        ))}
      </div>
    </section>
  );
}
