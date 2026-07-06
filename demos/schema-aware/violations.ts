import { eq } from "drizzle-orm";
import { db } from "../db";
import { users, posts, categories } from "../schema";

await db
  .select({ id: users.id, name: users.name, nonexistent: users.nonexistent })
  .from(users)
  .where(eq(users.id, 1))
  .limit(10);

await db
  .select({ id: posts.id, title: posts.title, content: posts.content })
  .from(posts)
  .where(eq(posts.tags, "rust"))
  .limit(20);

await db
  .select({ id: users.id, name: users.name })
  .from(users)
  .where(eq(users.name, "Alice"))
  .limit(5);
