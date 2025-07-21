// =====================================================================================
// File: webui/src/lib/api.ts
// Description: API client for RWA Platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

export interface ApiResponse<T = any> {
  data?: T;
  error?: string;
  message?: string;
  success: boolean;
}

export interface Asset {
  id: string;
  name: string;
  description: string;
  asset_type: string;
  location: string;
  valuation: {
    amount: number;
    currency: string;
    valuation_date: string;
  };
  metadata?: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface User {
  id: string;
  username: string;
  email: string;
  role: string;
  created_at: string;
  last_login?: string;
}

export interface Transaction {
  id: string;
  asset_id: string;
  from_address: string;
  to_address: string;
  amount: number;
  transaction_hash: string;
  status: 'pending' | 'confirmed' | 'failed';
  created_at: string;
}

class ApiClient {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
    
    // Try to get token from localStorage on client side
    if (typeof window !== 'undefined') {
      this.token = localStorage.getItem('auth_token');
    }
  }

  setToken(token: string) {
    this.token = token;
    if (typeof window !== 'undefined') {
      localStorage.setItem('auth_token', token);
    }
  }

  clearToken() {
    this.token = null;
    if (typeof window !== 'undefined') {
      localStorage.removeItem('auth_token');
    }
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    if (this.token) {
      headers.Authorization = `Bearer ${this.token}`;
    }

    try {
      const response = await fetch(url, {
        ...options,
        headers,
      });

      const data = await response.json();

      if (!response.ok) {
        return {
          success: false,
          error: data.message || `HTTP ${response.status}`,
        };
      }

      return {
        success: true,
        data,
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }

  // Authentication
  async login(username: string, password: string): Promise<ApiResponse<{ access_token: string; user: User }>> {
    return this.request('/api/v1/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    });
  }

  async register(username: string, email: string, password: string): Promise<ApiResponse<{ user: User }>> {
    return this.request('/api/v1/auth/register', {
      method: 'POST',
      body: JSON.stringify({ username, email, password }),
    });
  }

  async logout(): Promise<ApiResponse> {
    const result = await this.request('/api/v1/auth/logout', {
      method: 'POST',
    });
    this.clearToken();
    return result;
  }

  // Assets
  async getAssets(): Promise<ApiResponse<Asset[]>> {
    return this.request('/api/v1/assets');
  }

  async getAsset(id: string): Promise<ApiResponse<Asset>> {
    return this.request(`/api/v1/assets/${id}`);
  }

  async createAsset(asset: Partial<Asset>): Promise<ApiResponse<Asset>> {
    return this.request('/api/v1/assets', {
      method: 'POST',
      body: JSON.stringify(asset),
    });
  }

  async updateAsset(id: string, asset: Partial<Asset>): Promise<ApiResponse<Asset>> {
    return this.request(`/api/v1/assets/${id}`, {
      method: 'PUT',
      body: JSON.stringify(asset),
    });
  }

  async deleteAsset(id: string): Promise<ApiResponse> {
    return this.request(`/api/v1/assets/${id}`, {
      method: 'DELETE',
    });
  }

  // Tokenization
  async tokenizeAsset(
    assetId: string,
    tokenSupply: number,
    tokenSymbol: string,
    blockchainNetwork: string
  ): Promise<ApiResponse<{ transaction_hash: string }>> {
    return this.request(`/api/v1/assets/${assetId}/tokenize`, {
      method: 'POST',
      body: JSON.stringify({
        token_supply: tokenSupply,
        token_symbol: tokenSymbol,
        blockchain_network: blockchainNetwork,
      }),
    });
  }

  // AI Services
  async getAiValuation(assetData: any): Promise<ApiResponse<{
    estimated_value: number;
    confidence_score: number;
    analysis_report: string;
  }>> {
    return this.request('/api/v1/ai/valuate', {
      method: 'POST',
      body: JSON.stringify(assetData),
    });
  }

  // Transactions
  async getTransactions(): Promise<ApiResponse<Transaction[]>> {
    return this.request('/api/v1/transactions');
  }

  async getTransaction(id: string): Promise<ApiResponse<Transaction>> {
    return this.request(`/api/v1/transactions/${id}`);
  }

  // Users
  async getUsers(): Promise<ApiResponse<User[]>> {
    return this.request('/api/v1/users');
  }

  async getUser(id: string): Promise<ApiResponse<User>> {
    return this.request(`/api/v1/users/${id}`);
  }

  async getCurrentUser(): Promise<ApiResponse<User>> {
    return this.request('/api/v1/user/profile');
  }

  // Health check
  async healthCheck(): Promise<ApiResponse<{ status: string }>> {
    return this.request('/health');
  }

  // IPFS
  async uploadToIpfs(content: string, filename: string, contentType: string): Promise<ApiResponse<{ hash: string }>> {
    return this.request('/api/v1/ipfs/upload', {
      method: 'POST',
      body: JSON.stringify({
        content,
        filename,
        content_type: contentType,
      }),
    });
  }

  async getFromIpfs(hash: string): Promise<ApiResponse<string>> {
    return this.request(`/api/v1/ipfs/${hash}`);
  }
}

export const apiClient = new ApiClient();
export default apiClient;
