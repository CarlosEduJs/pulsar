const result = await db.execute(sql`SELECT * FROM users WHERE id = ${userId}`);
