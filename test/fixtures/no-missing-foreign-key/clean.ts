const posts = await db.select().from(posts).where(eq(posts.authorId, 1));
