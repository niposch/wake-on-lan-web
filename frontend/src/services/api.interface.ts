import type { 
    User, Device, LoginRequest, LoginResponse, CreateUserRequest, CreateDeviceRequest, UpdateDeviceRequest, AdminResetPasswordRequest, AdminResetPasswordResponse 
} from '../types';

export interface ApiService {
    // Auth
    login(data: LoginRequest): Promise<LoginResponse>;
    logout(): Promise<void>;
    getMe(): Promise<User>;
    
    // Users
    getUsers(): Promise<User[]>;
    createUser(data: CreateUserRequest): Promise<{ user: User; password: string }>;
    deleteUser(id: number): Promise<void>;
    updateUserRole(id: number, role: 'admin' | 'user'): Promise<void>;
    updateUserStatus(id: number, is_disabled: boolean): Promise<void>;
    resetUserPassword(id: number, data: AdminResetPasswordRequest): Promise<AdminResetPasswordResponse>;
    changePassword(data: { old_password: string, new_password: string }): Promise<void>;
    
    // Devices
    getDevices(): Promise<Device[]>;
    createDevice(data: CreateDeviceRequest): Promise<Device>;
    updateDevice(id: number, data: UpdateDeviceRequest): Promise<Device>;
    deleteDevice(id: number): Promise<void>;
    wakeDevice(id: number): Promise<void>;
    shutdownDevice(id: number): Promise<void>;
}
