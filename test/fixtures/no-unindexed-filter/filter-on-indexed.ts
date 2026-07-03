const user = await db.select({ id: users.id }).from(users).where(eq(users.email, "test@test.com")).limit(1);
