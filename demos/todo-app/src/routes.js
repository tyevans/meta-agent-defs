const express = require('express');
const db = require('./db');

const router = express.Router();

// ISSUE: No input validation on any route handler
// ISSUE: Routes and business logic mixed together (architecture smell)
// ISSUE: No pagination on list endpoint (performance)

router.get('/todos', (req, res) => {
  db.getAllTodos(req.userId, (err, todos) => {
    if (err) {
      return res.status(500).json({ error: 'Database error' });
    }
    res.json(todos);
  });
});

router.get('/todos/:id', (req, res) => {
  const id = req.params.id;

  db.getTodoById(id, req.userId, (err, todo) => {
    if (err) {
      return res.status(500).json({ error: 'Database error' });
    }
    if (!todo) {
      return res.status(404).json({ error: 'Todo not found' });
    }
    res.json(todo);
  });
});

router.post('/todos', (req, res) => {
  const { title } = req.body;

  // ISSUE: No validation - title could be undefined, empty, or too long
  db.createTodo(title, req.userId, (err, id) => {
    if (err) {
      return res.status(500).json({ error: 'Database error' });
    }
    res.status(201).json({ id, title, completed: 0 });
  });
});

router.put('/todos/:id', (req, res) => {
  const id = req.params.id;
  const { title, completed } = req.body;

  // ISSUE: No validation on updates
  const updates = {};
  if (title !== undefined) updates.title = title;
  if (completed !== undefined) updates.completed = completed ? 1 : 0;

  db.updateTodo(id, updates, (err) => {
    if (err) {
      return res.status(500).json({ error: 'Database error' });
    }
    res.json({ success: true });
  });
});

router.delete('/todos/:id', (req, res) => {
  const id = req.params.id;

  db.deleteTodo(id, (err) => {
    if (err) {
      return res.status(500).json({ error: 'Database error' });
    }
    res.json({ success: true });
  });
});

module.exports = router;
