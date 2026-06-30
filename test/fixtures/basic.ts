const users = await db.select().from(users);
const posts = await db.select({ id: posts.id, title: posts.title }).from(posts);
