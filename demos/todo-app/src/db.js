const sqlite3 = require('sqlite3').verbose();
const path = require('path');

class Database {
  constructor() {
    this.db = new sqlite3.Database(':memory:');
    this.init();
  }

  init() {
    this.db.run(`
      CREATE TABLE todos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        completed INTEGER DEFAULT 0,
        user_id INTEGER NOT NULL
      )
    `);
  }

  // ISSUE: SQL injection vulnerability - string interpolation in queries
  getAllTodos(userId, callback) {
    const query = `SELECT * FROM todos WHERE user_id = ${userId}`;
    this.db.all(query, callback);
  }

  // ISSUE: SQL injection vulnerability
  getTodoById(id, userId, callback) {
    const query = `SELECT * FROM todos WHERE id = ${id} AND user_id = ${userId}`;
    this.db.get(query, callback);
  }

  // ISSUE: SQL injection vulnerability
  createTodo(title, userId, callback) {
    const query = `INSERT INTO todos (title, user_id) VALUES ('${title}', ${userId})`;
    this.db.run(query, function(err) {
      callback(err, this ? this.lastID : null);
    });
  }

  updateTodo(id, updates, callback) {
    const fields = [];
    if (updates.title !== undefined) fields.push(`title = '${updates.title}'`);
    if (updates.completed !== undefined) fields.push(`completed = ${updates.completed}`);

    const query = `UPDATE todos SET ${fields.join(', ')} WHERE id = ${id}`;
    this.db.run(query, callback);
  }

  deleteTodo(id, callback) {
    const query = `DELETE FROM todos WHERE id = ${id}`;
    this.db.run(query, callback);
  }
}

module.exports = new Database();
