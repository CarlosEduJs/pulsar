const users = await db.select({ id: users.id, name: users.name }).from(users).limit(10);
