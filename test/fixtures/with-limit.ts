const recent = await db.select().from(comments).limit(10);
const paginated = await db.select({ id: posts.id, title: posts.title }).from(posts).limit(20);
