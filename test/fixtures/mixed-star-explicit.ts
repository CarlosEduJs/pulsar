const bad = await db.select().from(users);
const good = await db.select({ id: users.id, name: users.name }).from(users);
const alsoBad = await db.select().from(posts).where(eq(posts.id, 1));
