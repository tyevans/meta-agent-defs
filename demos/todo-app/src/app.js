const express = require('express');
const { authMiddleware, generateToken } = require('./auth');
const routes = require('./routes');

const app = express();
const PORT = process.env.PORT || 3000;

app.use(express.json());

// Demo login endpoint - not part of the main TODO API
app.post('/login', (req, res) => {
  const { username } = req.body;

  // ISSUE: No validation on username
  // In a real app, this would verify credentials
  const userId = 1; // Mock user ID
  const token = generateToken(userId);

  res.json({ token });
});

// All TODO routes require authentication
app.use('/api', authMiddleware, routes);

// Basic error handler
app.use((err, req, res, next) => {
  console.error(err.stack);
  res.status(500).json({ error: 'Something went wrong' });
});

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({ status: 'ok' });
});

if (require.main === module) {
  app.listen(PORT, () => {
    console.log(`TODO API listening on port ${PORT}`);
  });
}

module.exports = app;
