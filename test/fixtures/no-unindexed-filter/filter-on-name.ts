const user = await db.select().from(users).where(eq(users.name, "test"));
