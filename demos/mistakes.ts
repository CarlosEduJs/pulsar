import { eq, sql } from "drizzle-orm";
import { db } from "./db";
import { users, posts, categories } from "./schema";

const allUsers = await db.select().from(users);

for (let i = 0; i < users.length; i++) {
  const userPosts = await db
    .select()
    .from(posts)
    .where(eq(posts.authorId, i));
}

const user = db
  .select({ id: users.id, name: users.name })
  .from(users)
  .where(eq(users.id, 1))
  .then((res) => {
    return db.select().from(posts).where(eq(posts.authorId, res[0].id));
  });

const result = await db
  .select({ id: users.id, name: users.name })
  .from(users)
  .where(true)
  .limit(10);

const dangerous = await db.execute(
  sql`SELECT * FROM users WHERE email = ${email}`
);

const noLimit = await db
  .select({ id: users.id, name: users.name })
  .from(users);
