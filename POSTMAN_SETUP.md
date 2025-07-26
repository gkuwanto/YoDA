# YoDA API Postman Test Suite

## Overview

This Postman collection provides comprehensive testing for the YoDA (Your D&D Assistant) API. The collection includes tests for all major endpoints including authentication, campaign management, session management, character management, and game state management.

## 📁 Files

- `YoDA_API_Tests.postman_collection.json` - The main Postman collection file
- `POSTMAN_SETUP.md` - This setup guide

## 🚀 Quick Start

### 1. Import the Collection

1. Open Postman
2. Click **Import** button
3. Select the `YoDA_API_Tests.postman_collection.json` file
4. The collection will be imported with all tests and variables

### 2. Set Up Environment Variables

The collection uses several variables that need to be configured:

#### Base URL
- **Variable**: `base_url`
- **Value**: `http://localhost:3000` (or your server URL)
- **Description**: The base URL for all API requests

#### Authentication Token
- **Variable**: `auth_token`
- **Value**: (Auto-populated after login)
- **Description**: JWT token for authenticated requests

#### Test Data IDs
- **Variable**: `campaign_id` (Auto-populated)
- **Variable**: `session_id` (Auto-populated)
- **Variable**: `character_id` (Auto-populated)
- **Variable**: `user_id` (Auto-populated)

### 3. Start Your YoDA Server

Before running tests, ensure your YoDA backend is running:

```bash
# Navigate to backend directory
cd backend

# Start the server
cargo run
```

The server should start on `http://localhost:3000`

## 📋 Test Structure

### 1. Health Check
- **Health Check**: Basic server connectivity test

### 2. Authentication
- **Register User**: Create a new user account
- **Register Duplicate User**: Test duplicate registration handling
- **Login Success**: Authenticate and get JWT token
- **Login Invalid Credentials**: Test invalid login handling

### 3. Campaign Management
- **Create Campaign**: Create a new campaign
- **List Campaigns**: Get all user's campaigns
- **Get Campaign**: Retrieve specific campaign details
- **Update Campaign**: Modify campaign information
- **Delete Campaign**: Remove a campaign

### 4. Session Management
- **Create Session**: Create a new game session
- **List Sessions**: Get all user's sessions
- **Get Session**: Retrieve specific session details
- **Start Session**: Begin a game session
- **End Session**: End a game session

### 5. Character Management
- **Create Character**: Create a new character
- **List Characters**: Get all user's characters
- **Get Character**: Retrieve specific character details
- **Update Character**: Modify character information
- **Update Character HP**: Update character health points
- **Delete Character**: Remove a character

### 6. Game State Management
- **Update Initiative**: Update initiative order and combat state

### 7. WebSocket Tests
- **WebSocket Connection Test**: Basic WebSocket endpoint test

### 8. Error Handling
- **Unauthorized Access**: Test access without authentication
- **Invalid Token**: Test with invalid JWT token
- **Resource Not Found**: Test 404 error handling

## 🧪 Running Tests

### Method 1: Run Individual Tests
1. Open the collection in Postman
2. Navigate to any test folder
3. Click on a specific test
4. Click **Send** to run the test
5. View results in the **Test Results** tab

### Method 2: Run Collection
1. Right-click on the collection name
2. Select **Run collection**
3. Configure test run settings:
   - **Iterations**: 1 (or more for stress testing)
   - **Delay**: 1000ms (1 second between requests)
   - **Log responses**: Enabled
4. Click **Run YoDA API Tests**

### Method 3: Run with Newman (CLI)
```bash
# Install Newman globally
npm install -g newman

# Run the collection
newman run YoDA_API_Tests.postman_collection.json

# Run with environment variables
newman run YoDA_API_Tests.postman_collection.json \
  --env-var "base_url=http://localhost:3000"

# Run with detailed output
newman run YoDA_API_Tests.postman_collection.json \
  --reporters cli,json \
  --reporter-json-export results.json
```

## 🔄 Test Flow

The tests are designed to run in sequence:

1. **Health Check** - Verify server is running
2. **Authentication** - Register and login to get token
3. **Campaign Management** - Create, read, update, delete campaigns
4. **Session Management** - Create and manage game sessions
5. **Character Management** - Create and manage characters
6. **Game State** - Test initiative and combat state
7. **Error Handling** - Test various error scenarios

## 📊 Test Results

Each test includes assertions that verify:
- **Status Codes**: Correct HTTP response codes
- **Response Structure**: Expected JSON structure
- **Data Validation**: Proper data types and values
- **Error Handling**: Appropriate error responses

## 🔧 Customization

### Adding New Tests
1. Create a new request in the appropriate folder
2. Add test scripts in the **Tests** tab
3. Use the existing variable structure
4. Follow the naming convention

### Modifying Variables
```javascript
// Set a variable
pm.collectionVariables.set("variable_name", "value");

// Get a variable
const value = pm.collectionVariables.get("variable_name");

// Use in requests
{{variable_name}}
```

### Environment-Specific Configuration
Create different environments for:
- **Development**: `http://localhost:3000`
- **Staging**: `https://staging.yoda-api.com`
- **Production**: `https://api.yoda-app.com`

## 🐛 Troubleshooting

### Common Issues

1. **Connection Refused**
   - Ensure YoDA server is running
   - Check if port 3000 is available
   - Verify firewall settings

2. **Authentication Errors**
   - Run the login test first
   - Check if JWT token is valid
   - Verify token format in Authorization header

3. **Test Failures**
   - Check server logs for errors
   - Verify database connection
   - Ensure all required tables exist

4. **Variable Issues**
   - Check if variables are properly set
   - Verify variable names match exactly
   - Clear and re-run authentication tests

### Debug Mode
Enable detailed logging in Postman:
1. Go to **View** → **Show Postman Console**
2. Check **Console** for detailed request/response logs
3. Use `console.log()` in test scripts for debugging

## 📈 Performance Testing

### Load Testing with Newman
```bash
# Run with multiple iterations
newman run YoDA_API_Tests.postman_collection.json \
  --iteration-count 100 \
  --delay-request 100

# Run with concurrent requests
newman run YoDA_API_Tests.postman_collection.json \
  --iteration-count 50 \
  --concurrency 10
```

### Stress Testing
```bash
# High load test
newman run YoDA_API_Tests.postman_collection.json \
  --iteration-count 1000 \
  --concurrency 50 \
  --delay-request 50
```

## 🔒 Security Testing

The collection includes security tests:
- **Unauthorized Access**: Tests without authentication
- **Invalid Tokens**: Tests with malformed JWT
- **Resource Access Control**: Tests permission boundaries

## 📝 Best Practices

1. **Run Tests in Order**: Some tests depend on previous test results
2. **Clean Up Data**: Consider adding cleanup tests for production
3. **Use Variables**: Avoid hardcoding values
4. **Add Assertions**: Every test should have meaningful assertions
5. **Document Changes**: Update this guide when modifying tests

## 🚀 CI/CD Integration

### GitHub Actions Example
```yaml
name: API Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup Node.js
        uses: actions/setup-node@v2
        with:
          node-version: '16'
      - name: Install Newman
        run: npm install -g newman
      - name: Run API Tests
        run: |
          newman run YoDA_API_Tests.postman_collection.json \
            --env-var "base_url=http://localhost:3000" \
            --reporters cli,junit \
            --reporter-junit-export test-results.xml
```

## 📞 Support

For issues with the test suite:
1. Check the YoDA server logs
2. Verify database connectivity
3. Test individual endpoints manually
4. Review Postman console for detailed error messages

## 📚 Additional Resources

- [Postman Documentation](https://learning.postman.com/)
- [Newman CLI Documentation](https://learning.postman.com/docs/running-collections/using-newman-cli/)
- [YoDA API Documentation](http://localhost:3000/docs)

---

**Happy Testing! 🎲**

This test suite ensures your YoDA API is robust, secure, and ready for production use. 