const posts = await db.select({ id: posts.id, title: posts.title }).from(posts).limit(10);
