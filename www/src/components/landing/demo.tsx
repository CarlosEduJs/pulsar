"use client"

import { useRef } from 'react';
import { motion, useInView } from 'motion/react';
import { Link } from 'waku';
import { Button } from '@/components/ui/button';
import { DynamicCodeBlock } from 'fumadocs-ui/components/dynamic-codeblock';

const codeBefore = `// Pulsar catches 3 issues
const users = db.select().from(users);
const user = db.query.users.findFirst();
const posts = db.select({ id: posts.id }).from(posts);`;

const codeAfter = `// Clean and safe
const users = await db.select({ id: users.id, name: users.name })
  .from(users).limit(100);

const user = await db.query.users.findFirst({
  where: eq(users.id, 1),
});

const posts = await db.select({ id: posts.id })
  .from(posts).limit(50);`;

export function Demo() {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: '-80px' });

  return (
    <section
      ref={ref}
      className="flex flex-col items-center px-4 py-16 md:py-24"
    >
      <motion.h2
        initial={{ opacity: 0, y: 16 }}
        animate={isInView ? { opacity: 1, y: 0 } : {}}
        transition={{ duration: 0.5, ease: 'easeOut' }}
        className="font-heading text-2xl/tight font-bold tracking-tight text-center"
      >
        See it in action
      </motion.h2>

      <motion.p
        initial={{ opacity: 0, y: 12 }}
        animate={isInView ? { opacity: 1, y: 0 } : {}}
        transition={{ duration: 0.5, delay: 0.1, ease: 'easeOut' }}
        className="mt-3 text-sm text-muted-foreground text-center max-w-xl"
      >
        Pulsar analyzes your TypeScript + Prisma schema in a unified pipeline.
        One command, zero config.
      </motion.p>

      <div className="mt-10 grid gap-4 w-full max-w-4xl md:grid-cols-2">
        <motion.div
          initial={{ opacity: 0, x: -20 }}
          animate={isInView ? { opacity: 1, x: 0 } : {}}
          transition={{ duration: 0.5, delay: 0.2, ease: 'easeOut' }}
        >
          <DynamicCodeBlock lang="ts" code={codeBefore} codeblock={{ className: "border-none ring-0 shadow-none" }} />
        </motion.div>

        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={isInView ? { opacity: 1, x: 0 } : {}}
          transition={{ duration: 0.5, delay: 0.35, ease: 'easeOut' }}
        >
          <DynamicCodeBlock lang="ts" code={codeAfter} codeblock={{ className: "border-none ring-0 shadow-none" }} />
        </motion.div>
      </div>

      <motion.p
        initial={{ opacity: 0 }}
        animate={isInView ? { opacity: 1 } : {}}
        transition={{ duration: 0.5, delay: 0.5, ease: 'easeOut' }}
        className="mt-4 text-xs text-muted-foreground text-center"
      >
        <span className="text-destructive">no-select-star</span>
        {' \u00B7 '}
        <span className="text-destructive">no-unbounded-find</span>
        {' \u00B7 '}
        <span className="text-amber-500">no-missing-limit</span>
        {' \u2014 '}all fixed
      </motion.p>

      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={isInView ? { opacity: 1, y: 0 } : {}}
        transition={{ duration: 0.5, delay: 0.6, ease: 'easeOut' }}
        className="mt-10"
      >
        <Link to="/docs/guide/getting-started">
          <Button size="lg">
            Start analyzing your code
          </Button>
        </Link>
      </motion.div>
    </section>
  );
}
