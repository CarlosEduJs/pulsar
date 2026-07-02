const posts = await db.select({ id: posts.id }).from(posts).limit(10);
