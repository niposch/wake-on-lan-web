import axios from 'axios';
import type { ApiService } from './api.interface';
import type { 
    User, Device, LoginRequest, LoginResponse, CreateUserRequest, CreateDeviceRequest, UpdateDeviceRequest, RefreshTokenResponse
} from '../types';

const API_URL = '/api';

const client = axios.create({
    baseURL: API_URL,
    headers: {
        'Content-Type': 'application/json',
    },
});

// Add auth token to requests
client.interceptors.request.use((config) => {
    const token = localStorage.getItem('access_token');
    if (token) {
        config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
});

// Handle 401 Unauthorized globally with Refresh Logic
client.interceptors.response.use(
    (response) => response,
    async (error) => {
        const originalRequest = error.config;
        
        if (error.response?.status === 401 && !originalRequest._retry) {
            originalRequest._retry = true;
            const refreshToken = localStorage.getItem('refresh_token');

            if (refreshToken) {
                try {
                    // Call refresh endpoint directly using axios to avoid loop
                    const response = await axios.post<RefreshTokenResponse>(`${API_URL}/refresh`, {
                        refresh_token: refreshToken
                    });

                    const { access_token, refresh_token: new_refresh_token } = response.data;
                    
                    localStorage.setItem('access_token', access_token);
                    localStorage.setItem('refresh_token', new_refresh_token);

                    // Update header and retry original request
                    originalRequest.headers.Authorization = `Bearer ${access_token}`;
                    return client(originalRequest);
                } catch (refreshError) {
                    // Refresh failed
                    localStorage.removeItem('access_token');
                    localStorage.removeItem('refresh_token');
                    window.dispatchEvent(new Event('auth:unauthorized'));
                }
            } else {
                // No refresh token available
                localStorage.removeItem('access_token');
                localStorage.removeItem('refresh_token');
                window.dispatchEvent(new Event('auth:unauthorized'));
            }
        }
        return Promise.reject(error);
    }
);

export const api: ApiService = {
    // Auth
    async login(data: LoginRequest): Promise<LoginResponse> {
        const response = await client.post<LoginResponse>('/login', data);
        if (response.data.access_token) {
            localStorage.setItem('access_token', response.data.access_token);
            localStorage.setItem('refresh_token', response.data.refresh_token);
        }
        return response.data;
    },

    async logout(): Promise<void> {
        const refreshToken = localStorage.getItem('refresh_token');
        if (refreshToken) {
             try {
                await client.post('/logout', { refresh_token: refreshToken });
             } catch (e) {
                 // Ignore error on logout
             }
        }
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
    },

    async getMe(): Promise<User> {
        const response = await client.get<User>('/me');
        return response.data;
    },

    // Users
    async getUsers(): Promise<User[]> {
        const response = await client.get<User[]>('/users');
        return response.data;
    },

    async createUser(data: CreateUserRequest): Promise<{ user: User; password: string }> {
        const response = await client.post<{ user: User; password: string }>('/users', data);
        return response.data;
    },

    async deleteUser(id: number): Promise<void> {
        await client.delete(`/users/${id}`);
    },

    async updateUserRole(id: number, role: 'admin' | 'user'): Promise<void> {
        await client.put(`/users/${id}/role`, { role });
    },

    async updateUserStatus(id: number, is_disabled: boolean): Promise<void> {
        await client.put(`/users/${id}/status`, { is_disabled });
    },

    async resetUserPassword(id: number, data: { new_password: string }): Promise<void> {
        await client.post(`/users/${id}/reset-password`, data);
    },

    async changePassword(data: { old_password: string, new_password: string }): Promise<void> {
        await client.post(`/change-password`, data);
    },

    // Devices
    async getDevices(): Promise<Device[]> {
        const response = await client.get<Device[]>('/devices');
        return response.data;
    },

    async createDevice(data: CreateDeviceRequest): Promise<Device> {
        const response = await client.post<Device>('/devices', data);
        return response.data;
    },

    async updateDevice(id: number, data: UpdateDeviceRequest): Promise<Device> {
        const response = await client.put<Device>(`/devices/${id}`, data);
        return response.data;
    },

    async deleteDevice(id: number): Promise<void> {
        await client.delete(`/devices/${id}`);
    },

    async wakeDevice(id: number): Promise<void> {
        await client.post(`/devices/${id}/wake`);
    },

    async shutdownDevice(id: number): Promise<void> {
        await client.post(`/devices/${id}/shutdown`);
    }
};
