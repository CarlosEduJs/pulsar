const user = await db.select({ id: users.id }).from(users).where(eq(users.name, "test")).limit(1);
