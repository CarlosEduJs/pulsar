const user = await db.select().from(users).where(eq(users.id, 1));
const activeUser = await db.select({ id: users.id, name: users.name }).from(users).where(eq(users.active, true));
