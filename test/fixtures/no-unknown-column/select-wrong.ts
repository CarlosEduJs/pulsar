const user = await db.select({ nonexistent: users.nonexistent }).from(users).limit(1);
