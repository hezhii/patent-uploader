import type { LoginRequest, LoginResponse, UploadResponse } from '@/types';

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

  async uploadFile(file: File, onProgress?: (progress: number) => void): Promise<UploadResponse> {
    if (!this.token) {
      throw new Error('请先登录');
    }

    return new Promise((resolve, reject) => {
      const formData = new FormData();
      formData.append('file', file);

      const xhr = new XMLHttpRequest();

      // 监听上传进度
      if (onProgress) {
        xhr.upload.addEventListener('progress', (event) => {
          if (event.lengthComputable) {
            const progress = (event.loaded / event.total) * 100;
            onProgress(Math.round(progress));
          }
        });
      }

      // 监听请求完成
      xhr.addEventListener('load', () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          try {
            const result: UploadResponse = JSON.parse(xhr.responseText);
            resolve(result);
          } catch (error) {
            reject(new Error('解析响应数据失败'));
          }
        } else {
          reject(new Error(`上传失败: ${xhr.status} ${xhr.statusText}`));
        }
      });

      // 监听请求错误
      xhr.addEventListener('error', () => {
        reject(new Error('网络错误'));
      });

      // 监听请求中断
      xhr.addEventListener('abort', () => {
        reject(new Error('上传被中断'));
      });

      // 设置请求头和发送请求
      xhr.open('POST', `${this.baseUrl}/admin/patent/import`);
      xhr.setRequestHeader('Authorization', `Bearer ${this.token}`);
      xhr.send(formData);
    });
  }

  async testConnection(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/admin/login`, {
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