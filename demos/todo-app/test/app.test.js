const request = require('supertest');
const app = require('../src/app');

describe('TODO API', () => {
  let token;

  beforeAll(async () => {
    const response = await request(app)
      .post('/login')
      .send({ username: 'testuser' });
    token = response.body.token;
  });

  // ISSUE: Only happy path tests - no error cases or edge cases
  it('should create a todo', async () => {
    const response = await request(app)
      .post('/api/todos')
      .set('Authorization', `Bearer ${token}`)
      .send({ title: 'Test todo' });

    expect(response.status).toBe(201);
    expect(response.body.title).toBe('Test todo');
  });

  it('should get all todos', async () => {
    const response = await request(app)
      .get('/api/todos')
      .set('Authorization', `Bearer ${token}`);

    expect(response.status).toBe(200);
    expect(Array.isArray(response.body)).toBe(true);
  });

  // ISSUE: Missing tests for:
  // - Invalid input (empty title, too long title, missing title)
  // - Unauthorized access (no token, invalid token)
  // - Update operations
  // - Delete operations
  // - Getting single todo
  // - SQL injection attempts
  // - Edge cases (special characters, unicode, etc)
});
