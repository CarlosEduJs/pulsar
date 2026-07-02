const users = await db.select({ id: users.id, name: users.name, email: users.email }).from(users).limit(10);
const posts = await db.select({ id: posts.id, title: posts.title, content: posts.content }).from(posts).limit(20);
