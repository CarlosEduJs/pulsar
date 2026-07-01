const users = await db.select({ id: users.id, name: users.name, email: users.email }).from(users);
const posts = await db.select({ id: posts.id, title: posts.title, content: posts.content }).from(posts);
