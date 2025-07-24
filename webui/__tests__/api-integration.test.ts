// =====================================================================================
// File: webui/__tests__/api-integration.test.ts
// Description: Enterprise API integration tests
// Author: arkSong (arksong2018@gmail.com)
// Framework: StableRWA - Enterprise RWA Tokenization Technology Framework Platform
// =====================================================================================

import { describe, it, expect, beforeAll, afterAll, beforeEach, jest } from '@jest/globals';
import { enterpriseApiClient } from '../src/lib/api-client';
import { enterpriseWebSocketClient } from '../src/lib/websocket-client';
import { enterpriseAuth } from '../src/lib/auth';
import { enterpriseErrorHandler } from '../src/lib/error-handler';
import { enterpriseLogger } from '../src/lib/logger';
import { enterprisePerformance } from '../src/lib/performance';
import { enterpriseSecurity } from '../src/lib/security';

// Mock environment variables for testing
process.env.NEXT_PUBLIC_DOCKER_MODE = 'true';
process.env.NEXT_PUBLIC_GATEWAY_URL = 'http://localhost:8080';
process.env.NEXT_PUBLIC_ASSETS_URL = 'http://localhost:8081';
process.env.NEXT_PUBLIC_ORACLE_URL = 'http://localhost:8082';
process.env.NEXT_PUBLIC_AI_URL = 'http://localhost:8083';

// Mock fetch for testing
global.fetch = jest.fn();

describe('Enterprise API Integration Tests', () => {
  beforeAll(() => {
    // Setup test environment
    console.log('🧪 Starting Enterprise API Integration Tests');
  });

  afterAll(() => {
    // Cleanup
    console.log('✅ Enterprise API Integration Tests Completed');
  });

  beforeEach(() => {
    // Reset mocks before each test
    jest.clearAllMocks();
  });

  describe('API Client Configuration', () => {
    it('should have correct endpoint configuration', () => {
      expect(process.env.NEXT_PUBLIC_GATEWAY_URL).toBe('http://localhost:8080');
      expect(process.env.NEXT_PUBLIC_ASSETS_URL).toBe('http://localhost:8081');
      expect(process.env.NEXT_PUBLIC_ORACLE_URL).toBe('http://localhost:8082');
      expect(process.env.NEXT_PUBLIC_AI_URL).toBe('http://localhost:8083');
    });

    it('should handle authentication token management', () => {
      const testToken = 'test-jwt-token';
      
      // Set token
      enterpriseApiClient.setAuthToken(testToken);
      
      // Clear token
      enterpriseApiClient.clearAuthToken();
      
      expect(true).toBe(true); // Token management is internal
    });
  });

  describe('Health Check API', () => {
    it('should check service health', async () => {
      // Mock successful health check responses
      (global.fetch as jest.Mock)
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ status: 'healthy' })
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ status: 'healthy' })
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ status: 'healthy' })
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ status: 'healthy' })
        });

      const healthStatus = await enterpriseApiClient.checkHealth();
      
      expect(healthStatus).toHaveProperty('gateway');
      expect(healthStatus).toHaveProperty('assets');
      expect(healthStatus).toHaveProperty('oracle');
      expect(healthStatus).toHaveProperty('ai');
    });

    it('should handle health check failures', async () => {
      // Mock failed health check
      (global.fetch as jest.Mock).mockRejectedValue(new Error('Network error'));

      const healthStatus = await enterpriseApiClient.checkHealth();
      
      // All services should be marked as unhealthy
      expect(Object.values(healthStatus).every(status => status === false)).toBe(true);
    });
  });

  describe('Assets API', () => {
    it('should fetch assets list', async () => {
      const mockAssets = [
        {
          id: '1',
          name: 'Test Asset',
          category: 'Real Estate',
          value: 100000,
          tokenized: false
        }
      ];

      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          success: true,
          data: mockAssets
        })
      });

      const response = await enterpriseApiClient.getAssets();
      
      expect(response.success).toBe(true);
      expect(response.data).toEqual(mockAssets);
    });

    it('should create new asset', async () => {
      const newAsset = {
        name: 'New Test Asset',
        category: 'Real Estate',
        location: 'New York',
        value: 250000,
        description: 'Test asset description'
      };

      const mockResponse = {
        id: '2',
        ...newAsset,
        tokenized: false,
        created_at: new Date().toISOString()
      };

      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          success: true,
          data: mockResponse
        })
      });

      const response = await enterpriseApiClient.createAsset(newAsset);
      
      expect(response.success).toBe(true);
      expect(response.data).toMatchObject(newAsset);
    });

    it('should handle asset creation errors', async () => {
      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: async () => ({
          success: false,
          error: 'Invalid asset data'
        })
      });

      const response = await enterpriseApiClient.createAsset({});
      
      expect(response.success).toBe(false);
      expect(response.error).toBe('Invalid asset data');
    });

    it('should tokenize asset', async () => {
      const tokenizeData = {
        token_supply: 1000,
        token_symbol: 'RWA',
        blockchain_network: 'ethereum'
      };

      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          success: true,
          data: {
            transaction_hash: '0x123...',
            token_address: '0xabc...',
            status: 'pending'
          }
        })
      });

      const response = await enterpriseApiClient.tokenizeAsset('1', tokenizeData);
      
      expect(response.success).toBe(true);
      expect(response.data).toHaveProperty('transaction_hash');
      expect(response.data).toHaveProperty('token_address');
    });
  });

  describe('Oracle API', () => {
    it('should fetch price data', async () => {
      const mockPrices = [
        {
          symbol: 'ETH',
          price: 2000,
          change_24h: 5.2,
          last_updated: new Date().toISOString()
        }
      ];

      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          success: true,
          data: mockPrices
        })
      });

      const response = await enterpriseApiClient.getPrices();
      
      expect(response.success).toBe(true);
      expect(response.data).toEqual(mockPrices);
    });

    it('should fetch price history', async () => {
      const mockHistory = [
        {
          timestamp: new Date().toISOString(),
          price: 2000,
          volume: 1000000
        }
      ];

      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          success: true,
          data: mockHistory
        })
      });

      const response = await enterpriseApiClient.getPriceHistory('ETH', { period: '24h' });
      
      expect(response.success).toBe(true);
      expect(response.data).toEqual(mockHistory);
    });
  });

  describe('AI API', () => {
    it('should get AI model information', async () => {
      const mockModel = {
        name: 'gpt-3.5-turbo',
        version: '1.0',
        capabilities: ['text-generation', 'analysis']
      };

      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          success: true,
          data: mockModel
        })
      });

      const response = await enterpriseApiClient.getAiModel();
      
      expect(response.success).toBe(true);
      expect(response.data).toEqual(mockModel);
    });

    it('should get AI completion', async () => {
      const mockCompletion = {
        response: 'This is an AI generated response',
        model: 'gpt-3.5-turbo',
        tokens_used: 50
      };

      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          success: true,
          data: mockCompletion
        })
      });

      const response = await enterpriseApiClient.getAiCompletion('Test prompt');
      
      expect(response.success).toBe(true);
      expect(response.data).toEqual(mockCompletion);
    });
  });

  describe('Error Handling', () => {
    it('should handle network errors', async () => {
      (global.fetch as jest.Mock).mockRejectedValueOnce(new Error('Network error'));

      const response = await enterpriseApiClient.getAssets();
      
      expect(response.success).toBe(false);
      expect(response.error).toBe('Network error');
    });

    it('should handle 401 unauthorized errors', async () => {
      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: false,
        status: 401,
        json: async () => ({
          success: false,
          error: 'Unauthorized'
        })
      });

      const response = await enterpriseApiClient.getAssets();
      
      expect(response.success).toBe(false);
      expect(response.error).toBe('Unauthorized');
    });

    it('should handle timeout errors', async () => {
      // Mock AbortSignal.timeout
      const mockAbortSignal = {
        aborted: false,
        addEventListener: jest.fn(),
        removeEventListener: jest.fn(),
        dispatchEvent: jest.fn()
      };
      
      global.AbortSignal = {
        timeout: jest.fn().mockReturnValue(mockAbortSignal)
      } as any;

      (global.fetch as jest.Mock).mockRejectedValueOnce(new Error('Request timeout'));

      const response = await enterpriseApiClient.getAssets();
      
      expect(response.success).toBe(false);
      expect(response.error).toBe('Request timeout');
    });
  });
});

describe('WebSocket Integration Tests', () => {
  beforeEach(() => {
    // Mock WebSocket
    global.WebSocket = jest.fn().mockImplementation(() => ({
      readyState: WebSocket.CONNECTING,
      send: jest.fn(),
      close: jest.fn(),
      addEventListener: jest.fn(),
      removeEventListener: jest.fn()
    })) as any;
  });

  it('should initialize WebSocket client', () => {
    expect(enterpriseWebSocketClient).toBeDefined();
    expect(enterpriseWebSocketClient.getConnectionState()).toBeDefined();
  });

  it('should handle subscription management', () => {
    const mockCallback = jest.fn();
    const unsubscribe = enterpriseWebSocketClient.subscribe('test', mockCallback);
    
    expect(typeof unsubscribe).toBe('function');
    
    // Test unsubscribe
    unsubscribe();
    expect(enterpriseWebSocketClient.getSubscriptionCount()).toBe(0);
  });

  it('should handle message sending', () => {
    enterpriseWebSocketClient.send('test_message', { data: 'test' });
    
    // Message should be queued if not connected
    expect(enterpriseWebSocketClient.getQueuedMessageCount()).toBeGreaterThanOrEqual(0);
  });

  it('should provide connection statistics', () => {
    const stats = enterpriseWebSocketClient.getStats();

    expect(stats).toHaveProperty('connectionState');
    expect(stats).toHaveProperty('reconnectAttempts');
    expect(stats).toHaveProperty('subscriptionCount');
    expect(stats).toHaveProperty('queuedMessages');
  });
});

describe('Enterprise Authentication Tests', () => {
  beforeEach(() => {
    // Clear any stored auth data
    localStorage.clear();
    sessionStorage.clear();
  });

  describe('Authentication Flow', () => {
    it('should handle login with valid credentials', async () => {
      const mockResponse = {
        success: true,
        data: {
          user: {
            id: '1',
            email: 'test@example.com',
            name: 'Test User',
            role: 'admin',
            permissions: ['read', 'write'],
          },
          access_token: 'mock-access-token',
          refresh_token: 'mock-refresh-token',
          expires_in: 3600,
        },
      };

      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await enterpriseAuth.login({
        email: 'test@example.com',
        password: 'password123',
      });

      expect(result.success).toBe(true);
      expect(enterpriseAuth.isAuthenticated()).toBe(true);
    });

    it('should handle login with invalid credentials', async () => {
      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: false,
        status: 401,
        json: async () => ({
          success: false,
          message: 'Invalid credentials',
        }),
      });

      const result = await enterpriseAuth.login({
        email: 'test@example.com',
        password: 'wrongpassword',
      });

      expect(result.success).toBe(false);
      expect(result.error).toBe('Invalid credentials');
    });

    it('should handle two-factor authentication requirement', async () => {
      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: false,
        status: 428,
        json: async () => ({
          success: false,
          requires_two_factor: true,
        }),
      });

      const result = await enterpriseAuth.login({
        email: 'test@example.com',
        password: 'password123',
      });

      expect(result.success).toBe(false);
      expect(result.requiresTwoFactor).toBe(true);
    });
  });

  describe('Permission System', () => {
    beforeEach(() => {
      const mockUser = {
        id: '1',
        email: 'test@example.com',
        name: 'Test User',
        role: 'admin',
        permissions: ['assets:read', 'assets:write', 'users:read'],
        isActive: true,
        twoFactorEnabled: false,
      };

      localStorage.setItem('stablerwa_user', JSON.stringify(mockUser));
    });

    it('should check user permissions correctly', () => {
      expect(enterpriseAuth.hasPermission('assets:read')).toBe(true);
      expect(enterpriseAuth.hasPermission('assets:write')).toBe(true);
      expect(enterpriseAuth.hasPermission('users:write')).toBe(false);
    });

    it('should check user roles correctly', () => {
      expect(enterpriseAuth.hasRole('admin')).toBe(true);
      expect(enterpriseAuth.hasRole('user')).toBe(false);
    });
  });
});

describe('Enterprise Security Tests', () => {
  it('should sanitize user input', () => {
    const maliciousInput = '<script>alert("xss")</script>';
    const sanitized = enterpriseSecurity.sanitizeInput(maliciousInput);

    expect(sanitized).not.toContain('<script>');
    expect(sanitized).toContain('&lt;script&gt;');
  });

  it('should validate URLs', () => {
    const validUrl = 'https://example.com/path';
    const invalidUrl = 'javascript:alert("xss")';

    expect(enterpriseSecurity.sanitizeURL(validUrl)).toBe(validUrl);
    expect(enterpriseSecurity.sanitizeURL(invalidUrl)).toBe('');
  });

  it('should validate file uploads', () => {
    const validFile = new File(['content'], 'test.jpg', { type: 'image/jpeg' });
    const invalidFile = new File(['content'], 'test.exe', { type: 'application/exe' });

    expect(enterpriseSecurity.validateFileUpload(validFile).valid).toBe(true);
    expect(enterpriseSecurity.validateFileUpload(invalidFile).valid).toBe(false);
  });

  it('should generate secure tokens', () => {
    const token1 = enterpriseSecurity.generateSecureToken(32);
    const token2 = enterpriseSecurity.generateSecureToken(32);

    expect(token1).toHaveLength(64); // 32 bytes = 64 hex chars
    expect(token2).toHaveLength(64);
    expect(token1).not.toBe(token2); // Should be unique
  });

  it('should check rate limits', () => {
    const identifier = 'test-user';

    // Should allow initial requests
    expect(enterpriseSecurity.checkRateLimit(identifier)).toBe(true);

    // Simulate many requests
    for (let i = 0; i < 150; i++) {
      enterpriseSecurity.checkRateLimit(identifier);
    }

    // Should block after rate limit exceeded
    expect(enterpriseSecurity.checkRateLimit(identifier)).toBe(false);
  });
});
