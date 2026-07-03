const users = await db.select({ id: users.id }).from(users).limit(10);
