getUsers().then(() => {
  return db.select().from(users);
});
