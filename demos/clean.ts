import { eq } from "drizzle-orm";
import { db } from "./db";
import { users, posts, categories } from "./schema";

await db
  .select({ id: users.id, name: users.name, email: users.email })
  .from(users)
  .where(eq(users.id, 1))
  .limit(10);

await db
  .select({ id: posts.id, title: posts.title, content: posts.content })
  .from(posts)
  .limit(20);

const categoriesList = await db
  .select({ id: categories.id, name: categories.name })
  .from(categories)
  .where(eq(categories.isActive, true))
  .limit(5);

function greet(name: string): string {
  return `Hello, ${name}!`;
}
