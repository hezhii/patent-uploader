import type { LoginRequest, LoginResponse } from '@/types';

export class HttpClient {
  private baseUrl: string = '';
  private token: string = '';

  setBaseUrl(url: string) {
    this.baseUrl = url.replace(/\/$/, ''); // 移除末尾斜杠
  }

  async login(username: string, password: string): Promise<LoginResponse> {
    const response = await fetch(`${this.baseUrl}/auth/admin/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ username, password } as LoginRequest),
    });

    if (!response.ok) {
      throw new Error(`登录失败: ${response.status} ${response.statusText}`);
    }

    const result: LoginResponse = await response.json();
    
    if (result.success && result.data?.token) {
      this.token = result.data.token;
      return result;
    } else {
      throw new Error('登录失败：无效的响应数据');
    }
  }

  async testConnection(): Promise<boolean> {
    try {
      await fetch(`${this.baseUrl}/auth/admin/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username: 'test', password: 'test' }),
      });
      
      // 只要能收到响应就说明服务器可达，不管是否登录成功
      return true;
    } catch (error) {
      return false;
    }
  }

  getToken(): string {
    return this.token;
  }

  clearToken() {
    this.token = '';
  }
}

export const httpClient = new HttpClient();