// =====================================================================================
// K6 Load Test Script for RWA Platform API
// Tests various API endpoints under different load conditions
// =====================================================================================

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const responseTime = new Trend('response_time');

// Test configuration
export const options = {
  stages: [
    { duration: '2m', target: 10 },   // Ramp up to 10 users
    { duration: '5m', target: 10 },   // Stay at 10 users
    { duration: '2m', target: 20 },   // Ramp up to 20 users
    { duration: '5m', target: 20 },   // Stay at 20 users
    { duration: '2m', target: 50 },   // Ramp up to 50 users
    { duration: '5m', target: 50 },   // Stay at 50 users
    { duration: '5m', target: 0 },    // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests must complete below 500ms
    http_req_failed: ['rate<0.1'],    // Error rate must be below 10%
    errors: ['rate<0.1'],             // Custom error rate must be below 10%
  },
};

// Base URL configuration
const BASE_URL = __ENV.API_BASE_URL || 'http://localhost:8080';

// Test data
const testUsers = [];
const testAssets = [];

// Generate test user data
function generateTestUser(index) {
  return {
    email: `loadtest${index}@example.com`,
    password: 'LoadTest123!',
    first_name: `LoadTest${index}`,
    last_name: 'User',
    phone: `+1555000${String(index).padStart(4, '0')}`,
  };
}

// Generate test asset data
function generateTestAsset(index) {
  return {
    name: `Load Test Asset ${index}`,
    description: `Asset created during load testing - ${index}`,
    asset_type: 'real_estate',
    total_value: 100000 + (index * 1000),
    currency: 'USD',
    location: `Test City ${index % 10}, Test State`,
  };
}

// Setup function - runs once per VU
export function setup() {
  console.log('Setting up load test...');
  
  // Health check
  const healthResponse = http.get(`${BASE_URL}/health`);
  check(healthResponse, {
    'health check status is 200': (r) => r.status === 200,
    'health check response time < 100ms': (r) => r.timings.duration < 100,
  });

  return { baseUrl: BASE_URL };
}

// Main test function
export default function (data) {
  const vuId = __VU;
  const iterationId = __ITER;
  
  // Test scenarios based on VU ID
  if (vuId % 4 === 0) {
    testUserRegistrationAndLogin(data, vuId, iterationId);
  } else if (vuId % 4 === 1) {
    testAssetOperations(data, vuId, iterationId);
  } else if (vuId % 4 === 2) {
    testPaymentOperations(data, vuId, iterationId);
  } else {
    testMixedOperations(data, vuId, iterationId);
  }

  sleep(1); // Think time between iterations
}

// Test user registration and login flow
function testUserRegistrationAndLogin(data, vuId, iterationId) {
  const userIndex = vuId * 1000 + iterationId;
  const userData = generateTestUser(userIndex);

  // Register user
  const registerResponse = http.post(
    `${data.baseUrl}/api/v1/auth/register`,
    JSON.stringify(userData),
    {
      headers: { 'Content-Type': 'application/json' },
      tags: { endpoint: 'register' },
    }
  );

  const registerSuccess = check(registerResponse, {
    'registration status is 201': (r) => r.status === 201,
    'registration response time < 1000ms': (r) => r.timings.duration < 1000,
    'registration returns user_id': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.user_id !== undefined;
      } catch (e) {
        return false;
      }
    },
  });

  errorRate.add(!registerSuccess);
  responseTime.add(registerResponse.timings.duration);

  if (!registerSuccess) {
    console.error(`Registration failed for user ${userIndex}: ${registerResponse.status}`);
    return;
  }

  sleep(0.5);

  // Login user
  const loginData = {
    email: userData.email,
    password: userData.password,
  };

  const loginResponse = http.post(
    `${data.baseUrl}/api/v1/auth/login`,
    JSON.stringify(loginData),
    {
      headers: { 'Content-Type': 'application/json' },
      tags: { endpoint: 'login' },
    }
  );

  const loginSuccess = check(loginResponse, {
    'login status is 200': (r) => r.status === 200,
    'login response time < 500ms': (r) => r.timings.duration < 500,
    'login returns access_token': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.access_token !== undefined;
      } catch (e) {
        return false;
      }
    },
  });

  errorRate.add(!loginSuccess);
  responseTime.add(loginResponse.timings.duration);

  if (loginSuccess) {
    const loginBody = JSON.parse(loginResponse.body);
    testAuthenticatedEndpoints(data, loginBody.access_token);
  }
}

// Test asset operations
function testAssetOperations(data, vuId, iterationId) {
  // First, authenticate
  const token = authenticateTestUser(data, vuId);
  if (!token) return;

  const assetIndex = vuId * 1000 + iterationId;
  const assetData = generateTestAsset(assetIndex);

  // Create asset
  const createResponse = http.post(
    `${data.baseUrl}/api/v1/assets`,
    JSON.stringify(assetData),
    {
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      tags: { endpoint: 'create_asset' },
    }
  );

  const createSuccess = check(createResponse, {
    'asset creation status is 201': (r) => r.status === 201,
    'asset creation response time < 1000ms': (r) => r.timings.duration < 1000,
  });

  errorRate.add(!createSuccess);
  responseTime.add(createResponse.timings.duration);

  if (createSuccess) {
    const assetBody = JSON.parse(createResponse.body);
    const assetId = assetBody.id;

    sleep(0.5);

    // Get asset
    const getResponse = http.get(
      `${data.baseUrl}/api/v1/assets/${assetId}`,
      {
        headers: { 'Authorization': `Bearer ${token}` },
        tags: { endpoint: 'get_asset' },
      }
    );

    check(getResponse, {
      'asset retrieval status is 200': (r) => r.status === 200,
      'asset retrieval response time < 300ms': (r) => r.timings.duration < 300,
    });

    responseTime.add(getResponse.timings.duration);
  }
}

// Test payment operations
function testPaymentOperations(data, vuId, iterationId) {
  // First, authenticate
  const token = authenticateTestUser(data, vuId);
  if (!token) return;

  const paymentData = {
    amount: 100 + (iterationId % 1000),
    currency: 'USD',
    payment_method_type: 'credit_card',
    provider: 'stripe',
    description: `Load test payment ${vuId}-${iterationId}`,
  };

  // Process payment
  const paymentResponse = http.post(
    `${data.baseUrl}/api/v1/payments`,
    JSON.stringify(paymentData),
    {
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      tags: { endpoint: 'process_payment' },
    }
  );

  const paymentSuccess = check(paymentResponse, {
    'payment processing status is 201': (r) => r.status === 201,
    'payment processing response time < 2000ms': (r) => r.timings.duration < 2000,
  });

  errorRate.add(!paymentSuccess);
  responseTime.add(paymentResponse.timings.duration);

  if (paymentSuccess) {
    const paymentBody = JSON.parse(paymentResponse.body);
    const paymentId = paymentBody.id;

    sleep(1);

    // Get payment status
    const statusResponse = http.get(
      `${data.baseUrl}/api/v1/payments/${paymentId}`,
      {
        headers: { 'Authorization': `Bearer ${token}` },
        tags: { endpoint: 'get_payment' },
      }
    );

    check(statusResponse, {
      'payment status retrieval is 200': (r) => r.status === 200,
      'payment status response time < 300ms': (r) => r.timings.duration < 300,
    });

    responseTime.add(statusResponse.timings.duration);
  }
}

// Test mixed operations (realistic user behavior)
function testMixedOperations(data, vuId, iterationId) {
  // First, authenticate
  const token = authenticateTestUser(data, vuId);
  if (!token) return;

  // Get user profile
  const profileResponse = http.get(
    `${data.baseUrl}/api/v1/users/profile`,
    {
      headers: { 'Authorization': `Bearer ${token}` },
      tags: { endpoint: 'get_profile' },
    }
  );

  check(profileResponse, {
    'profile retrieval status is 200': (r) => r.status === 200,
    'profile retrieval response time < 300ms': (r) => r.timings.duration < 300,
  });

  responseTime.add(profileResponse.timings.duration);

  sleep(0.5);

  // List assets
  const assetsResponse = http.get(
    `${data.baseUrl}/api/v1/assets?page=1&per_page=10`,
    {
      headers: { 'Authorization': `Bearer ${token}` },
      tags: { endpoint: 'list_assets' },
    }
  );

  check(assetsResponse, {
    'assets listing status is 200': (r) => r.status === 200,
    'assets listing response time < 500ms': (r) => r.timings.duration < 500,
  });

  responseTime.add(assetsResponse.timings.duration);

  sleep(0.5);

  // List payments
  const paymentsResponse = http.get(
    `${data.baseUrl}/api/v1/payments?page=1&per_page=10`,
    {
      headers: { 'Authorization': `Bearer ${token}` },
      tags: { endpoint: 'list_payments' },
    }
  );

  check(paymentsResponse, {
    'payments listing status is 200': (r) => r.status === 200,
    'payments listing response time < 500ms': (r) => r.timings.duration < 500,
  });

  responseTime.add(paymentsResponse.timings.duration);
}

// Test authenticated endpoints
function testAuthenticatedEndpoints(data, token) {
  // Get user profile
  const profileResponse = http.get(
    `${data.baseUrl}/api/v1/users/profile`,
    {
      headers: { 'Authorization': `Bearer ${token}` },
      tags: { endpoint: 'get_profile' },
    }
  );

  check(profileResponse, {
    'authenticated profile access is 200': (r) => r.status === 200,
    'authenticated profile response time < 300ms': (r) => r.timings.duration < 300,
  });

  responseTime.add(profileResponse.timings.duration);
}

// Helper function to authenticate a test user
function authenticateTestUser(data, vuId) {
  const userIndex = vuId;
  const userData = generateTestUser(userIndex);

  // Try to login (user might already exist)
  const loginData = {
    email: userData.email,
    password: userData.password,
  };

  const loginResponse = http.post(
    `${data.baseUrl}/api/v1/auth/login`,
    JSON.stringify(loginData),
    {
      headers: { 'Content-Type': 'application/json' },
      tags: { endpoint: 'auth_login' },
    }
  );

  if (loginResponse.status === 200) {
    const loginBody = JSON.parse(loginResponse.body);
    return loginBody.access_token;
  }

  // If login failed, try to register first
  const registerResponse = http.post(
    `${data.baseUrl}/api/v1/auth/register`,
    JSON.stringify(userData),
    {
      headers: { 'Content-Type': 'application/json' },
      tags: { endpoint: 'auth_register' },
    }
  );

  if (registerResponse.status === 201) {
    // Now try to login again
    const secondLoginResponse = http.post(
      `${data.baseUrl}/api/v1/auth/login`,
      JSON.stringify(loginData),
      {
        headers: { 'Content-Type': 'application/json' },
        tags: { endpoint: 'auth_login_retry' },
      }
    );

    if (secondLoginResponse.status === 200) {
      const loginBody = JSON.parse(secondLoginResponse.body);
      return loginBody.access_token;
    }
  }

  console.error(`Failed to authenticate user ${userIndex}`);
  return null;
}

// Teardown function - runs once after all VUs finish
export function teardown(data) {
  console.log('Load test completed');
  
  // Final health check
  const healthResponse = http.get(`${data.baseUrl}/health`);
  check(healthResponse, {
    'final health check status is 200': (r) => r.status === 200,
  });
}
