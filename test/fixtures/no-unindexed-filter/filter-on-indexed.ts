const user = await db.select().from(users).where(eq(users.email, "test@test.com")).limit(1);
