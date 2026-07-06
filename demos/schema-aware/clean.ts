import { eq } from "drizzle-orm";
import { db } from "../db";
import { users, posts, categories } from "../schema";

await db
  .select({ id: users.id, name: users.name, email: users.email })
  .from(users)
  .where(eq(users.email, "user@example.com"))
  .limit(10);

await db
  .select({ id: posts.id, title: posts.title, content: posts.content })
  .from(posts)
  .where(eq(posts.authorId, 1))
  .limit(20);
