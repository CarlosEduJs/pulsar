const post = await db.select({ id: posts.id }).from(posts).where(eq(posts.authorId, 1)).limit(1);
