export interface User {
    id: number;
    username: string;
    role: 'admin' | 'user';
    last_login_at?: string;
    force_password_change: boolean;
    is_disabled: boolean;
}

export interface Device {
    id: number;
    name: string;
    mac_address: string;
    ip_address?: string;
    broadcast_addr?: string;
    icon?: string;
    is_online: boolean;
    last_seen_at?: string;
}

export interface CreateUserRequest {
    username: string;
}

export interface LoginRequest {
    username: string;
    password: string;
    remember_me?: boolean;
}

export interface LoginResponse {
    message: string;
    user: User;
    access_token: string;
    refresh_token: string;
}

export interface RefreshTokenRequest {
    refresh_token: string;
}

export interface RefreshTokenResponse {
    access_token: string;
    refresh_token: string;
}

export interface CreateDeviceRequest {
    name: string;
    mac_address: string;
    ip_address?: string;
    broadcast_addr?: string;
    icon?: string;
}

export interface UpdateDeviceRequest {
    name?: string;
    mac_address?: string;
    ip_address?: string;
    broadcast_addr?: string;
    icon?: string;
}
