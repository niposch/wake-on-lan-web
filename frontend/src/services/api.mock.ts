import type { ApiService } from './api.interface';
import type { 
    User, Device, LoginRequest, LoginResponse, CreateUserRequest, CreateDeviceRequest 
} from '../types';

const STORAGE_KEY_USERS = 'wol_mock_users';
const STORAGE_KEY_DEVICES = 'wol_mock_devices';

const DEFAULT_ADMIN: User = {
    id: 1,
    username: 'admin',
    role: 'admin',
    force_password_change: false,
    last_login_at: new Date().toISOString()
};

// Mock Auth Token
// const MOCK_TOKEN = "mock-jwt-token";

const loadUsers = (): User[] => {
    const s = localStorage.getItem(STORAGE_KEY_USERS);
    if (s) return JSON.parse(s);
    return [DEFAULT_ADMIN];
};

const saveUsers = (users: User[]) => {
    localStorage.setItem(STORAGE_KEY_USERS, JSON.stringify(users));
};

const loadDevices = (): Device[] => {
    const s = localStorage.getItem(STORAGE_KEY_DEVICES);
    if (s) return JSON.parse(s);
    return [];
};

const saveDevices = (devices: Device[]) => {
    localStorage.setItem(STORAGE_KEY_DEVICES, JSON.stringify(devices));
};

export const api: ApiService = {
    async login(data: LoginRequest): Promise<LoginResponse> {
        await new Promise(r => setTimeout(r, 500)); // Simulate latency
        const users = loadUsers();
        const user = users.find(u => u.username === data.username);
        
        if (user) {
            // For mock, accept any password or specific mock password
            // In a real mock we might check password, but here we assume success for simplicity 
            // OR checks against a hardcoded "password" property we don't store in User type?
            // Let's just say password must be 'password' for everyone or 'admin' for admin.
            
            if (data.password) {
                 return {
                    message: "Login successful",
                    user: user,
                };
            }
        }
        throw new Error("Invalid credentials");
    },

    async logout(): Promise<void> {
        // no-op
    },

    async getMe(): Promise<User> {
        // For mock, just return the first user or admin
        return loadUsers()[0];
    },

    async getUsers(): Promise<User[]> {
        return loadUsers();
    },

    async createUser(data: CreateUserRequest): Promise<{ user: User; password: string }> {
        const users = loadUsers();
        if (users.find(u => u.username === data.username)) {
            throw new Error("Username taken");
        }
        
        const newUser: User = {
            id: Date.now(),
            username: data.username,
            role: 'user',
            force_password_change: true,
        };
        
        users.push(newUser);
        saveUsers(users);
        
        return { user: newUser, password: "temp-password" };
    },

    async deleteUser(id: number): Promise<void> {
        const users = loadUsers().filter(u => u.id !== id);
        saveUsers(users);
    },

    async updateUserRole(id: number, role: 'admin' | 'user'): Promise<void> {
        const users = loadUsers();
        const user = users.find(u => u.id === id);
        if (user) {
            user.role = role;
            saveUsers(users);
        }
    },

    async resetUserPassword(id: number, data: { new_password: string }): Promise<void> {
        // Mock: just pretend we did it.
        console.log(`Reset password for user ${id} to ${data.new_password}`);
    },

    async changePassword(data: { old_password: string, new_password: string }): Promise<void> {
        console.log(`Changed password from ${data.old_password} to ${data.new_password}`);
    },

    async getDevices(): Promise<Device[]> {
        return loadDevices();
    },

    async createDevice(data: CreateDeviceRequest): Promise<Device> {
        const devices = loadDevices();
        const newDevice: Device = {
            id: Date.now(),
            name: data.name,
            mac_address: data.mac_address,
            ip_address: data.ip_address,
            broadcast_addr: data.broadcast_addr,
            icon: data.icon,
            is_online: false,
        };
        devices.push(newDevice);
        saveDevices(devices);
        return newDevice;
    },

    async deleteDevice(id: number): Promise<void> {
        const devices = loadDevices().filter(d => d.id !== id);
        saveDevices(devices);
    },

    async wakeDevice(id: number): Promise<void> {
        console.log(`Waking device ${id}`);
        // Simulate device coming online?
        const devices = loadDevices();
        const device = devices.find(d => d.id === id);
        if (device) {
            device.is_online = true;
            device.last_seen_at = new Date().toISOString();
            saveDevices(devices);
        }
    },

    async shutdownDevice(id: number): Promise<void> {
        console.log(`Shutting down device ${id}`);
        const devices = loadDevices();
        const device = devices.find(d => d.id === id);
        if (device) {
            device.is_online = false;
            saveDevices(devices);
        }
    }
};
